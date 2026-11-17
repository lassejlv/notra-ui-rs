use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use ::gpui::{
    AnyElement, App, ElementId, FontWeight, Global, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Transformation, Window, anchored, deferred, div, point,
    prelude::*, px, radians, svg,
};

use crate::components::button::{ButtonSize, ButtonVariant};
use crate::components::overlays::toast::{
    LEAVE_BEHIND, LEAVE_BEHIND_FADE, MOVE, SPIN, Slot, Toast, ToastKind, ToastQueue, ToastSkin,
    metrics,
};
use crate::fonts;
use crate::gpui::{ActiveTheme, Button, gradient, shadows};
use crate::motion::{Animated, Tween};

/// The app's toast queue, a GPUI global.
struct Toasts {
    queue: ToastQueue,
    epoch: Instant,
}

impl Global for Toasts {}

fn seconds(epoch: Instant) -> f64 {
    epoch.elapsed().as_secs_f64()
}

/// Shows `toast` (Sonner's `toast()`); a toast with the same id updates in place.
pub fn toast(cx: &mut App, toast: Toast) {
    if !cx.has_global::<Toasts>() {
        cx.set_global(Toasts {
            queue: ToastQueue::default(),
            epoch: Instant::now(),
        });
    }
    let toasts = cx.global_mut::<Toasts>();
    let now = seconds(toasts.epoch);
    toasts.queue.push(toast, now);
    cx.refresh_windows();
}

/// Starts the toast `id` leaving (Sonner's `toast.dismiss(id)`).
pub fn dismiss_toast(cx: &mut App, id: &str) {
    if let Some(toasts) = cx.try_global::<Toasts>() {
        let now = seconds(toasts.epoch);
        cx.global_mut::<Toasts>().queue.dismiss(id, now);
        cx.refresh_windows();
    }
}

type ActionHandler = Rc<dyn Fn(&Toast, &mut Window, &mut App)>;

/// Sonner's `<Toaster position="bottom-center" />` with Notra's toast styling: renders the
/// queue filled by [`toast`] as a stack at the bottom of the window. Hovering the stack
/// expands it and pauses the timers.
#[derive(IntoElement)]
pub struct Toaster {
    id: ElementId,
    on_action: Option<ActionHandler>,
}

impl Toaster {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            on_action: None,
        }
    }

    /// A toast's action button was pressed (the toast is dismissed after).
    pub fn on_action(mut self, handler: impl Fn(&Toast, &mut Window, &mut App) + 'static) -> Self {
        self.on_action = Some(Rc::new(handler));
        self
    }
}

/// One toast's motion.
struct Motion {
    lift: Animated,
    scale: Animated,
    height: Animated,
    opacity: Animated,
    content: Animated,
}

impl Motion {
    fn entering(slot: &Slot, height: f32, now: Instant) -> Self {
        let mut m = Self {
            lift: Animated::new(-height, MOVE),
            scale: Animated::new(slot.scale, MOVE),
            height: Animated::new(slot.height, MOVE),
            opacity: Animated::new(0.0, MOVE),
            content: Animated::new(slot.content, MOVE),
        };
        m.lift.set(slot.lift, now);
        m.opacity.set(slot.opacity, now);
        m
    }

    fn retarget(&mut self, slot: &Slot, now: Instant) {
        let set = |a: &mut Animated, v: f32| {
            if a.target() != v {
                a.set(v, now);
            }
        };
        set(&mut self.lift, slot.lift);
        set(&mut self.scale, slot.scale);
        set(&mut self.height, slot.height);
        set(&mut self.opacity, slot.opacity);
        set(&mut self.content, slot.content);
    }

    fn leave(&mut self, front: bool, expanded: bool, height: f32, now: Instant) {
        if !front && !expanded {
            self.lift = retween(self.lift, LEAVE_BEHIND, now);
            self.lift.set(self.lift.value(now) - 0.4 * height, now);
            self.opacity = retween(self.opacity, LEAVE_BEHIND_FADE, now);
            self.opacity.set(0.0, now);
        } else {
            self.lift.set(self.lift.target() - height, now);
            self.opacity.set(0.0, now);
        }
    }
}

fn retween(a: Animated, tween: Tween, now: Instant) -> Animated {
    let mut b = Animated::new(a.value(now), tween);
    b.snap(a.value(now));
    b
}

#[derive(Default)]
struct ToasterState {
    motions: HashMap<String, (Motion, bool)>,
    hovered: bool,
}

