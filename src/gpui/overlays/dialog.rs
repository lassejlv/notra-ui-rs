use std::rc::Rc;
use std::time::Instant;

use ::gpui::{
    AnyElement, App, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, RenderOnce, SharedString, Styled, Window, anchored, deferred, div,
    point, prelude::*, px,
};

use super::{PopupMotion, Presence, Side};
use crate::components::button::{ButtonSize, ButtonVariant};
use crate::components::overlays::dialog::{
    DIALOG_MOTION, ModalSkin, SHEET_MOTION, dialog_width, metrics, sheet_offset, sheet_rect,
};
use crate::fonts;
use crate::gpui::{ActiveTheme, Button, shadows};
use crate::icons::Icon;
use crate::theme::Shadow;

type CloseHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// What a modal surface shares: its open state, focus, and the handler that closes it.
struct ModalState {
    presence: Presence,
    focus: FocusHandle,
    restore: Option<FocusHandle>,
    height: f32,
}

/// Syncs a modal's presence, moves focus in when it opens and back when it closes, and
/// returns whether it still draws.
fn sync_modal(
    id: &ElementId,
    open: bool,
    motion: &PopupMotion,
    window: &mut Window,
    cx: &mut App,
) -> (::gpui::Entity<ModalState>, bool) {
    let now = Instant::now();
    let state = window.use_keyed_state(id.clone(), cx, |_, cx| ModalState {
        presence: Presence::new(false),
        focus: cx.focus_handle(),
        restore: None,
        height: 0.0,
    });
    let (opened, closed, focus) = state.update(cx, |s, _| {
        let was = s.presence.is_open();
        s.presence.set(open, motion, now);
        (!was && open, was && !open, s.focus.clone())
    });
    if opened {
        let previous = window.focused(cx);
        state.update(cx, |s, _| s.restore = previous);
        window.defer(cx, move |window, cx| focus.focus(window, cx));
    } else if closed {
        let restore = state.update(cx, |s, _| s.restore.take());
        if let Some(previous) = restore {
            window.defer(cx, move |window, cx| previous.focus(window, cx));
        }
    }
    let s = state.read(cx);
    let visible = s.presence.is_visible(motion, now);
    if s.presence.is_animating(motion, now) {
        window.request_animation_frame();
    }
    (state, visible)
}

/// Escape closes; Tab and Shift-Tab stay inside the modal.
fn trap_keys(
    focus: FocusHandle,
    close: Option<CloseHandler>,
) -> impl Fn(&KeyDownEvent, &mut Window, &mut App) {
    move |event, window, cx| match event.keystroke.key.as_str() {
        "escape" => {
            cx.stop_propagation();
            if let Some(close) = &close {
                close(window, cx);
            }
        }
        "tab" => {
            cx.stop_propagation();
            if event.keystroke.modifiers.shift {
                window.focus_prev(cx);
            } else {
                window.focus_next(cx);
            }
            if !focus.contains_focused(window, cx) {
                focus.focus(window, cx);
            }
        }
        _ => {}
    }
}

fn close_button(id: impl Into<ElementId>, close: Option<CloseHandler>) -> Button {
    Button::icon_button(id, Icon::X)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::IconSm)
        .when_some(close, |b, close| {
            b.on_click(move |_, window, cx| close(window, cx))
        })
}

/// A modal dialog over a dimmed, blurred backdrop: title, description, body and an optional
/// footer band. Escape, the close button and a press on the backdrop ask to close; focus
/// moves in when it opens, stays inside while open and returns to the opener.
///
/// ```ignore
/// Dialog::new("rename", self.renaming)
///     .title("Rename workflow")
///     .description("Pick a name your team will recognise.")
///     .child(field)
///     .footer(Button::new("save", "Save").on_click(..))
///     .on_close(cx.listener(|this, _, cx| { this.renaming = false; cx.notify(); }))
/// ```
#[derive(IntoElement)]
pub struct Dialog {
    id: ElementId,
    open: bool,
    title: Option<SharedString>,
    description: Option<SharedString>,
    children: Vec<AnyElement>,
    footer: Vec<AnyElement>,
    width: f32,
    close_button: bool,
    on_close: Option<CloseHandler>,
}

