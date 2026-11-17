//! Notra's tooltip and the duotone tooltip for GPUI.
//!
//! A [`Tooltip`] wraps a trigger element. On its own it runs Base UI's default timing (600 ms
//! to open). Inside a [`TooltipProvider`] (Notra's `TooltipProvider`), every trigger shares one
//! popup that the app places once with [`TooltipProvider::popup`]: hovering a neighbour while it
//! is open glides it there over 200 ms, and its content swaps.
//!
//! ```ignore
//! let provider = TooltipProvider::new("toolbar", TooltipDelays::PROVIDER.with_open(ms(400)), window, cx);
//! div()
//!     .child(Tooltip::new("bold", bold_button, "Bold").keys(&["⌘", "B"]).provider(&provider))
//!     .child(provider.popup())
//! ```
//!
//! GPUI can't scale an element, so the popups' `scale-[0.97]` is dropped; opacity and the 2 px
//! slide play as on the web.

use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use ::gpui::{
    AnyElement, App, Bounds, DispatchPhase, ElementId, Entity, FocusHandle, FontWeight,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, PathBuilder, Pixels,
    RenderOnce, SharedString, Styled, Task, Window, canvas, div, point, prelude::*, px,
};

use super::{AnchorBounds, Collision, Placed, Placement, Popup, PopupMotion, Side, rect};
use crate::components::overlays::popover::{HoverCardDelays, HoverIntent};
use crate::components::overlays::tooltip::{
    GLIDE, MOTION, PLACEMENT, TooltipGroup, TooltipSkin, arrow_center, duotone, kbd, metrics,
};
use crate::fonts;
use crate::gpui::{ActiveTheme, gradient, shadows};
use crate::motion::Animated;

pub use crate::components::overlays::tooltip::TooltipDelays;

type Content = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// Runs `tick` on `entity` when `deadline` passes, replacing the timer `slot` holds.
pub(super) fn arm<T: 'static>(
    entity: &Entity<T>,
    deadline: Option<Instant>,
    slot: fn(&mut T) -> &mut Option<Task<()>>,
    tick: fn(&mut T, Instant) -> bool,
    deadline_of: fn(&T) -> Option<Instant>,
    cx: &mut App,
) {
    let Some(at) = deadline else {
        entity.update(cx, |s, _| *slot(s) = None);
        return;
    };
    let weak = entity.downgrade();
    let task = cx.spawn(async move |cx| {
        let wait = at.saturating_duration_since(Instant::now());
        cx.background_executor().timer(wait).await;
        let Some(entity) = weak.upgrade() else {
            return;
        };
        let next = entity.update(cx, |s, cx| {
            if tick(s, Instant::now()) {
                cx.notify();
            }
            deadline_of(s)
        });
        if next.is_some() {
            cx.update(|cx| arm(&entity, next, slot, tick, deadline_of, cx));
        }
    });
    entity.update(cx, |s, _| *slot(s) = Some(task));
}

/// What a trigger shows in the shared popup.
#[derive(Clone)]
struct Body {
    label: SharedString,
    keys: Vec<SharedString>,
    content: Option<Content>,
    placement: Placement,
}

/// The state behind a [`TooltipProvider`].
struct ProviderState {
    group: TooltipGroup<SharedString>,
    anchors: HashMap<SharedString, AnchorBounds>,
    bodies: HashMap<SharedString, Body>,
    focused: Option<SharedString>,
    timer: Option<Task<()>>,
    glide: Option<[Animated; 4]>,
    shown: Option<SharedString>,
    anchor: AnchorBounds,
}

impl ProviderState {
    fn new(delays: TooltipDelays) -> Self {
        Self {
            group: TooltipGroup::new(delays),
            anchors: HashMap::new(),
            bodies: HashMap::new(),
            focused: None,
            timer: None,
            glide: None,
            shown: None,
            anchor: AnchorBounds::default(),
        }
    }
}

fn arm_group(state: &Entity<ProviderState>, cx: &mut App) {
    let deadline = state.read(cx).group.deadline();
    arm(
        state,
        deadline,
        |s| &mut s.timer,
        |s, now| s.group.tick(now),
        |s| s.group.deadline(),
        cx,
    );
}

/// A group of tooltips sharing one popup and one delay (Notra's `TooltipProvider`).
#[derive(Clone)]
pub struct TooltipProvider {
    id: ElementId,
    state: Entity<ProviderState>,
}

