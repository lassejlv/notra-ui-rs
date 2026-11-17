//! Overlay building blocks for GPUI: anchoring a popup to a trigger, placing it with
//! [`place`](crate::components::overlays::place), animating its presence, and dismissing it on
//! Escape or a press outside.
//!
//! ```ignore
//! let anchor = window.use_keyed_state("menu-anchor", cx, |_, _| AnchorBounds::default()).read(cx).clone();
//! div()
//!     .relative()
//!     .child(trigger)
//!     .child(anchor.tracker())
//!     .child(
//!         Popup::new("menu", anchor.clone())
//!             .open(self.open)
//!             .placement(Placement::new(Side::Bottom, Align::Start).side_offset(4.0))
//!             .on_dismiss(cx.listener(|this, _, _, cx| { this.open = false; cx.notify(); }))
//!             .content(|placed, _, _| menu_surface(placed).into_any_element()),
//!     )
//! ```

use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

use ::gpui::{
    AnyElement, App, Bounds, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    MouseDownEvent, ParentElement, Pixels, RenderOnce, Size, Styled, Window, anchored, canvas,
    deferred, div, point, prelude::*, px,
};

use crate::components::overlays::place;
pub use crate::components::overlays::{
    Align, Avoid, Collision, Placed, Placement, PopupFrame, PopupMotion, Presence, Rect, Side,
};

/// The window bounds of a trigger, captured while it prepaints so a popup can anchor to it.
/// Clones share one cell.
#[derive(Clone, Default)]
pub struct AnchorBounds(Rc<Cell<Option<Bounds<Pixels>>>>);

impl AnchorBounds {
    /// The bounds from the last prepaint.
    pub fn get(&self) -> Option<Bounds<Pixels>> {
        self.0.get()
    }

    /// The bounds as a [`Rect`].
    pub fn rect(&self) -> Option<Rect> {
        self.get().map(rect)
    }

    /// An invisible element that records its parent's bounds; add it as the last child of a
    /// `relative()` trigger or wrapper.
    pub fn tracker(&self) -> impl IntoElement {
        let cell = self.0.clone();
        canvas(move |bounds, _, _| cell.set(Some(bounds)), |_, _, _, _| {})
            .absolute()
            .top_0()
            .left_0()
            .size_full()
    }
}

/// Converts GPUI bounds to a [`Rect`].
pub fn rect(bounds: Bounds<Pixels>) -> Rect {
    Rect::new(
        f32::from(bounds.origin.x),
        f32::from(bounds.origin.y),
        f32::from(bounds.size.width),
        f32::from(bounds.size.height),
    )
}

/// Why a popup asked to close.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dismiss {
    Escape,
    /// A press outside both the popup and its anchor.
    OutsidePress,
}

type Content = Box<dyn FnOnce(&Placed, &mut Window, &mut App) -> AnyElement>;
type DismissHandler = Rc<dyn Fn(Dismiss, &mut Window, &mut App)>;

/// A floating surface anchored to a trigger, drawn above the window with `deferred`.
///
/// It stays mounted while its exit transition runs, measures itself on its first frame (drawn
/// transparent), and is placed with [`place`] every frame so it follows its anchor. With
/// [`Popup::focus`] it takes focus when it opens, handles Escape, and gives focus back to
/// whatever had it when it closes.
#[derive(IntoElement)]
pub struct Popup {
    id: ElementId,
    anchor: AnchorBounds,
    open: bool,
    instant: bool,
    placement: Placement,
    collision: Collision,
    motion: PopupMotion,
    focus: Option<FocusHandle>,
    priority: usize,
    content: Option<Content>,
    on_dismiss: Option<DismissHandler>,
}

