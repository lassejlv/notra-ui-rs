use std::rc::Rc;
use std::time::Instant;

use ::gpui::{
    AnyElement, App, ElementId, Entity, FocusHandle, FontWeight, Hsla, InteractiveElement,
    IntoElement, KeyDownEvent, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window, div, prelude::*, px, svg,
};

use super::{AnchorBounds, Dismiss, Fade, Popup};
use crate::components::button::{ButtonHover, ButtonSkin};
use crate::components::overlays::PopupMotion;
use crate::components::overlays::menu::{MenuKey, metrics as row};
use crate::components::overlays::select::{
    COLLISION, MOTION, OPTION_PADDING_RIGHT, PLACEMENT, SelectCommand, SelectEntry, SelectNav,
    SelectSize, SelectSkin, SelectVariant, TRANSITION, TriggerSkin, label_of, metrics, options,
};
use crate::fonts;
use crate::gpui::{ActiveTheme, gradient, shadows};
use crate::icons::Icon;
use crate::motion::Animated;
use crate::theme::{Color, Shadow};

type ChangeHandler = Rc<dyn Fn(&str, &mut Window, &mut App)>;
type Commit = Rc<dyn Fn(SelectCommand, &mut Window, &mut App)>;

/// Notra's select: a Depth (or ghost) button showing the value, opening a listbox below it.
///
/// The app owns the value; the select owns being open, its highlight and focus.
///
/// ```ignore
/// Select::new("frequency", select::demo_grouped())
///     .value(Some(self.frequency.clone()))
///     .width(192.0)
///     .on_change(cx.listener(|this, value: &str, _, cx| { this.frequency = value.into(); cx.notify(); }))
/// ```
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    entries: Rc<Vec<SelectEntry>>,
    value: Option<String>,
    placeholder: SharedString,
    variant: SelectVariant,
    size: SelectSize,
    width: Option<f32>,
    disabled: bool,
    on_change: Option<ChangeHandler>,
}

impl Select {
    pub fn new(id: impl Into<ElementId>, entries: Vec<SelectEntry>) -> Self {
        Self {
            id: id.into(),
            entries: Rc::new(entries),
            value: None,
            placeholder: SharedString::default(),
            variant: SelectVariant::Default,
            size: SelectSize::Default,
            width: None,
            disabled: false,
            on_change: None,
        }
    }

    pub fn value(mut self, value: Option<String>) -> Self {
        self.value = value;
        self
    }

    /// Shown in `text-muted-foreground` while nothing is selected.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn variant(mut self, variant: SelectVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: SelectSize) -> Self {
        self.size = size;
        self
    }

    /// The trigger's width (`w-48`); by default it fits its value (`w-fit`).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

struct SelectState {
    open: bool,
    nav: SelectNav,
    trigger_focus: FocusHandle,
    list_focus: FocusHandle,
    anchor: AnchorBounds,
    hover: Animated,
    expanded: Animated,
    fade: Fade,
}