impl TooltipProvider {
    /// The provider keyed `id` in this window, with `delays` (`TooltipDelays::PROVIDER` is
    /// Notra's default: open at once).
    pub fn new(
        id: impl Into<ElementId>,
        delays: TooltipDelays,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let id = id.into();
        let state = window.use_keyed_state(id.clone(), cx, |_, _| ProviderState::new(delays));
        Self { id, state }
    }

    /// Whether the popup is open on the trigger keyed `id`.
    pub fn is_open_on(&self, id: &str, cx: &App) -> bool {
        self.state
            .read(cx)
            .group
            .is_open_on(&SharedString::from(id.to_owned()))
    }

    /// The shared popup. Place it once, after the provider's triggers.
    pub fn popup(&self) -> TooltipPopup {
        TooltipPopup {
            provider: self.clone(),
        }
    }
}

/// A trigger with a tooltip: `Tooltip::new("bold", button, "Bold").keys(&["⌘", "B"])`.
#[derive(IntoElement)]
pub struct Tooltip {
    id: SharedString,
    trigger: AnyElement,
    body: Body,
    provider: Option<TooltipProvider>,
    focus: Option<FocusHandle>,
}

impl Tooltip {
    /// `trigger` showing `label` above it. `id` names the trigger within its provider.
    pub fn new(
        id: impl Into<SharedString>,
        trigger: impl IntoElement,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            body: Body {
                label: label.into(),
                keys: Vec::new(),
                content: None,
                placement: PLACEMENT,
            },
            provider: None,
            focus: None,
        }
    }

    /// Key caps after the label (a `KbdGroup`).
    pub fn keys(mut self, keys: &[&str]) -> Self {
        self.body.keys = keys
            .iter()
            .map(|k| SharedString::from((*k).to_owned()))
            .collect();
        self
    }

    /// Custom content in place of the label.
    pub fn content(
        mut self,
        content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.body.content = Some(Rc::new(content));
        self
    }

    /// The side of the trigger it opens on (`top` by default).
    pub fn side(mut self, side: Side) -> Self {
        self.body.placement.side = side;
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.body.placement = placement;
        self
    }

    /// Share `provider`'s popup and delay.
    pub fn provider(mut self, provider: &TooltipProvider) -> Self {
        self.provider = Some(provider.clone());
        self
    }

    /// The trigger's focus handle: keyboard focus opens the tooltip at once.
    pub fn focus(mut self, handle: FocusHandle) -> Self {
        self.focus = Some(handle);
        self
    }
}

impl RenderOnce for Tooltip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let standalone = self.provider.is_none();
        let provider = self.provider.unwrap_or_else(|| {
            TooltipProvider::new(
                ElementId::Name(format!("{id}-tooltip").into()),
                TooltipDelays::BASE_UI,
                window,
                cx,
            )
        });
        let state = provider.state.clone();
        let anchor = state.update(cx, |s, _| {
            s.bodies.insert(id.clone(), self.body);
            s.anchors.entry(id.clone()).or_default().clone()
        });

        if let Some(handle) = &self.focus {
            let focused = handle.is_focused(window) && window.last_input_was_keyboard();
            let now = Instant::now();
            state.update(cx, |s, cx| {
                let was = s.focused.as_ref() == Some(&id);
                if focused && !was {
                    s.focused = Some(id.clone());
                    if s.group.focus(id.clone()) {
                        cx.notify();
                    }
                } else if !focused && was {
                    s.focused = None;
                    if s.group.blur(&id, now) {
                        cx.notify();
                    }
                }
            });
        }

        let hover_state = state.clone();
        let hover_id = id.clone();
        let press_state = state.clone();
        div()
            .id(ElementId::Name(format!("{id}-trigger").into()))
            .relative()
            .flex_none()
            .child(self.trigger)
            .child(anchor.tracker())
            .on_hover(move |hovered, _, cx| {
                let now = Instant::now();
                let changed = hover_state.update(cx, |s, _| {
                    if *hovered {
                        s.group.hover(hover_id.clone(), now)
                    } else {
                        s.group.unhover(&hover_id, now)
                    }
                });
                if changed {
                    hover_state.update(cx, |_, cx| cx.notify());
                }
                arm_group(&hover_state, cx);
            })
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                press_state.update(cx, |s, cx| {
                    if s.group.dismiss(Instant::now()) {
                        cx.notify();
                    }
                });
            })
            .when(standalone, |d| d.child(provider.popup()))
    }
}