impl Popup {
    pub fn new(id: impl Into<ElementId>, anchor: AnchorBounds) -> Self {
        Self {
            id: id.into(),
            anchor,
            open: false,
            instant: false,
            placement: Placement::default(),
            collision: Collision::DEFAULT,
            motion: PopupMotion::DEPTH,
            focus: None,
            priority: 1,
            content: None,
            on_dismiss: None,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Open and close without a transition (Base UI's `data-instant`).
    pub fn instant(mut self, instant: bool) -> Self {
        self.instant = instant;
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    pub fn collision(mut self, collision: Collision) -> Self {
        self.collision = collision;
        self
    }

    pub fn motion(mut self, motion: PopupMotion) -> Self {
        self.motion = motion;
        self
    }

    /// Focus this handle while open (menus, selects, popovers), with Escape dismissing.
    pub fn focus(mut self, handle: FocusHandle) -> Self {
        self.focus = Some(handle);
        self
    }

    /// Stacking among deferred elements; nested popups (submenus) use a higher value.
    pub fn priority(mut self, priority: usize) -> Self {
        self.priority = priority;
        self
    }

    /// The surface, built once its placement is known (its final side, origin and arrow).
    pub fn content(
        mut self,
        content: impl FnOnce(&Placed, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(Box::new(content));
        self
    }

    /// Escape (with [`Popup::focus`]) or a press outside the popup and its anchor.
    pub fn on_dismiss(
        mut self,
        handler: impl Fn(Dismiss, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }
}

struct PopupState {
    presence: Presence,
    size: Option<Size<Pixels>>,
    restore: Option<FocusHandle>,
}

impl RenderOnce for Popup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let now = Instant::now();
        let motion = if cx.reduce_motion() {
            self.motion.reduced()
        } else {
            self.motion
        };
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| PopupState {
            presence: Presence::new(false),
            size: None,
            restore: None,
        });

        let (opened, closed) = state.update(cx, |s, _| {
            let was = s.presence.is_open();
            if self.instant {
                s.presence.snap(self.open);
            } else {
                s.presence.set(self.open, &motion, now);
            }
            (!was && self.open, was && !self.open)
        });

        if let Some(handle) = self.focus.clone() {
            if opened {
                let previous = window.focused(cx);
                state.update(cx, |s, _| s.restore = previous);
                window.defer(cx, move |window, cx| handle.focus(window, cx));
            } else if closed {
                let restore = state.update(cx, |s, _| s.restore.take());
                if handle.contains_focused(window, cx)
                    && let Some(previous) = restore
                {
                    window.defer(cx, move |window, cx| previous.focus(window, cx));
                }
            }
        }

        let s = state.read(cx);
        let visible = s.presence.is_visible(&motion, now);
        let (Some(anchor), Some(content), true) = (self.anchor.rect(), self.content, visible)
        else {
            return div().into_any_element();
        };
        if s.presence.is_animating(&motion, now) || s.size.is_none() {
            window.request_animation_frame();
        }

        let measured = s.size;
        let size = measured.map_or((0.0, 0.0), |size| {
            (f32::from(size.width), f32::from(size.height))
        });
        let viewport = window.viewport_size();
        let placed = place(
            anchor,
            size,
            Rect::new(
                0.0,
                0.0,
                f32::from(viewport.width),
                f32::from(viewport.height),
            ),
            self.placement,
            self.collision,
        );
        let frame = s.presence.frame(&motion, placed.side, now);
        let opacity = if measured.is_some() {
            frame.opacity
        } else {
            0.0
        };
        let is_open = s.presence.is_open();

        let body = content(&placed, window, cx);
        let measure_state = state.clone();
        let on_dismiss = self.on_dismiss;
        let outside = on_dismiss.clone();
        let anchor_bounds = self.anchor.clone();

        let surface = div()
            .on_children_prepainted(move |bounds, _, cx| {
                let Some(size) = bounds.first().map(|b| b.size) else {
                    return;
                };
                measure_state.update(cx, |s, cx| {
                    if s.size != Some(size) {
                        s.size = Some(size);
                        cx.notify();
                    }
                });
            })
            .id(self.id)
            .occlude()
            .opacity(opacity)
            .when_some(self.focus, |d, handle| d.track_focus(&handle))
            .when(is_open, |d| {
                d.on_mouse_down_out(move |event: &MouseDownEvent, window, cx| {
                    let inside_anchor = anchor_bounds
                        .get()
                        .is_some_and(|b| b.contains(&event.position));
                    if !inside_anchor && let Some(handler) = &outside {
                        handler(Dismiss::OutsidePress, window, cx);
                    }
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape"
                        && let Some(handler) = &on_dismiss
                    {
                        cx.stop_propagation();
                        handler(Dismiss::Escape, window, cx);
                    }
                })
            })
            .child(body);

        deferred(
            anchored()
                .position(point(
                    px(placed.rect.x + frame.offset.0),
                    px(placed.rect.y + frame.offset.1),
                ))
                .snap_to_window()
                .child(surface),
        )
        .with_priority(self.priority)
        .into_any_element()
    }
}
