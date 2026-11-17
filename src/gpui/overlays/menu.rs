use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use ::gpui::{
    AnyElement, App, ElementId, FocusHandle, FontWeight, Hsla, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::*, px, svg,
};

use super::{AnchorBounds, Dismiss, Fade, Popup};
use crate::components::overlays::menu::{
    MOTION, MenuCommand, MenuEntry, MenuItem, MenuKey, MenuNav, MenuSkin, OpenFocus, PLACEMENT,
    SUBMENU_DELAY, SUBMENU_PLACEMENT, item_at, metrics,
};
use crate::components::overlays::{Placement, PopupMotion};
use crate::fonts;
use crate::gpui::{ActiveTheme, gradient, shadows};
use crate::icons::Icon;
use crate::theme::{Color, Shadow};

type SelectHandler = Rc<dyn Fn(&MenuItem, &mut Window, &mut App)>;
type TriggerBuilder = Box<dyn FnOnce(bool, &mut Window, &mut App) -> AnyElement>;

/// Notra's dropdown menu: a trigger and the Depth menu it opens, with labels, separators,
/// shortcuts, checkbox and radio rows and nested submenus.
///
/// The app owns what the rows show (checked states live in the entries it passes each
/// render) and reacts to activation; the menu owns being open, its highlight and focus.
///
/// ```ignore
/// DropdownMenu::new("workspace", menu::demo_entries(self.activity, &self.view))
///     .trigger(|open, _, cx| outline_button("open-menu", "Open menu", open, cx).into_any_element())
///     .width(224.0)
///     .on_select(cx.listener(|this, item: &MenuItem, _, cx| this.pick(&item.id, cx)))
/// ```
#[derive(IntoElement)]
pub struct DropdownMenu {
    id: ElementId,
    entries: Rc<Vec<MenuEntry>>,
    trigger: Option<TriggerBuilder>,
    width: Option<f32>,
    placement: Placement,
    trigger_radius: f32,
    on_select: Option<SelectHandler>,
}

impl DropdownMenu {
    pub fn new(id: impl Into<ElementId>, entries: Vec<MenuEntry>) -> Self {
        Self {
            id: id.into(),
            entries: Rc::new(entries),
            trigger: None,
            width: None,
            placement: PLACEMENT,
            trigger_radius: 8.0,
            on_select: None,
        }
    }

    /// The trigger, built with whether the menu is open (`aria-expanded`).
    pub fn trigger(
        mut self,
        trigger: impl FnOnce(bool, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.trigger = Some(Box::new(trigger));
        self
    }

    /// Corner radius of the trigger's keyboard focus ring.
    pub fn trigger_radius(mut self, radius: f32) -> Self {
        self.trigger_radius = radius;
        self
    }

    /// The menu's width (`w-56`); by default the trigger's, at least 128 px.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// A row was activated: plain rows close the menu first, checkbox and radio rows keep it
    /// open.
    pub fn on_select(
        mut self,
        handler: impl Fn(&MenuItem, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

struct MenuState {
    open: bool,
    nav: MenuNav,
    trigger_focus: FocusHandle,
    menu_focus: FocusHandle,
    anchor: AnchorBounds,
    rows: HashMap<Vec<usize>, AnchorBounds>,
    fades: Vec<Fade>,
    pending: Option<(usize, usize, Instant)>,
}

impl MenuState {
    fn open(&mut self, entries: &[MenuEntry], focus: OpenFocus) {
        self.open = true;
        self.nav = MenuNav::opened(entries, focus);
        self.pending = None;
    }

    fn close(&mut self) {
        self.open = false;
        self.pending = None;
    }

    fn row_anchor(&mut self, path: &[usize]) -> AnchorBounds {
        self.rows.entry(path.to_vec()).or_default().clone()
    }
}

fn menu_key(event: &KeyDownEvent) -> Option<MenuKey> {
    let ks = &event.keystroke;
    Some(match ks.key.as_str() {
        "up" => MenuKey::Up,
        "down" => MenuKey::Down,
        "home" => MenuKey::Home,
        "end" => MenuKey::End,
        "left" => MenuKey::Left,
        "right" => MenuKey::Right,
        "enter" => MenuKey::Enter,
        "space" => MenuKey::Space,
        "escape" => MenuKey::Escape,
        "tab" => MenuKey::Tab,
        _ => {
            let mut chars = ks.key_char.as_deref()?.chars();
            let c = chars.next()?;
            if chars.next().is_some() || ks.modifiers.control || ks.modifiers.platform {
                return None;
            }
            MenuKey::Char(c)
        }
    })
}

/// Paths of every submenu trigger in `entries`, outermost first.
fn submenu_paths(entries: &[MenuEntry], prefix: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
    for (i, entry) in entries.iter().enumerate() {
        if let MenuEntry::Item(item) = entry
            && let Some(sub) = item.submenu_entries()
        {
            prefix.push(i);
            out.push(prefix.clone());
            submenu_paths(sub, prefix, out);
            prefix.pop();
        }
    }
}

fn entries_at_path<'a>(root: &'a [MenuEntry], path: &[usize]) -> Option<&'a [MenuEntry]> {
    let mut entries = root;
    for &i in path {
        entries = match entries.get(i)? {
            MenuEntry::Item(item) => item.submenu_entries()?,
            _ => return None,
        };
    }
    Some(entries)
}

impl RenderOnce for DropdownMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let now = Instant::now();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| MenuState {
            open: false,
            nav: MenuNav::default(),
            trigger_focus: cx.focus_handle(),
            menu_focus: cx.focus_handle(),
            anchor: AnchorBounds::default(),
            rows: HashMap::new(),
            fades: Vec::new(),
            pending: None,
        });
        let entries = self.entries.clone();

