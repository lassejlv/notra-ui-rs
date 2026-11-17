use std::rc::Rc;
use std::time::Duration;

use ::rune::{El, Key, KeyEvent, Tx, col, el, icon, row, text};

use super::popup;
use crate::components::overlays::Placement;
use crate::components::overlays::menu::{
    DepthSurface, MOTION, MenuEntry, MenuItem, MenuSkin, PLACEMENT, SUBMENU_DELAY,
    SUBMENU_PLACEMENT, metrics,
};
use crate::icons::Icon;
use crate::rune::{ToRune, mono, sans};
use crate::theme::Theme;

/// Whether a dropdown menu is open and which submenus are, owned by the app.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuState {
    pub open: bool,
    /// Indices of the open submenu triggers, outermost first.
    pub submenus: Vec<usize>,
}

impl MenuState {
    pub fn close(&mut self) {
        self.open = false;
        self.submenus.clear();
    }
}

/// Picks the menu's state out of the app's.
pub type Lens<S> = fn(&mut S) -> &mut MenuState;

type Select<S> = Rc<dyn Fn(&mut S, &MenuItem)>;

/// Notra's dropdown menu: the trigger stays where the view puts it, the Depth menu and its
/// submenus mount later in the tree (an overlay layer).
///
/// ```ignore
/// let menu = DropdownMenu::new("workspace", &entries).width(224.0);
/// let trigger = menu.trigger(outline_button("Open menu", s.menu.open, t), |s: &mut App| &mut s.menu);
/// layer.extend(menu.popups(&s.menu, t, |s: &mut App| &mut s.menu, |s: &mut App, item| s.pick(&item.id)));
/// ```
///
/// The open menu is modal: it takes focus (its first row), arrows and type-ahead move between
/// rows, Enter or Space activate, ArrowRight and ArrowLeft open and close submenus, Escape or
/// a press outside closes it and focus returns to the trigger.
pub struct DropdownMenu<'a> {
    key: String,
    entries: &'a [MenuEntry],
    width: Option<f32>,
    placement: Placement,
}

impl<'a> DropdownMenu<'a> {
    pub fn new(key: impl Into<String>, entries: &'a [MenuEntry]) -> Self {
        Self {
            key: key.into(),
            entries,
            width: None,
            placement: PLACEMENT,
        }
    }