impl Dialog {
    pub fn new(id: impl Into<ElementId>, open: bool) -> Self {
        Self {
            id: id.into(),
            open,
            title: None,
            description: None,
            children: Vec::new(),
            footer: Vec::new(),
            width: metrics::DIALOG_WIDTH,
            close_button: true,
            on_close: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// A button or other element in the footer band (`DialogFooter`), end-aligned.
    pub fn footer(mut self, element: impl IntoElement) -> Self {
        self.footer.push(element.into_any_element());
        self
    }

    /// The widest the dialog gets (`sm:max-w-sm`, 384, by default; blocks use `max-w-xl`).
    pub fn max_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// The X at the top right (`showCloseButton`), on by default.
    pub fn close_button(mut self, on: bool) -> Self {
        self.close_button = on;
        self
    }

    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }
}

impl ParentElement for Dialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Dialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = if cx.reduce_motion() {
            DIALOG_MOTION.reduced()
        } else {
            DIALOG_MOTION
        };
        let (state, visible) = sync_modal(&self.id, self.open, &motion, window, cx);
        if !visible {
            return div().absolute().into_any_element();
        }
        let now = Instant::now();
        let s = state.read(cx);
        let shown = s.presence.shown(&motion, now);
        let focus = s.focus.clone();
        let theme = *cx.notra();
        let skin = ModalSkin::new(&theme);
        let viewport = window.viewport_size();
        let (vw, vh) = (f32::from(viewport.width), f32::from(viewport.height));
        let width = dialog_width(vw, self.width);
        let height = s.height;
        let scale = motion
            .frame(shown, s.presence.is_open(), Side::Bottom)
            .scale;
        let backdrop_close = self.on_close.clone();

        let header = (self.title.is_some() || self.description.is_some()).then(|| {
            div()
                .flex()
                .flex_col()
                .gap(px(metrics::HEADER_GAP))
                .when_some(self.title.clone(), |d, title| {
                    d.child(
                        div()
                            .pr(px(32.0))
                            .text_size(px(metrics::TITLE_FONT))
                            .line_height(px(metrics::DIALOG_TITLE_LINE))
                            .font_weight(FontWeight(500.0))
                            .child(title),
                    )
                })
                .when_some(self.description.clone(), |d, text| {
                    d.child(div().text_color(skin.description).child(text))
                })
        });
        let footer = (!self.footer.is_empty()).then(|| {
            div()
                .mx(px(-metrics::DIALOG_PADDING))
                .mb(px(-metrics::DIALOG_PADDING))
                .flex()
                .justify_end()
                .gap(px(metrics::FOOTER_GAP))
                .p(px(metrics::DIALOG_PADDING))
                .border_t_1()
                .border_color(skin.footer_border)
                .bg(skin.footer)
                .rounded_b(px(metrics::DIALOG_RADIUS))
                .children(self.footer)
        });
        let measure = state.clone();
        let content = div()
            .on_children_prepainted(move |bounds, _, cx| {
                if let Some(b) = bounds.first() {
                    let h = f32::from(b.size.height);
                    measure.update(cx, |s, cx| {
                        if (s.height - h).abs() > 0.5 {
                            s.height = h;
                            cx.notify();
                        }
                    });
                }
            })
            .child(
                div()
                    .id((self.id.clone(), "content"))
                    .track_focus(&focus)
                    .on_key_down(trap_keys(focus.clone(), self.on_close.clone()))
                    .on_mouse_down(::gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .relative()
                    .w(px(width * scale))
                    .max_h(px(vh - 2.0 * metrics::DIALOG_MARGIN))
                    .flex()
                    .flex_col()
                    .gap(px(metrics::DIALOG_GAP))
                    .p(px(metrics::DIALOG_PADDING))
                    .rounded(px(metrics::DIALOG_RADIUS))
                    .bg(skin.surface)
                    .shadow(shadows(&[Shadow::drop(0.0, 0.0, 1.0, skin.ring)]))
                    .text_color(skin.text)
                    .text_size(px(metrics::BODY_FONT))
                    .line_height(px(metrics::BODY_LINE))
                    .font_family(fonts::SANS)
                    .children(header)
                    .children(self.children)
                    .children(footer)
                    .when(self.close_button, |d| {
                        d.child(
                            div()
                                .absolute()
                                .top(px(metrics::DIALOG_CLOSE_INSET))
                                .right(px(metrics::DIALOG_CLOSE_INSET))
                                .child(close_button(
                                    (self.id.clone(), "close"),
                                    self.on_close.clone(),
                                )),
                        )
                    }),
            );

        deferred(
            anchored().position(point(px(0.0), px(0.0))).child(
                div()
                    .id((self.id.clone(), "backdrop"))
                    .occlude()
                    .w(px(vw))
                    .h(px(vh))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(skin.backdrop.fade(shown))
                    .when_some(backdrop_close, |d, close| {
                        d.on_mouse_down(::gpui::MouseButton::Left, move |_, window, cx| {
                            close(window, cx)
                        })
                    })
                    .child(
                        div()
                            .opacity(if height > 0.0 { shown } else { 0.0 })
                            .child(content),
                    ),
            ),
        )
        .with_priority(10)
        .into_any_element()
    }
}

/// A side panel over the dimmed, blurred backdrop, sliding in from `side`: header, body and
/// a footer pinned to its bottom. Modal like [`Dialog`].
#[derive(IntoElement)]
pub struct Sheet {
    id: ElementId,
    open: bool,
    side: Side,
    title: Option<SharedString>,
    description: Option<SharedString>,
    children: Vec<AnyElement>,
    footer: Vec<AnyElement>,
    close_button: bool,
    on_close: Option<CloseHandler>,
}

