//! shadcn's Popover and Hover Card for GPUI.
//!
//! ```ignore
//! Popover::new("dimensions")
//!     .trigger(|open, _, cx| outline_button("open", "Open popover", open, cx).into_any_element())
//!     .content(|_, cx| popover_header("Dimensions", "Set the dimensions for the layer.", cx).into_any_element())
//! ```
//!
//! GPUI can't scale an element, so the `zoom-in-95` / `zoom-out-95` part of the motion is
//! dropped; the fade and the 8 px slide play as on the web.

use std::rc::Rc;
use std::time::Instant;

use ::gpui::{
    AnyElement, App, Div, ElementId, Entity, FocusHandle, FontWeight, InteractiveElement,
    IntoElement, MouseButton, ParentElement, RenderOnce, SharedString, Styled, Task, Window, div,
    prelude::*, px,
};

use super::tooltip::arm;
use super::{AnchorBounds, Placement, Popup};
use crate::components::overlays::popover::{
    HOVER_CARD_PLACEMENT, HoverCardDelays, HoverIntent, MOTION, PLACEMENT, PopoverSkin, metrics,
};
use crate::fonts;
use crate::gpui::{ActiveTheme, shadows};

type Trigger = Box<dyn FnOnce(bool, &mut Window, &mut App) -> AnyElement>;
type Content = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;
type OpenChange = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// The popover and hover card surface: `rounded-lg bg-popover p-2.5 text-sm shadow-md ring-1`.
fn surface(width: f32, cx: &App) -> Div {
    let skin = PopoverSkin::new(cx.notra());
    div()
        .w(px(width))
        .p(px(metrics::PADDING))
        .rounded(px(metrics::RADIUS))
        .bg(skin.fill)
        .shadow(shadows(&skin.shadows))
        .text_color(skin.text)
        .text_size(px(metrics::FONT_SIZE))
        .line_height(px(metrics::LINE_HEIGHT))
        .font_family(fonts::SANS)
}

/// `PopoverTitle`: `font-medium`.
pub fn popover_title(title: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .font_weight(FontWeight(metrics::TITLE_WEIGHT))
        .text_color(PopoverSkin::new(cx.notra()).text)
        .child(title.into())
}

/// `PopoverDescription`: `text-muted-foreground`.
pub fn popover_description(description: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_color(PopoverSkin::new(cx.notra()).muted)
        .child(description.into())
}

/// `PopoverHeader` with a title and a description (`flex-col gap-0.5`).
pub fn popover_header(
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    cx: &App,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(metrics::HEADER_GAP))
        .child(popover_title(title, cx))
        .child(popover_description(description, cx))
}

struct PopoverState {
    open: bool,
    focus: FocusHandle,
}

/// A popover: pressing the trigger opens a surface below it that takes focus; Escape or a press
/// outside closes it and focus returns. Uncontrolled unless [`Popover::open`] is set.
#[derive(IntoElement)]
pub struct Popover {
    id: ElementId,
    trigger: Option<Trigger>,
    content: Option<Content>,
    width: f32,
    placement: Placement,
    open: Option<bool>,
    on_open_change: Option<OpenChange>,
}

impl Popover {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            content: None,
            width: metrics::POPOVER_WIDTH,
            placement: PLACEMENT,
            open: None,
            on_open_change: None,
        }
    }

    /// The trigger, built with whether the popover is open (`aria-expanded`).
    pub fn trigger(
        mut self,
        trigger: impl FnOnce(bool, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.trigger = Some(Box::new(trigger));
        self
    }

    /// The surface's children, laid out `flex-col gap-2.5`.
    pub fn content(
        mut self,
        content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(Rc::new(content));
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// Controls the open state; pair it with [`Popover::on_open_change`].
    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(handler));
        self
    }
}

fn set_open(
    state: &Entity<PopoverState>,
    open: bool,
    handler: &Option<OpenChange>,
    window: &mut Window,
    cx: &mut App,
) {
    state.update(cx, |s, cx| {
        s.open = open;
        cx.notify();
    });
    if let Some(handler) = handler {
        handler(open, window, cx);
    }
}