    /// The menu's width (`w-56`); 128 at least.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    fn trigger_key(&self) -> String {
        format!("{}-trigger", self.key)
    }

    /// Wraps `trigger` as the menu's anchor: pressing it toggles the menu, and ArrowDown or
    /// ArrowUp open it from the keyboard.
    pub fn trigger<S: 'static>(&self, trigger: El, state: &MenuState, lens: Lens<S>) -> El {
        let first = self.first_row_key();
        let last = self.last_row_key();
        row()
            .key(self.trigger_key())
            .child(trigger.expanded(state.open))
            .on_press(move |s: &mut S| {
                let m = lens(s);
                if m.open {
                    m.close();
                } else {
                    m.open = true;
                }
            })
            .on_key(move |s: &mut S, ev: &KeyEvent, tx: &mut Tx| {
                let target = match ev.key {
                    Key::Down => &first,
                    Key::Up => &last,
                    _ => return false,
                };
                lens(s).open = true;
                tx.focus(target.clone(), true);
                true
            })
    }

    fn row_key(&self, path: &[usize]) -> String {
        let mut key = format!("{}-row", self.key);
        for i in path {
            key.push_str(&format!("-{i}"));
        }
        key
    }

    fn first_row_key(&self) -> String {
        let i = self.entries.iter().position(|e| e.enabled_item().is_some());
        self.row_key(&[i.unwrap_or(0)])
    }

    fn last_row_key(&self) -> String {
        let i = self
            .entries
            .iter()
            .rposition(|e| e.enabled_item().is_some());
        self.row_key(&[i.unwrap_or(0)])
    }

    /// The open menu and its open submenus, to mount after the trigger. Empty while closed.
    pub fn popups<S: 'static>(
        &self,
        state: &MenuState,
        theme: &Theme,
        lens: Lens<S>,
        on_select: impl Fn(&mut S, &MenuItem) + 'static,
    ) -> Vec<El> {
        if !state.open {
            return Vec::new();
        }
        let skin = MenuSkin::new(theme);
        let select: Select<S> = Rc::new(on_select);
        let motion = MOTION;
        let mut out = Vec::new();
        let width = self
            .width
            .unwrap_or(metrics::MIN_WIDTH)
            .max(metrics::MIN_WIDTH);
        let root = self.surface(
            self.entries,
            &[],
            state,
            &skin,
            lens,
            &select,
            Some(width),
            metrics::MIN_WIDTH,
        );
        let trigger = self.trigger_key();
        out.push(
            popup(root, &trigger, self.placement, &motion)
                .focus_return(trigger.clone())
                .modal(true, move |s: &mut S| lens(s).close()),
        );
        let mut entries = self.entries;
        let mut path = Vec::new();
        for &index in &state.submenus {
            let Some(sub) = entries
                .get(index)
                .and_then(MenuEntry::enabled_item)
                .and_then(MenuItem::submenu_entries)
            else {
                break;
            };
            path.push(index);
            let surface = self.surface(
                sub,
                &path,
                state,
                &skin,
                lens,
                &select,
                None,
                metrics::SUB_MIN_WIDTH,
            );
            if let Some(menu) = out.last_mut() {
                let anchor = self.row_key(&path);
                let placed = popup(surface, &anchor, SUBMENU_PLACEMENT, &motion);
                *menu = std::mem::replace(menu, el()).child(placed);
            }
            entries = sub;
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn surface<S: 'static>(
        &self,
        entries: &[MenuEntry],
        path: &[usize],
        state: &MenuState,
        skin: &MenuSkin,
        lens: Lens<S>,
        select: &Select<S>,
        width: Option<f32>,
        min_width: f32,
    ) -> El {
        let level = path.len();
        let rows = entries
            .iter()
            .enumerate()
            .map(|(index, entry)| match entry {
                MenuEntry::Label(label) => row()
                    .key(format!("{}-label-{index}", self.row_key(path)))
                    .px(metrics::LABEL_PADDING_X)
                    .py(metrics::LABEL_PADDING_Y)
                    .a11y_hidden()
                    .child(
                        text(label.clone())
                            .font(sans(metrics::LABEL_FONT, 500.0, metrics::LABEL_LINE))
                            .color(skin.label.rune()),
                    ),
                MenuEntry::Separator => el()
                    .key(format!("{}-sep-{index}", self.row_key(path)))
                    .h(1.0)
                    .mx(metrics::SEPARATOR_MARGIN_X)
                    .my(metrics::SEPARATOR_MARGIN_Y)
                    .fill(skin.separator.rune())
                    .a11y_hidden(),
                MenuEntry::Item(item) => {
                    let mut row_path = path.to_vec();
                    row_path.push(index);
                    let open =
                        state.submenus.get(level) == Some(&index) && state.submenus.len() > level;
                    self.row(item, &row_path, open, skin, lens, select)
                }
            });
        let surface = depth_surface(&skin.surface)
            .key(format!("{}-menu-{level}", self.key))
            .role("menu")
            .focus_group()
            .items_stretch()
            .min_w(min_width)
            .max_h(metrics::MAX_HEIGHT)
            .padding(metrics::SURFACE_PADDING)
            .children(rows);
        match width {
            Some(w) => surface.w(w),
            None => surface,
        }
    }

    fn row<S: 'static>(
        &self,
        item: &MenuItem,
        path: &[usize],
        open: bool,
        skin: &MenuSkin,
        lens: Lens<S>,
        select: &Select<S>,
    ) -> El {
        let level = path.len() - 1;
        let index = path[level];
        let key = self.row_key(path);
        let (_, text_color, icon_color) = skin.row(false, item.destructive);
        let (fill, hot_text, hot_icon) = skin.row(true, item.destructive);
        let glyph = |i: Icon, color: crate::theme::Color| {
            icon(i.rune(), metrics::ICON)
                .color(color.rune())
                .group_color(hot_icon.rune())
                .a11y_hidden()
        };
        let mut e = row()
            .key(key.clone())
            .role(item.role())
            .label(item.label.clone())
            .min_h(metrics::ITEM_HEIGHT)
            .pl(if item.inset {
                metrics::INSET_PADDING_LEFT
            } else {
                metrics::ITEM_PADDING_X
            })
            .pr(if item.checked().is_some() {
                metrics::INDICATOR_ROW_PADDING_RIGHT
            } else {
                metrics::ITEM_PADDING_X
            })
            .py(metrics::ITEM_PADDING_Y)
            .gap(metrics::ITEM_GAP)
            .center_y()
            .radius(metrics::ITEM_RADIUS)
            .continuous_corners()
            .hover_group();
        if let Some(on) = item.checked() {
            e = e.checked(on);
        }
        if let Some(s) = &item.shortcut {
            e = e.shortcut_hint(s.clone());
        }
        if let Some(i) = item.icon {
            e = e.child(glyph(i, icon_color));
        }
        e = e.child(
            text(item.label.clone())
                .font(sans(metrics::ITEM_FONT, 400.0, metrics::ITEM_LINE))
                .color(text_color.rune())
                .group_color(hot_text.rune())
                .grow(1.0)
                .truncate(),
        );
        if let Some(s) = &item.shortcut {
            e = e.child(
                text(s.clone())
                    .font(mono(metrics::SHORTCUT_FONT, 400.0, metrics::SHORTCUT_LINE))
                    .color(skin.shortcut.rune())
                    .pl(metrics::SHORTCUT_PADDING_LEFT)
                    .a11y_hidden(),
            );
        }
        let submenu = item.submenu_entries();
        if submenu.is_some() {
            e = e
                .expanded(open)
                .child(glyph(Icon::ChevronRight, icon_color));
        }
        if item.checked() == Some(true) {
            e = e.child(
                el().pin(None, Some(8.0), Some(metrics::INDICATOR_RIGHT), None)
                    .size(metrics::ICON, metrics::ICON)
                    .a11y_hidden()
                    .child(
                        icon(Icon::Check.rune(), metrics::ICON)
                            .color(text_color.rune())
                            .group_color(hot_text.rune()),
                    ),
            );
        }
        if item.disabled {
            return e
                .layer_opacity(metrics::DISABLED_OPACITY)
                .disabled(true)
                .focusable(false);
        }
        if open {
            e = e.fill(fill.rune());
        }
        e = e
            .focusable(true)
            .focus_highlight(fill.rune())
            .cursor(::rune::Cursor::Pointer);

        let sub_first = submenu.and_then(|sub| {
            sub.iter()
                .position(|e| e.enabled_item().is_some())
                .map(|i| {
                    let mut p = path.to_vec();
                    p.push(i);
                    self.row_key(&p)
                })
        });
        let parent = (level > 0).then(|| self.row_key(&path[..level]));
        let opens = submenu.is_some();
        let closes = item.closes_menu();
        let id = item.clone();
        let select = select.clone();
        let focus_sub = sub_first.clone();
        e = e
            .on_press_tx(move |s: &mut S, tx: &mut Tx| {
                let m = lens(s);
                if opens {
                    m.submenus.truncate(level);
                    m.submenus.push(index);
                    if let Some(k) = &focus_sub {
                        tx.focus(k.clone(), true);
                    }
                    return;
                }
                if closes {
                    m.close();
                }
                select(s, &id);
            })
            .on_key(move |s: &mut S, ev: &KeyEvent, tx: &mut Tx| {
                let m = lens(s);
                match ev.key {
                    Key::Right if opens => {
                        m.submenus.truncate(level);
                        m.submenus.push(index);
                        if let Some(k) = &sub_first {
                            tx.focus(k.clone(), true);
                        }
                        true
                    }
                    Key::Left | Key::Escape if level > 0 => {
                        m.submenus.truncate(level - 1);
                        if let Some(p) = &parent {
                            tx.focus(p.clone(), true);
                        }
                        true
                    }
                    _ => false,
                }
            })
            .hover_delays(
                if opens { SUBMENU_DELAY } else { Duration::ZERO },
                Duration::ZERO,
            )
            .on_hover_change(move |s: &mut S, inside: bool, _tx: &mut Tx| {
                if !inside {
                    return;
                }
                let m = lens(s);
                m.submenus.truncate(level);
                if opens {
                    m.submenus.push(index);
                }
            });
        e
    }
}

/// The Depth popup surface (`DROPDOWN_POPUP_CLASS_NAME`): border, top-to-bottom gradient,
/// inset highlight and shadows, squircle corners. Menus and selects build on it.
pub fn depth_surface(surface: &DepthSurface) -> El {
    let mut e = col()
        .radius(metrics::SURFACE_RADIUS)
        .continuous_corners()
        .border(1.0, surface.border.rune())
        .fill(surface.fill.rune())
        .gradient(
            surface.gradient.0.over(surface.fill).rune(),
            surface.gradient.1.over(surface.fill).rune(),
        );
    for s in &surface.shadows {
        e = e.box_shadow(s.rune());
    }
    e
}