impl RenderOnce for Toaster {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let now = Instant::now();
        let reduce = cx.reduce_motion();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| ToasterState::default());
        let Some(toasts) = cx.try_global::<Toasts>() else {
            return div().absolute().into_any_element();
        };
        let epoch = toasts.epoch;
        let hovered = state.read(cx).hovered;
        let queue = {
            let toasts = cx.global_mut::<Toasts>();
            toasts.queue.expanded = hovered;
            toasts.queue.tick(seconds(epoch));
            toasts.queue.clone()
        };
        if queue.entries().is_empty() {
            state.update(cx, |s, _| s.motions.clear());
            return div().absolute().into_any_element();
        }
        window.request_animation_frame();

        let slots = queue.slots();
        let live: Vec<&str> = queue.live().map(|e| e.toast.id.as_str()).collect();
        let front = live.first().copied();
        state.update(cx, |s, _| {
            for entry in queue.entries() {
                let id = entry.toast.id.as_str();
                let height = entry.toast.height();
                match live.iter().position(|l| *l == id) {
                    Some(i) => {
                        let slot = &slots[i];
                        match s.motions.get_mut(id) {
                            Some((m, _)) => m.retarget(slot, now),
                            None => {
                                let m = Motion::entering(slot, height, now);
                                s.motions.insert(id.to_string(), (m, false));
                            }
                        }
                    }
                    None => {
                        if let Some((m, left)) = s.motions.get_mut(id)
                            && !*left
                        {
                            *left = true;
                            m.leave(
                                front.is_none() || front == Some(id),
                                queue.expanded,
                                height,
                                now,
                            );
                        }
                    }
                }
            }
            s.motions
                .retain(|id, _| queue.entries().iter().any(|e| e.toast.id == *id));
        });

        let theme = *cx.notra();
        let skin = Rc::new(ToastSkin::new(&theme));
        let viewport = window.viewport_size();
        let (vw, vh) = (f32::from(viewport.width), f32::from(viewport.height));
        let extent = slots
            .iter()
            .map(|s| s.lift + s.height)
            .fold(0.0_f32, f32::max);
        let s = state.read(cx);
        let mut children: Vec<AnyElement> = Vec::new();
        for entry in queue.entries().iter().rev() {
            let Some((m, _)) = s.motions.get(&entry.toast.id) else {
                continue;
            };
            let at = |a: &Animated| if reduce { a.target() } else { a.value(now) };
            let (lift, scale, height, opacity, content) = (
                at(&m.lift),
                at(&m.scale),
                at(&m.height),
                at(&m.opacity),
                at(&m.content),
            );
            let width = metrics::WIDTH * scale;
            children.push(
                div()
                    .absolute()
                    .left(px((metrics::WIDTH - width) / 2.0))
                    .bottom(px(lift + height * (1.0 - scale) / 2.0))
                    .w(px(width))
                    .h(px(height * scale))
                    .opacity(opacity)
                    .child(toast_box(
                        &entry.toast,
                        &skin,
                        content,
                        &self.on_action,
                        now,
                    ))
                    .into_any_element(),
            );
        }
        let hover_state = state.clone();
        deferred(
            anchored()
                .position(point(
                    px((vw - metrics::WIDTH) / 2.0),
                    px(vh - metrics::OFFSET - extent),
                ))
                .child(
                    div()
                        .id(self.id)
                        .relative()
                        .w(px(metrics::WIDTH))
                        .h(px(extent))
                        .on_hover(move |hovered, _, cx| {
                            hover_state.update(cx, |s, cx| {
                                s.hovered = *hovered;
                                cx.notify();
                            })
                        })
                        .children(children),
                ),
        )
        .with_priority(5)
        .into_any_element()
    }
}

fn toast_box(
    toast: &Toast,
    skin: &ToastSkin,
    content: f32,
    on_action: &Option<ActionHandler>,
    now: Instant,
) -> AnyElement {
    let (tile, ink) = skin.status.of(toast.kind);
    let icon = toast.kind.icon().map(|icon| {
        let spin = toast.kind == ToastKind::Loading;
        let turn = spin_angle(now);
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(metrics::ICON_BOX))
            .rounded(px(metrics::ICON_RADIUS))
            .bg(tile)
            .child(
                svg()
                    .path(icon.path())
                    .size(px(metrics::ICON))
                    .text_color(ink)
                    .when(spin, |s| {
                        s.with_transformation(Transformation::rotate(radians(turn)))
                    }),
            )
    });
    let action = toast.action.clone().map(|label| {
        let handler = on_action.clone();
        let t = toast.clone();
        Button::new(SharedString::from(format!("{}-action", toast.id)), label)
            .size(ButtonSize::Xs)
            .on_click(move |_, window, cx| {
                dismiss_toast(cx, &t.id);
                if let Some(h) = &handler {
                    h(&t, window, cx);
                }
            })
    });
    let cancel = toast.cancel.clone().map(|label| {
        let id = toast.id.clone();
        Button::new(SharedString::from(format!("{}-cancel", toast.id)), label)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Xs)
            .on_click(move |_, _, cx| dismiss_toast(cx, &id))
    });
    div()
        .size_full()
        .flex()
        .items_center()
        .gap(px(metrics::GAP))
        .p(px(metrics::PADDING))
        .rounded(px(metrics::RADIUS))
        .border_1()
        .border_color(skin.border)
        .bg(gradient(
            skin.gradient.0.over(skin.fill),
            skin.gradient.1.over(skin.fill),
        ))
        .shadow(shadows(&skin.shadows))
        .text_color(skin.text)
        .font_family(fonts::SANS)
        .overflow_hidden()
        .child(
            div()
                .flex()
                .flex_1()
                .items_center()
                .gap(px(metrics::GAP))
                .opacity(content)
                .children(icon)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .gap(px(metrics::TEXT_GAP))
                        .child(
                            div()
                                .text_size(px(metrics::TITLE_FONT))
                                .line_height(px(metrics::TITLE_LINE))
                                .font_weight(FontWeight(500.0))
                                .child(SharedString::from(toast.title.clone())),
                        )
                        .when_some(toast.description.clone(), |d, text| {
                            d.child(
                                div()
                                    .text_size(px(metrics::DESCRIPTION_FONT))
                                    .line_height(px(metrics::DESCRIPTION_LINE))
                                    .text_color(skin.description)
                                    .child(SharedString::from(text)),
                            )
                        }),
                )
                .children(cancel)
                .children(action),
        )
        .into_any_element()
}

fn spin_angle(now: Instant) -> f32 {
    let epoch = *SPIN_EPOCH.get_or_init(Instant::now);
    let t = now.saturating_duration_since(epoch).as_secs_f32() % SPIN.as_secs_f32();
    t / SPIN.as_secs_f32() * std::f32::consts::TAU
}

static SPIN_EPOCH: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