        state.update(cx, |s, _| {
            if let Some((level, index, at)) = s.pending
                && now - at >= SUBMENU_DELAY
            {
                s.pending = None;
                if s.nav.highlighted(level) == Some(index) {
                    s.nav.open_submenu(&entries, level, false);
                }
            }
            let depth = s.nav.depth();
            if s.fades.len() < depth.max(1) {
                s.fades.resize(depth.max(1), Fade::default());
            }
            let open = s.open;
            for (level, fade) in s.fades.iter_mut().enumerate() {
                let to = if open { s.nav.highlighted(level) } else { None };
                fade.sync(if level < depth { to } else { None }, now);
            }
        });

        let s = state.read(cx);
        let open = s.open;
        let trigger_focus = s.trigger_focus.clone();
        let menu_focus = s.menu_focus.clone();
        let anchor = s.anchor.clone();
        let nav = s.nav.clone();
        if s.pending.is_some() || s.fades.iter().any(|f| f.is_animating(now)) {
            window.request_animation_frame();
        }

        let theme = *cx.notra();
        let skin = Rc::new(MenuSkin::new(&theme));
        let motion = if cx.reduce_motion() {
            PopupMotion::NONE
        } else {
            MOTION
        };

        let select = self.on_select.clone();
        let apply: Apply = {
            let state = state.clone();
            let entries = entries.clone();
            Rc::new(
                move |command: MenuCommand, window: &mut Window, cx: &mut App| match command {
                    MenuCommand::Activate { path, close } => {
                        if close {
                            state.update(cx, |s, cx| {
                                s.close();
                                cx.notify();
                            });
                        }
                        if let (Some(item), Some(select)) = (item_at(&entries, &path), &select) {
                            select(item, window, cx);
                        }
                    }
                    MenuCommand::Close => state.update(cx, |s, cx| {
                        s.close();
                        cx.notify();
                    }),
                    MenuCommand::Changed | MenuCommand::None => {}
                },
            )
        };

        let trigger_el = self
            .trigger
            .map(|build| build(open, window, cx))
            .unwrap_or_else(|| div().into_any_element());

        let width = self.width;
        let mut popups: Vec<AnyElement> = Vec::new();
        let root_popup = {
            let state = state.clone();
            let key_state = state.clone();
            let entries = entries.clone();
            let skin = skin.clone();
            let apply = apply.clone();
            let key_apply = apply.clone();
            let key_entries = entries.clone();
            Popup::new((self.id.clone(), "menu"), anchor.clone())
                .open(open)
                .placement(self.placement)
                .motion(motion)
                .focus(menu_focus.clone())
                .on_key(move |event, window, cx| {
                    let Some(key) = menu_key(event) else {
                        return false;
                    };
                    let command = key_state.update(cx, |s, cx| {
                        let command = s.nav.key(&key_entries, key);
                        cx.notify();
                        command
                    });
                    let handled = command != MenuCommand::None
                        || !matches!(key, MenuKey::Escape | MenuKey::Tab);
                    key_apply(command, window, cx);
                    handled
                })
                .on_dismiss({
                    let state = state.clone();
                    move |_: Dismiss, _, cx| {
                        state.update(cx, |s, cx| {
                            s.close();
                            cx.notify();
                        })
                    }
                })
                .content(move |_, window, cx| {
                    let w =
                        width.unwrap_or_else(|| anchor_width(&state, cx).max(metrics::MIN_WIDTH));
                    surface(
                        &state,
                        &entries,
                        &[],
                        Some(w),
                        metrics::MIN_WIDTH,
                        &skin,
                        &apply,
                        window,
                        cx,
                    )
                })
        };
        popups.push(root_popup.into_any_element());

