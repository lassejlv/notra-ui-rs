use std::rc::Rc;
use std::time::Duration;

use ::rune::{El, Tx, col, el, icon, row, text};
use ::rune_motion::{Effect, LoopSpec, SpringPair, Transition, Vec2};

use super::spring;
use crate::components::button::{ButtonSize, ButtonVariant};
use crate::components::overlays::toast::{
    MOVE, SPIN, Slot, Toast, ToastKind, ToastQueue, ToastSkin, metrics,
};
use crate::rune::{Button, ToRune, sans, tween};
use crate::theme::Theme;

/// The app's toasts. Push from handlers, call [`ToastState::poll`] from `AppSpec::poll` so
/// toasts expire, and mount [`Toaster`] over the window.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToastState {
    queue: ToastQueue,
}

impl ToastState {
    /// Shows `toast` (Sonner's `toast()`); a toast with the same id updates in place.
    pub fn push(&mut self, toast: Toast, tx: &mut Tx) {
        let now = tx.clock();
        self.queue.tick(now);
        self.queue.push(toast, now);
        self.schedule(now, tx);
    }

    /// Shows `toast` at `now` on the frame clock, outside a handler (a restored session, a
    /// scene); the next [`ToastState::poll`] schedules its expiry.
    pub fn push_at(&mut self, toast: Toast, now: f64) {
        self.queue.push(toast, now);
    }

    /// Starts the toast `id` leaving.
    pub fn dismiss(&mut self, id: &str, tx: &mut Tx) {
        let now = tx.clock();
        self.queue.tick(now);
        self.queue.dismiss(id, now);
        self.schedule(now, tx);
    }

    /// Expires and unmounts toasts at `now` and asks for a frame at the next deadline; true
    /// when the stack changed. Call it from `AppSpec::poll`.
    pub fn poll(&mut self, now: f64, tx: &mut Tx) -> bool {
        let changed = self.queue.tick(now);
        self.schedule(now, tx);
        changed
    }

    fn schedule(&self, now: f64, tx: &mut Tx) {
        if let Some(wait) = self.queue.next_deadline(now) {
            tx.wake_after(Duration::from_secs_f64(wait + 0.001));
        }
    }

    pub fn queue(&self) -> &ToastQueue {
        &self.queue
    }

    /// Expands or collapses the stack outside a handler (a scene).
    pub fn expand(&mut self, expanded: bool) {
        self.queue.expanded = expanded;
    }

    /// Expands the stack and pauses timers (the stack is hovered).
    pub fn set_expanded(&mut self, expanded: bool, tx: &mut Tx) {
        let now = tx.clock();
        self.queue.tick(now);
        self.queue.expanded = expanded;
        self.schedule(now, tx);
    }
}

/// Picks the toast state out of the app's.
pub type Lens<S> = fn(&mut S) -> &mut ToastState;

type Action<S> = Rc<dyn Fn(&mut S, &Toast, &mut Tx)>;

/// Sonner's `<Toaster position="bottom-center" />` with Notra's toasts: a full-window layer
/// holding the stack 24 pt above the bottom edge. Hovering it expands the stack and pauses
/// the timers.
pub struct Toaster {
    key: String,
}