/// The popup of a [`TooltipProvider`], placed once after its triggers.
#[derive(IntoElement)]
pub struct TooltipPopup {
    provider: TooltipProvider,
}

impl RenderOnce for TooltipPopup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.provider.state.clone();
        let now = Instant::now();
        let s = state.read(cx);
        let Some(active) = s.group.active().cloned() else {
            return div().into_any_element();
        };
        let (Some(body), Some(target)) = (
            s.bodies.get(&active).cloned(),
            s.anchors.get(&active).and_then(AnchorBounds::get),
        ) else {
            return div().into_any_element();
        };
        let open = s.group.is_open();
        let instant = s.group.is_instant();
        let reduce = cx.reduce_motion();
        let still_shown = open
            || s.group
                .closed_at()
                .is_some_and(|at| now.saturating_duration_since(at) < MOTION.exit.duration);

        let target = rect(target);
        let glide_anchor = state.update(cx, |s, _| {
            let values = [target.x, target.y, target.w, target.h];
            let moved = s.shown.as_ref() != Some(&active);
            match &mut s.glide {
                Some(glide) if moved && still_shown && !reduce => {
                    for (a, v) in glide.iter_mut().zip(values) {
                        a.set(v, now);
                    }
                }
                Some(glide) if moved || glide.iter().all(|a| !a.is_animating(now)) => {
                    for (a, v) in glide.iter_mut().zip(values) {
                        a.snap(v);
                    }
                }
                Some(glide) => {
                    for (a, v) in glide.iter_mut().zip(values) {
                        if a.target() != v {
                            a.set(v, now);
                        }
                    }
                }
                None => s.glide = Some(values.map(|v| Animated::new(v, GLIDE))),
            }
            s.shown = Some(active.clone());
            let [x, y, w, h] = s
                .glide
                .as_ref()
                .map_or(values, |g| g.each_ref().map(|a| a.value(now)));
            s.anchor.set(Some(Bounds::new(
                point(px(x), px(y)),
                ::gpui::size(px(w), px(h)),
            )));
            if s.glide
                .as_ref()
                .is_some_and(|g| g.iter().any(|a| a.is_animating(now)))
            {
                window.request_animation_frame();
            }
            s.anchor.clone()
        });

        let escape_state = state.clone();
        let theme = *cx.notra();
        Popup::new(
            ElementId::Name(format!("{}-popup", self.provider.id).into()),
            glide_anchor,
        )
        .open(open)
        .instant(instant)
        .placement(body.placement)
        .motion(MOTION)
        .priority(3)
        .content(move |placed, window, cx| {
            let skin = TooltipSkin::new(&theme);
            let placed = *placed;
            let has_keys = !body.keys.is_empty();
            let inner = match &body.content {
                Some(content) => content(window, cx),
                None => div()
                    .flex()
                    .items_center()
                    .child(body.label.clone())
                    .when(has_keys, |d| {
                        d.child(
                            div()
                                .flex()
                                .items_center()
                                .ml(px(kbd::MARGIN_START))
                                .gap(px(kbd::GAP))
                                .children(body.keys.iter().map(|k| Kbd::new(k.clone()))),
                        )
                    })
                    .into_any_element(),
            };
            let arrow_fill = skin.arrow;
            let arrow = canvas(
                move |_, _, _| {},
                move |bounds, _, window, cx| {
                    paint_arrow(&placed, bounds, arrow_fill, window);
                    if open {
                        let state = escape_state.clone();
                        window.on_key_event(move |e: &KeyDownEvent, phase, _, cx| {
                            if phase == DispatchPhase::Bubble && e.keystroke.key == "escape" {
                                state.update(cx, |s, cx| {
                                    if s.group.dismiss(Instant::now()) {
                                        cx.notify();
                                    }
                                });
                            }
                        });
                    }
                    let _ = cx;
                },
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full();
            div()
                .relative()
                .max_w(px(metrics::MAX_WIDTH))
                .pl(px(metrics::PADDING_X))
                .pr(px(if has_keys {
                    metrics::PADDING_END_WITH_KBD
                } else {
                    metrics::PADDING_X
                }))
                .py(px(metrics::PADDING_Y))
                .rounded(px(metrics::CORNER))
                .border_1()
                .border_color(skin.border)
                .bg(gradient(skin.gradient.0, skin.gradient.1))
                .shadow(shadows(&skin.shadows))
                .text_color(skin.text)
                .text_size(px(metrics::FONT_SIZE))
                .line_height(px(metrics::LINE_HEIGHT))
                .font_family(fonts::SANS)
                .font_weight(FontWeight(metrics::FONT_WEIGHT))
                .child(arrow)
                .child(inner)
                .into_any_element()
        })
        .into_any_element()
    }
}