impl Sheet {
    pub fn new(id: impl Into<ElementId>, open: bool) -> Self {
        Self {
            id: id.into(),
            open,
            side: Side::Right,
            title: None,
            description: None,
            children: Vec::new(),
            footer: Vec::new(),
            close_button: true,
            on_close: None,
        }
    }

    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// An element in the footer (`SheetFooter`, pinned to the bottom).
    pub fn footer(mut self, element: impl IntoElement) -> Self {
        self.footer.push(element.into_any_element());
        self
    }

    pub fn close_button(mut self, on: bool) -> Self {
        self.close_button = on;
        self
    }

    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }
}

impl ParentElement for Sheet {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Sheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = if cx.reduce_motion() {
            SHEET_MOTION.reduced()
        } else {
            SHEET_MOTION
        };
        let (state, visible) = sync_modal(&self.id, self.open, &motion, window, cx);
        if !visible {
            return div().absolute().into_any_element();
        }
        let now = Instant::now();
        let s = state.read(cx);
        let shown = s.presence.shown(&motion, now);
        let focus = s.focus.clone();
        let theme = *cx.notra();
        let skin = ModalSkin::new(&theme);
        let viewport = window.viewport_size();
        let (vw, vh) = (f32::from(viewport.width), f32::from(viewport.height));
        let rect = sheet_rect(self.side, (vw, vh), s.height);
        let (dx, dy) = if motion.enter_travel > 0.0 {
            sheet_offset(self.side, shown)
        } else {
            (0.0, 0.0)
        };
        let backdrop_close = self.on_close.clone();
        let edge = |d: ::gpui::Stateful<::gpui::Div>| match self.side {
            Side::Right => d.border_l_1(),
            Side::Left => d.border_r_1(),
            Side::Top => d.border_b_1(),
            Side::Bottom => d.border_t_1(),
        };
        let measure = state.clone();
        let vertical = matches!(self.side, Side::Top | Side::Bottom);

        let header = (self.title.is_some() || self.description.is_some()).then(|| {
            div()
                .flex()
                .flex_col()
                .gap(px(metrics::SHEET_HEADER_GAP))
                .p(px(metrics::SHEET_PADDING))
                .when_some(self.title.clone(), |d, title| {
                    d.child(
                        div()
                            .text_size(px(metrics::TITLE_FONT))
                            .line_height(px(metrics::SHEET_TITLE_LINE))
                            .font_weight(FontWeight(500.0))
                            .text_color(theme.foreground)
                            .child(title),
                    )
                })
                .when_some(self.description.clone(), |d, text| {
                    d.child(div().text_color(skin.description).child(text))
                })
        });
        let footer = (!self.footer.is_empty()).then(|| {
            div()
                .mt_auto()
                .flex()
                .flex_col()
                .gap(px(metrics::FOOTER_GAP))
                .p(px(metrics::SHEET_PADDING))
                .children(self.footer)
        });

        let panel = edge(
            div()
                .id((self.id.clone(), "panel"))
                .track_focus(&focus)
                .on_key_down(trap_keys(focus.clone(), self.on_close.clone()))
                .on_mouse_down(::gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation()),
        )
        .absolute()
        .left(px(rect.x + dx))
        .top(px(rect.y + dy))
        .w(px(rect.w))
        .when(!vertical, |d| d.h(px(rect.h)))
        .opacity(shown)
        .flex()
        .flex_col()
        .gap(px(metrics::SHEET_GAP))
        .bg(skin.surface)
        .border_color(skin.border)
        .shadow(shadows(&skin.sheet_shadows))
        .text_color(skin.text)
        .text_size(px(metrics::BODY_FONT))
        .line_height(px(metrics::BODY_LINE))
        .font_family(fonts::SANS)
        .children(header)
        .children(self.children)
        .children(footer)
        .when(self.close_button, |d| {
            d.child(
                div()
                    .absolute()
                    .top(px(metrics::SHEET_CLOSE_INSET))
                    .right(px(metrics::SHEET_CLOSE_INSET))
                    .child(close_button(
                        (self.id.clone(), "close"),
                        self.on_close.clone(),
                    )),
            )
        });

        deferred(
            anchored().position(point(px(0.0), px(0.0))).child(
                div()
                    .id((self.id.clone(), "backdrop"))
                    .occlude()
                    .relative()
                    .w(px(vw))
                    .h(px(vh))
                    .bg(skin.backdrop.fade(shown))
                    .when_some(backdrop_close, |d, close| {
                        d.on_mouse_down(::gpui::MouseButton::Left, move |_, window, cx| {
                            close(window, cx)
                        })
                    })
                    .child(
                        div()
                            .on_children_prepainted(move |bounds, _, cx| {
                                if let Some(b) = bounds.first() {
                                    let h = f32::from(b.size.height);
                                    measure.update(cx, |s, cx| {
                                        if vertical && (s.height - h).abs() > 0.5 {
                                            s.height = h;
                                            cx.notify();
                                        }
                                    });
                                }
                            })
                            .child(panel),
                    ),
            ),
        )
        .with_priority(10)
        .into_any_element()
    }
}