impl Toaster {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }

    /// The layer; `on_action` runs when a toast's action button is pressed (the toast is
    /// dismissed first).
    pub fn build<S: 'static>(
        self,
        state: &ToastState,
        theme: &Theme,
        lens: Lens<S>,
        on_action: impl Fn(&mut S, &Toast, &mut Tx) + 'static,
    ) -> El {
        let queue = state.queue();
        let skin = ToastSkin::new(theme);
        let slots = queue.slots();
        let extent = slots
            .iter()
            .map(|s| s.lift + s.height)
            .fold(0.0_f32, f32::max);
        let action: Action<S> = Rc::new(on_action);
        let live: Vec<_> = queue.live().collect();
        let toasts = live
            .iter()
            .zip(slots.iter())
            .rev()
            .map(|(entry, slot)| self.toast(&entry.toast, slot, theme, &skin, lens, &action))
            .collect::<Vec<_>>();
        let stack = el()
            .key(format!("{}-stack", self.key))
            .role("region")
            .label("Notifications")
            .w(metrics::WIDTH)
            .h(extent)
            .motion(spring(MOVE))
            .on_hover_change(move |s: &mut S, inside: bool, tx: &mut Tx| {
                lens(s).set_expanded(inside, tx)
            })
            .children(toasts);
        col()
            .key(self.key)
            .absolute(0.0, 0.0)
            .w_full()
            .h_full()
            .justify_end()
            .items_center()
            .pb(metrics::OFFSET)
            .child(stack)
    }

    fn toast<S: 'static>(
        &self,
        toast: &Toast,
        slot: &Slot,
        theme: &Theme,
        skin: &ToastSkin,
        lens: Lens<S>,
        action: &Action<S>,
    ) -> El {
        let height = toast.height();
        let (tile, ink) = skin.status.of(toast.kind);
        let mut body = row()
            .key("content")
            .grow(1.0)
            .center_y()
            .gap(metrics::GAP)
            .opacity(slot.content)
            .opacity_motion(tween(MOVE));
        if let Some(i) = toast.kind.icon() {
            let mut glyph = icon(i.rune(), metrics::ICON).color(ink.rune());
            if toast.kind == ToastKind::Loading {
                glyph = glyph.looping(LoopSpec::rotation(SPIN.as_secs_f64()));
            }
            body = body.child(
                row()
                    .key("icon")
                    .size(metrics::ICON_BOX, metrics::ICON_BOX)
                    .center_y()
                    .justify_center()
                    .radius(metrics::ICON_RADIUS)
                    .continuous_corners()
                    .fill(tile.rune())
                    .a11y_hidden()
                    .child(glyph),
            );
        }
        let mut copy = col()
            .key("text")
            .grow(1.0)
            .shrink()
            .gap(metrics::TEXT_GAP)
            .child(
                text(toast.title.clone())
                    .font(sans(metrics::TITLE_FONT, 500.0, metrics::TITLE_LINE))
                    .color(skin.text.rune()),
            );
        if let Some(d) = &toast.description {
            copy = copy.child(
                text(d.clone())
                    .font(sans(
                        metrics::DESCRIPTION_FONT,
                        400.0,
                        metrics::DESCRIPTION_LINE,
                    ))
                    .color(skin.description.rune()),
            );
        }
        body = body.child(copy);
        if let Some(label) = &toast.cancel {
            let id = toast.id.clone();
            body = body.child(
                Button::new(label.clone())
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Xs)
                    .on_press_tx(move |s: &mut S, tx: &mut Tx| lens(s).dismiss(&id, tx))
                    .build(theme)
                    .key("cancel"),
            );
        }
        if let Some(label) = &toast.action {
            let t = toast.clone();
            let action = action.clone();
            body = body.child(
                Button::new(label.clone())
                    .size(ButtonSize::Xs)
                    .on_press_tx(move |s: &mut S, tx: &mut Tx| {
                        lens(s).dismiss(&t.id, tx);
                        action(s, &t, tx);
                    })
                    .build(theme)
                    .key("action"),
            );
        }
        let mut e = row()
            .key(format!("{}-{}", self.key, toast.id))
            .role("status")
            .label(toast.title.clone())
            .pin(Some(0.0), None, Some(0.0), Some(0.0))
            .h(slot.height)
            .padding(metrics::PADDING)
            .radius(metrics::RADIUS)
            .continuous_corners()
            .border(1.0, skin.border.rune())
            .fill(skin.fill.rune())
            .gradient(
                skin.gradient.0.over(skin.fill).rune(),
                skin.gradient.1.over(skin.fill).rune(),
            )
            .clip_rounded(metrics::RADIUS)
            .offset(0.0, -slot.lift)
            .offset_motion(tween(MOVE))
            .scale(slot.scale)
            .scale_motion(tween(MOVE))
            .opacity(slot.opacity)
            .opacity_motion(tween(MOVE))
            .motion(spring(MOVE))
            .transition(Transition {
                insertion: Effect {
                    opacity: 0.0,
                    offset: Vec2::new(0.0, height),
                    ..Effect::IDENTITY
                },
                removal: Effect {
                    opacity: 0.0,
                    offset: Vec2::new(0.0, height),
                    ..Effect::IDENTITY
                },
                pop_layout: true,
                springs: SpringPair::new(spring(MOVE), spring(MOVE)),
            })
            .child(body);
        for s in &skin.shadows {
            e = e.box_shadow(s.rune());
        }
        e
    }
}