/// Paints the arrow: a 10 px square with 2 px corners turned 45°, centred at
/// [`arrow_center`] of `placed`, inside `bounds` (the popup's padding box).
fn paint_arrow(
    placed: &Placed,
    bounds: Bounds<Pixels>,
    fill: crate::theme::Color,
    window: &mut Window,
) {
    let inset_x = (placed.rect.w - f32::from(bounds.size.width)) / 2.0;
    let inset_y = (placed.rect.h - f32::from(bounds.size.height)) / 2.0;
    let (cx, cy) = arrow_center(placed);
    let (ox, oy) = (
        f32::from(bounds.origin.x) - inset_x + cx,
        f32::from(bounds.origin.y) - inset_y + cy,
    );
    let half = metrics::ARROW_SIZE / 2.0;
    let r = metrics::ARROW_RADIUS;
    let turn = |x: f32, y: f32| {
        let k = std::f32::consts::FRAC_1_SQRT_2;
        point(px(ox + (x - y) * k), px(oy + (x + y) * k))
    };
    let (h, r) = (half, r);
    let corners = [
        ((h - r, -h), (h, -h), (h, -h + r)),
        ((h, h - r), (h, h), (h - r, h)),
        ((-h + r, h), (-h, h), (-h, h - r)),
        ((-h, -h + r), (-h, -h), (-h + r, -h)),
    ];
    let mut path = PathBuilder::fill();
    path.move_to(turn(-h + r, -h));
    for ((lx, ly), (cx, cy), (tx, ty)) in corners {
        path.line_to(turn(lx, ly));
        path.curve_to(turn(tx, ty), turn(cx, cy));
    }
    path.close();
    if let Ok(path) = path.build() {
        window.paint_path(path, fill);
    }
}

/// A key cap inside a tooltip (shadcn's `Kbd`).
#[derive(IntoElement)]
pub struct Kbd {
    label: SharedString,
}

impl Kbd {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl RenderOnce for Kbd {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let skin = TooltipSkin::new(cx.notra());
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .h(px(kbd::HEIGHT))
            .min_w(px(kbd::MIN_WIDTH))
            .px(px(kbd::PADDING_X))
            .rounded(px(kbd::RADIUS))
            .bg(skin.kbd_fill)
            .text_color(skin.kbd_text)
            .text_size(px(kbd::FONT_SIZE))
            .line_height(px(kbd::LINE_HEIGHT))
            .font_weight(FontWeight(kbd::FONT_WEIGHT))
            .child(self.label)
    }
}

/// The duotone tooltip's enter and exit: opacity and a 4 px drop, 150 ms either way.
const DUOTONE_MOTION: PopupMotion = PopupMotion {
    enter: duotone::MOTION,
    exit: duotone::MOTION,
    scale: 1.0,
    enter_travel: -duotone::TRAVEL,
    exit_travel: -duotone::TRAVEL,
};

struct DuotoneState {
    intent: HoverIntent,
    focus: FocusHandle,
    focused: bool,
}

/// The duotone tooltip: a trigger button and, below it while hovered or focused, a card with a
/// shell header (title and an optional aside) over a content card in a 2 px rim.
#[derive(IntoElement)]
pub struct DuotoneTooltip {
    id: SharedString,
    label: SharedString,
    title: SharedString,
    aside: Option<SharedString>,
    body: SharedString,
}

impl DuotoneTooltip {
    /// A trigger labelled `label` whose card shows `title` over `body`.
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        title: impl Into<SharedString>,
        body: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            title: title.into(),
            aside: None,
            body: body.into(),
        }
    }

    /// Text at the header's end (`text-xs tabular-nums`).
    pub fn aside(mut self, aside: impl Into<SharedString>) -> Self {
        self.aside = Some(aside.into());
        self
    }
}