impl RenderOnce for Popover {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| PopoverState {
            open: false,
            focus: cx.focus_handle(),
        });
        let anchor = window
            .use_keyed_state((self.id.clone(), "anchor"), cx, |_, _| {
                AnchorBounds::default()
            })
            .read(cx)
            .clone();
        if let Some(open) = self.open {
            state.update(cx, |s, _| s.open = open);
        }
        let (open, focus) = {
            let s = state.read(cx);
            (s.open, s.focus.clone())
        };
        let trigger = self
            .trigger
            .map(|trigger| trigger(open, window, cx))
            .unwrap_or_else(|| div().into_any_element());
        let content = self.content;
        let width = self.width;
        let (toggle_state, dismiss_state) = (state.clone(), state.clone());
        let (toggle_change, dismiss_change) = (self.on_open_change.clone(), self.on_open_change);

        div()
            .id(self.id.clone())
            .relative()
            .flex_none()
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                let open = !toggle_state.read(cx).open;
                set_open(&toggle_state, open, &toggle_change, window, cx);
            })
            .child(trigger)
            .child(anchor.tracker())
            .child(
                Popup::new((self.id, "popup"), anchor)
                    .open(open)
                    .placement(self.placement)
                    .motion(MOTION)
                    .focus(focus)
                    .on_dismiss(move |_, window, cx| {
                        set_open(&dismiss_state, false, &dismiss_change, window, cx);
                    })
                    .content(move |_, window, cx| {
                        surface(width, cx)
                            .flex()
                            .flex_col()
                            .gap(px(metrics::GAP))
                            .when_some(content, |d, content| d.child(content(window, cx)))
                            .into_any_element()
                    }),
            )
    }
}

struct HoverCardState {
    intent: HoverIntent,
    timer: Option<Task<()>>,
}

fn arm_intent(state: &Entity<HoverCardState>, cx: &mut App) {
    let deadline = state.read(cx).intent.deadline();
    arm(
        state,
        deadline,
        |s| &mut s.timer,
        |s, now| s.intent.tick(now),
        |s| s.intent.deadline(),
        cx,
    );
}

/// A hover card: resting on the trigger for 600 ms opens a card below it, which stays open
/// while the pointer is over the trigger or the card and closes 300 ms after it leaves both.
#[derive(IntoElement)]
pub struct HoverCard {
    id: ElementId,
    trigger: AnyElement,
    content: Option<Content>,
    width: f32,
    placement: Placement,
    delays: HoverCardDelays,
}

impl HoverCard {
    pub fn new(id: impl Into<ElementId>, trigger: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            content: None,
            width: metrics::HOVER_CARD_WIDTH,
            placement: HOVER_CARD_PLACEMENT,
            delays: HoverCardDelays::BASE_UI,
        }
    }

    pub fn content(
        mut self,
        content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(Rc::new(content));
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// Base UI's `delay` and `closeDelay`.
    pub fn delays(mut self, delays: HoverCardDelays) -> Self {
        self.delays = delays;
        self
    }
}

impl RenderOnce for HoverCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let delays = self.delays;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| HoverCardState {
            intent: HoverIntent::new(delays),
            timer: None,
        });
        let anchor = window
            .use_keyed_state((self.id.clone(), "anchor"), cx, |_, _| {
                AnchorBounds::default()
            })
            .read(cx)
            .clone();
        let open = state.read(cx).intent.is_open();
        let content = self.content;
        let width = self.width;
        let (trigger_state, card_state, dismiss_state) =
            (state.clone(), state.clone(), state.clone());
        let card_id: ElementId = (self.id.clone(), "card").into();

        div()
            .id(self.id.clone())
            .relative()
            .flex_none()
            .on_hover(move |hovered, _, cx| {
                trigger_state.update(cx, |s, cx| {
                    if s.intent.set_trigger(*hovered, Instant::now()) {
                        cx.notify();
                    }
                });
                arm_intent(&trigger_state, cx);
            })
            .child(self.trigger)
            .child(anchor.tracker())
            .child(
                Popup::new((self.id, "popup"), anchor)
                    .open(open)
                    .placement(self.placement)
                    .motion(MOTION)
                    .on_dismiss(move |_, _, cx| {
                        dismiss_state.update(cx, |s, cx| {
                            if s.intent.close() {
                                cx.notify();
                            }
                        });
                    })
                    .content(move |_, window, cx| {
                        surface(width, cx)
                            .id(card_id)
                            .on_hover(move |hovered, _, cx| {
                                card_state.update(cx, |s, cx| {
                                    if s.intent.set_card(*hovered, Instant::now()) {
                                        cx.notify();
                                    }
                                });
                                arm_intent(&card_state, cx);
                            })
                            .when_some(content, |d, content| d.child(content(window, cx)))
                            .into_any_element()
                    }),
            )
    }
}