        let mut paths = Vec::new();
        submenu_paths(&entries, &mut Vec::new(), &mut paths);
        for (n, path) in paths.into_iter().enumerate() {
            let level = path.len() - 1;
            let sub_open = open
                && path
                    .iter()
                    .enumerate()
                    .all(|(l, &i)| nav.is_submenu_open(l, i));
            let row_anchor = state.update(cx, |s, _| s.row_anchor(&path));
            let state = state.clone();
            let entries = entries.clone();
            let skin = skin.clone();
            let apply = apply.clone();
            popups.push(
                Popup::new(
                    (self.id.clone(), SharedString::from(format!("sub-{n}"))),
                    row_anchor,
                )
                .open(sub_open)
                .placement(SUBMENU_PLACEMENT)
                .motion(motion)
                .priority(level + 2)
                .content(move |_, window, cx| {
                    surface(
                        &state,
                        &entries,
                        &path,
                        None,
                        metrics::SUB_MIN_WIDTH,
                        &skin,
                        &apply,
                        window,
                        cx,
                    )
                })
                .into_any_element(),
            );
        }

        let toggle_state = state.clone();
        let key_state = state.clone();
        let click_entries = entries.clone();
        let key_entries = entries.clone();
        let ring = theme.ring.fade(0.5);
        div()
            .id((self.id.clone(), "trigger"))
            .relative()
            .flex_none()
            .rounded(px(self.trigger_radius))
            .track_focus(&trigger_focus)
            .tab_stop(true)
            .focus_visible(move |style| style.shadow(shadows(&[Shadow::drop(0.0, 0.0, 3.0, ring)])))
            .on_click(move |_, window, cx| {
                toggle_state.update(cx, |s, cx| {
                    if s.open {
                        s.close();
                    } else {
                        s.open(&click_entries, OpenFocus::None);
                    }
                    cx.notify();
                });
                window.refresh();
            })
            .on_key_down(move |event: &KeyDownEvent, _, cx| {
                let focus = match event.keystroke.key.as_str() {
                    "enter" | "space" | "down" => OpenFocus::First,
                    "up" => OpenFocus::Last,
                    _ => return,
                };
                cx.stop_propagation();
                key_state.update(cx, |s, cx| {
                    s.open(&key_entries, focus);
                    cx.notify();
                });
            })
            .child(trigger_el)
            .child(anchor.tracker())
            .children(popups)
    }
}

fn anchor_width(state: &::gpui::Entity<MenuState>, cx: &App) -> f32 {
    state
        .read(cx)
        .anchor
        .get()
        .map_or(0.0, |b| f32::from(b.size.width))
}

type Apply = Rc<dyn Fn(MenuCommand, &mut Window, &mut App)>;