impl RenderOnce for DuotoneTooltip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use duotone::trigger as m;
        let theme = *cx.notra();
        let skin = duotone::DuotoneSkin::new(&theme);
        let id = self.id;
        let state = window.use_keyed_state(ElementId::Name(id.clone()), cx, |_, cx| DuotoneState {
            intent: HoverIntent::new(HoverCardDelays::NONE),
            focus: cx.focus_handle(),
            focused: false,
        });
        let anchor = window
            .use_keyed_state(
                ElementId::Name(format!("{id}-anchor").into()),
                cx,
                |_, _| AnchorBounds::default(),
            )
            .read(cx)
            .clone();
        let focus = state.read(cx).focus.clone();
        let focused = focus.contains_focused(window, cx);
        state.update(cx, |s, cx| {
            if s.focused != focused {
                s.focused = focused;
                cx.notify();
            }
        });
        let s = state.read(cx);
        let open = s.intent.is_open() || s.focused;

        let (trigger_state, key_state, card_state) = (state.clone(), state.clone(), state.clone());
        let trigger = div()
            .id(ElementId::Name(format!("{id}-trigger").into()))
            .track_focus(&focus)
            .tab_stop(true)
            .relative()
            .flex_none()
            .px(px(m::PADDING_X))
            .py(px(m::PADDING_Y))
            .rounded(px(m::RADIUS))
            .border_1()
            .border_color(skin.trigger_border)
            .bg(skin.trigger_fill)
            .hover(move |st| st.bg(skin.trigger_hover))
            .text_color(skin.trigger_text)
            .text_size(px(m::FONT_SIZE))
            .line_height(px(m::LINE_HEIGHT))
            .font_family(fonts::SANS)
            .cursor_pointer()
            .on_hover(move |hovered, _, cx| {
                trigger_state.update(cx, |s, cx| {
                    if s.intent.set_trigger(*hovered, Instant::now()) {
                        cx.notify();
                    }
                });
            })
            .on_key_down(move |e: &KeyDownEvent, window, cx| {
                if e.keystroke.key == "escape" {
                    key_state.update(cx, |s, cx| {
                        s.intent.close();
                        s.focused = false;
                        cx.notify();
                    });
                    window.blur();
                }
            })
            .child(self.label)
            .child(anchor.tracker());

        let (title, aside, body) = (self.title, self.aside, self.body);
        let card = Popup::new(ElementId::Name(format!("{id}-card").into()), anchor)
            .open(open)
            .placement(Placement::new(Side::Bottom, super::Align::Center))
            .collision(Collision {
                side: super::Avoid::None,
                align: super::Avoid::Shift,
                padding: duotone::VIEWPORT_MARGIN,
            })
            .motion(DUOTONE_MOTION)
            .content(move |_, _, _| {
                let header = div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(duotone::HEADER_GAP))
                    .px(px(duotone::HEADER_PADDING_X))
                    .py(px(duotone::HEADER_PADDING_Y))
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .text_size(px(14.0))
                            .line_height(px(20.0))
                            .font_weight(FontWeight(600.0))
                            .text_color(skin.title)
                            .child(title),
                    )
                    .when_some(aside, |d, aside| {
                        d.child(
                            div()
                                .flex_none()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(skin.aside)
                                .child(aside),
                        )
                    });
                let content = div()
                    .px(px(duotone::CARD_PADDING_X))
                    .py(px(duotone::CARD_PADDING_Y + duotone::BODY_MARGIN_Y))
                    .rounded(px(duotone::CARD_RADIUS))
                    .border_1()
                    .border_color(skin.card_border)
                    .bg(skin.card)
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .text_color(skin.body)
                    .child(body);
                div()
                    .id("duotone-card")
                    .w(px(duotone::WIDTH))
                    .pt(px(duotone::GAP))
                    .font_family(fonts::SANS)
                    .on_hover(move |hovered, _, cx| {
                        card_state.update(cx, |s, cx| {
                            if s.intent.set_card(*hovered, Instant::now()) {
                                cx.notify();
                            }
                        });
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .px(px(duotone::RIM))
                            .pb(px(duotone::RIM))
                            .rounded(px(duotone::SHELL_RADIUS))
                            .border_1()
                            .border_color(skin.shell_border)
                            .bg(skin.shell)
                            .shadow(shadows(&skin.shadows))
                            .child(header)
                            .child(content),
                    )
                    .into_any_element()
            });

        div().relative().flex_none().child(trigger).child(card)
    }
}