fn select_key(event: &KeyDownEvent) -> Option<MenuKey> {
    let ks = &event.keystroke;
    Some(match ks.key.as_str() {
        "up" => MenuKey::Up,
        "down" => MenuKey::Down,
        "home" => MenuKey::Home,
        "end" => MenuKey::End,
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

fn mix(a: Color, b: Color, t: f32) -> Color {
    if t <= 0.0 {
        a
    } else if t >= 1.0 {
        b
    } else {
        a.mix(b, t)
    }
}

/// The skin between `a` and `b`.
fn blend(a: &ButtonSkin, b: &ButtonSkin, t: f32) -> ButtonSkin {
    ButtonSkin {
        fill: mix(a.fill, b.fill, t),
        gradient: match (a.gradient, b.gradient) {
            (Some((t0, b0)), Some((t1, b1))) => Some((mix(t0, t1, t), mix(b0, b1, t))),
            (g, _) => g,
        },
        border: mix(a.border, b.border, t),
        text: mix(a.text, b.text, t),
        ..a.clone()
    }
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let now = Instant::now();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| SelectState {
            open: false,
            nav: SelectNav::default(),
            trigger_focus: cx.focus_handle(),
            list_focus: cx.focus_handle(),
            anchor: AnchorBounds::default(),
            hover: Animated::new(0.0, TRANSITION),
            expanded: Animated::new(0.0, TRANSITION),
            fade: Fade::default(),
        });
        state.update(cx, |s, _| {
            let target = if s.open { 1.0 } else { 0.0 };
            if s.expanded.target() != target {
                s.expanded.set(target, now);
            }
            let to = if s.open { s.nav.highlighted() } else { None };
            s.fade.sync(to, now);
        });
        let s = state.read(cx);
        let open = s.open;
        let (hover, expanded) = (s.hover.value(now), s.expanded.value(now));
        if s.hover.is_animating(now) || s.expanded.is_animating(now) || s.fade.is_animating(now) {
            window.request_animation_frame();
        }
        let trigger_focus = s.trigger_focus.clone();
        let list_focus = s.list_focus.clone();
        let anchor = s.anchor.clone();

        let theme = *cx.notra();
        let reduce = cx.reduce_motion();
        let skin = TriggerSkin::new(&theme, self.variant);
        let rest = blend(&skin.button, &skin.expanded, expanded);
        let paint = match rest.hover {
            ButtonHover::Brightness(f) => {
                let hot = ButtonSkin {
                    gradient: rest
                        .gradient
                        .map(|(t, b)| (t.brightness(f), b.brightness(f))),
                    fill: rest.fill.brightness(f),
                    border: rest.border.brightness(f),
                    ..rest.clone()
                };
                blend(&rest, &hot, hover)
            }
            ButtonHover::Fill { fill, text } => blend(
                &rest,
                &ButtonSkin {
                    fill: fill.over(rest.fill),
                    text,
                    ..rest.clone()
                },
                hover.max(expanded),
            ),
            _ => rest.clone(),
        };
        let m = self.size.metrics();
        let label = self
            .value
            .as_deref()
            .and_then(|v| label_of(&self.entries, v))
            .map(|l| SharedString::from(l.to_string()));
        let text: Hsla = if label.is_some() {
            paint.text.into()
        } else {
            skin.placeholder.into()
        };

        let menu_skin = Rc::new(SelectSkin::new(&theme));
        let entries = self.entries.clone();
        let value = self.value.clone();
        let on_change = self.on_change.clone();
        let commit: Commit = {
            let state = state.clone();
            Rc::new(
                move |command: SelectCommand, window: &mut Window, cx: &mut App| match command {
                    SelectCommand::Select(value) => {
                        state.update(cx, |s, cx| {
                            s.open = false;
                            cx.notify();
                        });
                        if let Some(f) = &on_change {
                            f(&value, window, cx);
                        }
                    }
                    SelectCommand::Close => state.update(cx, |s, cx| {
                        s.open = false;
                        cx.notify();
                    }),
                    SelectCommand::Changed | SelectCommand::None => {}
                },
            )
        };

        let popup = {
            let key_state = state.clone();
            let key_entries = entries.clone();
            let key_commit = commit.clone();
            let dismiss_state = state.clone();
            let content_state = state.clone();
            let entries = entries.clone();
            let value = value.clone();
            let commit = commit.clone();
            Popup::new((self.id.clone(), "list"), anchor.clone())
                .open(open)
                .placement(PLACEMENT)
                .collision(COLLISION)
                .motion(if reduce { PopupMotion::NONE } else { MOTION })
                .focus(list_focus)
                .on_key(move |event, window, cx| {
                    let Some(key) = select_key(event) else {
                        return false;
                    };
                    let command = key_state.update(cx, |s, cx| {
                        cx.notify();
                        s.nav.key(&key_entries, key)
                    });
                    let handled = command != SelectCommand::None
                        || !matches!(key, MenuKey::Escape | MenuKey::Tab);
                    key_commit(command, window, cx);
                    handled
                })
                .on_dismiss(move |_: Dismiss, _, cx| {
                    dismiss_state.update(cx, |s, cx| {
                        s.open = false;
                        cx.notify();
                    })
                })
                .content(move |placed, _, cx| {
                    list(
                        &content_state,
                        &entries,
                        value.as_deref(),
                        placed.rect.w,
                        &menu_skin,
                        &commit,
                        cx,
                    )
                })
        };

        let click_state = state.clone();
        let click_entries = entries.clone();
        let click_value = value.clone();
        let key_state = state.clone();
        let key_entries = entries.clone();
        let key_value = value.clone();
        let hover_state = state.clone();
        let ring = theme.ring;
        let radius = paint.radius.unwrap_or(m.radius);
        let interactive = !self.disabled;

        div()
            .id((self.id.clone(), "trigger"))
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .gap(px(m.gap))
            .h(px(m.height))
            .px(px(m.padding_x.unwrap_or(10.0)))
            .map(|d| match self.width {
                Some(w) => d.w(px(w)),
                None => d.self_start(),
            })
            .rounded(px(radius))
            .border_1()
            .border_color(paint.border)
            .bg(match paint.gradient {
                Some((top, bottom)) => gradient(top, bottom),
                None => paint.fill.into(),
            })
            .shadow(shadows(&paint.shadows))
            .text_color(text)
            .text_size(px(m.font_size))
            .line_height(px(m.line_height))
            .font_family(fonts::SANS)
            .font_weight(FontWeight(400.0))
            .whitespace_nowrap()
            .track_focus(&trigger_focus)
            .tab_stop(interactive)
            .when(self.disabled, |d| d.opacity(0.5))
            .focus_visible(move |s| {
                s.border_color(ring)
                    .shadow(shadows(&[Shadow::drop(0.0, 0.0, 3.0, ring.fade(0.5))]))
            })
            .when(interactive, |d| {
                d.cursor_pointer()
                    .on_hover(move |hovered, _, cx| {
                        hover_state.update(cx, |s, cx| {
                            s.hover
                                .set(if *hovered { 1.0 } else { 0.0 }, Instant::now());
                            cx.notify();
                        })
                    })
                    .on_click(move |_, _, cx| {
                        click_state.update(cx, |s, cx| {
                            s.open = !s.open;
                            if s.open {
                                s.nav = SelectNav::opened(
                                    &click_entries,
                                    click_value.as_deref(),
                                    false,
                                );
                            }
                            cx.notify();
                        })
                    })
                    .on_key_down(move |event: &KeyDownEvent, _, cx| {
                        if !matches!(
                            event.keystroke.key.as_str(),
                            "enter" | "space" | "down" | "up"
                        ) {
                            return;
                        }
                        cx.stop_propagation();
                        key_state.update(cx, |s, cx| {
                            s.open = true;
                            s.nav = SelectNav::opened(&key_entries, key_value.as_deref(), true);
                            cx.notify();
                        });
                    })
            })
            .child(
                div()
                    .when(self.width.is_some(), |d| {
                        d.flex_1().min_w_0().overflow_hidden().text_ellipsis()
                    })
                    .child(label.unwrap_or(self.placeholder)),
            )
            .child(
                svg()
                    .path(Icon::ChevronsUpDown.path())
                    .size(px(metrics::CHEVRON))
                    .flex_none()
                    .text_color(skin.chevron),
            )
            .child(anchor.tracker())
            .child(popup)
    }
}

fn list(
    state: &Entity<SelectState>,
    entries: &Rc<Vec<SelectEntry>>,
    value: Option<&str>,
    anchor_width: f32,
    skin: &Rc<SelectSkin>,
    commit: &Commit,
    cx: &App,
) -> AnyElement {
    let now = Instant::now();
    let s = state.read(cx);
    let fade = s.fade;
    let width = s
        .anchor
        .get()
        .map_or(anchor_width, |b| f32::from(b.size.width))
        .max(metrics::MIN_WIDTH);
    let all = options(entries);
    let mut index = 0;
    let option_row = |option_index: usize| {
        let option = all[option_index];
        let lit = fade.amount(option_index, now);
        let (_, rest_text, _) = skin.row(false, false);
        let (hot_fill, hot_text, _) = skin.row(true, false);
        let selected = value == Some(option.value.as_str());
        let hover_state = state.clone();
        let click_value = option.value.clone();
        let commit = commit.clone();
        let disabled = option.disabled;
        let text: Hsla = mix(rest_text, hot_text, lit).into();
        div()
            .id(("option", option_index))
            .relative()
            .flex()
            .items_center()
            .gap(px(row::ITEM_GAP))
            .w_full()
            .min_h(px(row::ITEM_HEIGHT))
            .pl(px(row::ITEM_PADDING_X))
            .pr(px(OPTION_PADDING_RIGHT))
            .py(px(row::ITEM_PADDING_Y))
            .rounded(px(row::ITEM_RADIUS))
            .bg(hot_fill.fade(lit))
            .when_some(skin.highlight_inset.filter(|_| lit > 0.0), |d, inset| {
                d.shadow(shadows(&[Shadow {
                    color: inset.color.fade(lit),
                    ..inset
                }]))
            })
            .text_size(px(row::ITEM_FONT))
            .line_height(px(row::ITEM_LINE))
            .text_color(text)
            .when(disabled, |d| d.opacity(row::DISABLED_OPACITY))
            .cursor_pointer()
            .on_hover(move |hovered, window, cx| {
                if *hovered {
                    hover_state.update(cx, |s, cx| {
                        s.nav.hover(option_index);
                        cx.notify();
                    });
                    window.refresh();
                }
            })
            .when(!disabled, |d| {
                d.on_click(move |_, window, cx| {
                    commit(SelectCommand::Select(click_value.clone()), window, cx)
                })
            })
            .child(SharedString::from(option.label.clone()))
            .when(selected, |d| {
                d.child(
                    div()
                        .absolute()
                        .right(px(row::INDICATOR_RIGHT))
                        .top_0()
                        .bottom_0()
                        .flex()
                        .items_center()
                        .child(
                            svg()
                                .path(Icon::Check.path())
                                .size(px(row::ICON))
                                .text_color(text),
                        ),
                )
            })
            .into_any_element()
    };
    let label = |text: &str| {
        div()
            .px(px(row::LABEL_PADDING_X))
            .py(px(row::LABEL_PADDING_Y))
            .text_size(px(row::LABEL_FONT))
            .line_height(px(row::LABEL_LINE))
            .font_weight(FontWeight(500.0))
            .text_color(skin.label)
            .child(SharedString::from(text.to_string()))
    };
    let mut children: Vec<AnyElement> = Vec::new();
    for entry in entries.iter() {
        match entry {
            SelectEntry::Group {
                label: title,
                options,
            } => {
                let mut group = div().flex().flex_col().p(px(metrics::GROUP_PADDING));
                if let Some(title) = title {
                    group = group.child(label(title));
                }
                for _ in options {
                    group = group.child(option_row(index));
                    index += 1;
                }
                children.push(group.into_any_element());
            }
            SelectEntry::Option(_) => {
                children.push(option_row(index));
                index += 1;
            }
            SelectEntry::Separator => children.push(
                div()
                    .mx(px(row::SEPARATOR_MARGIN_X))
                    .my(px(row::SEPARATOR_MARGIN_Y))
                    .h(px(1.0))
                    .bg(skin.separator)
                    .into_any_element(),
            ),
        }
    }
    let surface = &skin.surface;
    div()
        .id("select-list")
        .w(px(width))
        .max_h(px(row::MAX_HEIGHT))
        .overflow_y_scroll()
        .rounded(px(row::SURFACE_RADIUS))
        .border_1()
        .border_color(surface.border)
        .bg(gradient(
            surface.gradient.0.over(surface.fill),
            surface.gradient.1.over(surface.fill),
        ))
        .shadow(shadows(&surface.shadows))
        .text_color(surface.text)
        .font_family(fonts::SANS)
        .child(
            div()
                .flex()
                .flex_col()
                .p(px(metrics::LIST_PADDING))
                .children(children),
        )
        .into_any_element()
}