/// One open menu level: the Depth surface and its rows.
#[allow(clippy::too_many_arguments)]
fn surface(
    state: &::gpui::Entity<MenuState>,
    root: &Rc<Vec<MenuEntry>>,
    path: &[usize],
    width: Option<f32>,
    min_width: f32,
    skin: &Rc<MenuSkin>,
    apply: &Apply,
    _window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let level = path.len();
    let Some(entries) = entries_at_path(root, path) else {
        return div().into_any_element();
    };
    let now = Instant::now();
    let s = state.read(cx);
    let fade = s.fades.get(level).copied().unwrap_or_default();
    let open_sub = (0..level)
        .all(|l| s.nav.highlighted(l) == path.get(l).copied())
        .then(|| {
            s.nav
                .highlighted(level)
                .filter(|_| s.nav.depth() > level + 1)
        })
        .flatten();

    let rows: Vec<AnyElement> = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| match entry {
            MenuEntry::Label(text) => div()
                .px(px(metrics::LABEL_PADDING_X))
                .py(px(metrics::LABEL_PADDING_Y))
                .text_size(px(metrics::LABEL_FONT))
                .line_height(px(metrics::LABEL_LINE))
                .font_weight(FontWeight(500.0))
                .text_color(skin.label)
                .child(SharedString::from(text.clone()))
                .into_any_element(),
            MenuEntry::Separator => div()
                .mx(px(metrics::SEPARATOR_MARGIN_X))
                .my(px(metrics::SEPARATOR_MARGIN_Y))
                .h(px(1.0))
                .bg(skin.separator)
                .into_any_element(),
            MenuEntry::Item(item) => {
                let mut row_path = path.to_vec();
                row_path.push(index);
                let tracker = item
                    .submenu_entries()
                    .is_some()
                    .then(|| state.update(cx, |s, _| s.row_anchor(&row_path)).tracker());
                let lit =
                    fade.amount(index, now)
                        .max(if open_sub == Some(index) { 1.0 } else { 0.0 });
                row(state, root, item, level, index, lit, skin, apply, tracker)
            }
        })
        .collect();

    let s = &skin.surface;
    div()
        .id(("menu-level", level))
        .when_some(width, |d, w| d.w(px(w)))
        .min_w(px(min_width))
        .max_h(px(metrics::MAX_HEIGHT))
        .overflow_y_scroll()
        .p(px(metrics::SURFACE_PADDING))
        .rounded(px(metrics::SURFACE_RADIUS))
        .border_1()
        .border_color(s.border)
        .bg(gradient(
            s.gradient.0.over(s.fill),
            s.gradient.1.over(s.fill),
        ))
        .shadow(shadows(&s.shadows))
        .text_color(s.text)
        .font_family(fonts::SANS)
        .flex()
        .flex_col()
        .children(rows)
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn row(
    state: &::gpui::Entity<MenuState>,
    root: &Rc<Vec<MenuEntry>>,
    item: &MenuItem,
    level: usize,
    index: usize,
    lit: f32,
    skin: &MenuSkin,
    apply: &Apply,
    tracker: Option<impl IntoElement>,
) -> AnyElement {
    let (rest_fill, rest_text, rest_icon) = skin.row(false, item.destructive);
    let (hot_fill, hot_text, hot_icon) = skin.row(true, item.destructive);
    let mix = |a: Color, b: Color| -> Hsla {
        if lit <= 0.0 {
            a.into()
        } else if lit >= 1.0 {
            b.into()
        } else {
            a.mix(b, lit).into()
        }
    };
    let fill = hot_fill.fade(lit).over(rest_fill);
    let (text, icon_color) = (mix(rest_text, hot_text), mix(rest_icon, hot_icon));
    let indicator = item.checked().is_some();
    let glyph = |icon: Icon, color: Hsla| {
        svg()
            .path(icon.path())
            .size(px(metrics::ICON))
            .flex_none()
            .text_color(color)
    };

    let hover_state = state.clone();
    let hover_root = root.clone();
    let click_state = state.clone();
    let click_root = root.clone();
    let click_apply = apply.clone();
    let disabled = item.disabled;

    div()
        .id(("menu-row", level * 1000 + index))
        .relative()
        .flex()
        .items_center()
        .gap(px(metrics::ITEM_GAP))
        .min_h(px(metrics::ITEM_HEIGHT))
        .px(px(metrics::ITEM_PADDING_X))
        .py(px(metrics::ITEM_PADDING_Y))
        .when(indicator, |d| {
            d.pr(px(metrics::INDICATOR_ROW_PADDING_RIGHT))
        })
        .when(item.inset, |d| d.pl(px(metrics::INSET_PADDING_LEFT)))
        .rounded(px(metrics::ITEM_RADIUS))
        .bg(fill)
        .when_some(skin.highlight_inset.filter(|_| lit > 0.0), |d, inset| {
            d.shadow(shadows(&[Shadow {
                color: inset.color.fade(lit),
                ..inset
            }]))
        })
        .text_size(px(metrics::ITEM_FONT))
        .line_height(px(metrics::ITEM_LINE))
        .text_color(text)
        .when(disabled, |d| d.opacity(metrics::DISABLED_OPACITY))
        .when(!disabled, |d| {
            d.cursor_pointer()
                .on_hover(move |hovered, window, cx| {
                    hover_state.update(cx, |s, cx| {
                        if *hovered {
                            if s.nav.hover(&hover_root, level, index) {
                                s.pending = Some((level, index, Instant::now()));
                            }
                        } else {
                            s.nav.unhover(level);
                        }
                        cx.notify();
                    });
                    window.refresh();
                })
                .on_click(move |_, window, cx| {
                    let command = click_state.update(cx, |s, cx| {
                        cx.notify();
                        s.nav.press(&click_root, level, index)
                    });
                    click_apply(command, window, cx);
                })
        })
        .when_some(item.icon, |d, icon| d.child(glyph(icon, icon_color)))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(SharedString::from(item.label.clone())),
        )
        .when_some(item.shortcut.clone(), |d, shortcut| {
            d.child(
                div()
                    .pl(px(metrics::SHORTCUT_PADDING_LEFT))
                    .font_family(fonts::MONO)
                    .text_size(px(metrics::SHORTCUT_FONT))
                    .line_height(px(metrics::SHORTCUT_LINE))
                    .text_color(skin.shortcut)
                    .child(SharedString::from(shortcut)),
            )
        })
        .when(item.submenu_entries().is_some(), |d| {
            d.child(glyph(Icon::ChevronRight, icon_color))
        })
        .when(item.checked() == Some(true), |d| {
            d.child(
                div()
                    .absolute()
                    .right(px(metrics::INDICATOR_RIGHT))
                    .top_0()
                    .bottom_0()
                    .flex()
                    .items_center()
                    .child(glyph(Icon::Check, text)),
            )
        })
        .children(tracker)
        .into_any_element()
}
