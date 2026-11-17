//! Overlay primitives shared by every floating surface: tooltips, menus, selects,
//! popovers, hover cards, dialogs, sheets and toasts.
//!
//! [`place`] is Base UI's positioner (Floating UI's `offset`, `flip` and `shift`
//! middleware): where a popup of a given size goes next to its anchor, which side it
//! ends up on after collision handling, its `--transform-origin` and where an arrow
//! points. [`PopupMotion`] and [`Presence`] describe the `data-starting-style` /
//! `data-ending-style` transitions the popups run when they mount and unmount.

use std::time::{Duration, Instant};

use crate::motion::{Tween, ease, lerp};

pub mod dialog;
pub mod menu;
pub mod select;
pub mod toast;

/// The side of its anchor a popup sits on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Side {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
}

impl Side {
    pub const ALL: [Side; 4] = [Side::Top, Side::Right, Side::Bottom, Side::Left];

    pub fn opposite(self) -> Self {
        match self {
            Side::Top => Side::Bottom,
            Side::Bottom => Side::Top,
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }

    /// Top and bottom popups align along the x axis.
    pub fn is_vertical(self) -> bool {
        matches!(self, Side::Top | Side::Bottom)
    }

    /// The unit vector pointing from the anchor towards the popup.
    pub fn direction(self) -> (f32, f32) {
        match self {
            Side::Top => (0.0, -1.0),
            Side::Bottom => (0.0, 1.0),
            Side::Left => (-1.0, 0.0),
            Side::Right => (1.0, 0.0),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Side::Top => "top",
            Side::Right => "right",
            Side::Bottom => "bottom",
            Side::Left => "left",
        }
    }
}

/// How a popup lines up with its anchor along the side it sits on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Align {
    Start,
    #[default]
    Center,
    End,
}

impl Align {
    pub fn opposite(self) -> Self {
        match self {
            Align::Start => Align::End,
            Align::Center => Align::Center,
            Align::End => Align::Start,
        }
    }

    fn fraction(self) -> f32 {
        match self {
            Align::Start => 0.0,
            Align::Center => 0.5,
            Align::End => 1.0,
        }
    }
}

/// An axis-aligned rectangle in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.right() && y >= self.y && y <= self.bottom()
    }
}

/// What a positioner does when the popup would leave the viewport on an axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Avoid {
    /// Move to the opposite side or alignment when it has more room.
    #[default]
    Flip,
    /// Slide along the axis until the popup is inside the collision padding.
    Shift,
    /// Stay put.
    None,
}

/// Base UI's `collisionAvoidance` and `collisionPadding`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Collision {
    pub side: Avoid,
    pub align: Avoid,
    /// Space kept between the popup and the viewport edges.
    pub padding: f32,
}

impl Collision {
    /// Base UI's popover, menu and tooltip default: flip the side and the alignment,
    /// then shift whatever still overflows, 5 px inside the viewport.
    pub const DEFAULT: Collision = Collision {
        side: Avoid::Flip,
        align: Avoid::Flip,
        padding: 5.0,
    };

    /// Notra's select: flip the side, shift the alignment.
    pub const FLIP_SHIFT: Collision = Collision {
        side: Avoid::Flip,
        align: Avoid::Shift,
        padding: 5.0,
    };
}

impl Default for Collision {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Where a popup asks to go: Base UI's `side`, `align`, `sideOffset` and `alignOffset`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Placement {
    pub side: Side,
    pub align: Align,
    /// Gap between the anchor and the popup.
    pub side_offset: f32,
    /// Shift along the anchor's edge, towards the end.
    pub align_offset: f32,
}

impl Placement {
    pub const fn new(side: Side, align: Align) -> Self {
        Self {
            side,
            align,
            side_offset: 0.0,
            align_offset: 0.0,
        }
    }

    pub const fn side_offset(mut self, offset: f32) -> Self {
        self.side_offset = offset;
        self
    }

    pub const fn align_offset(mut self, offset: f32) -> Self {
        self.align_offset = offset;
        self
    }
}

/// The result of [`place`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    /// The popup's rectangle, in the same space as the anchor and viewport.
    pub rect: Rect,
    /// The side it ended up on (`data-side`).
    pub side: Side,
    /// The alignment it ended up with (`data-align`).
    pub align: Align,
    /// `--transform-origin`, relative to the popup's top-left corner: the point on the
    /// anchor's edge the popup grows from.
    pub origin: (f32, f32),
    /// Where an arrow's centre sits along the popup's edge facing the anchor, relative to
    /// the popup's start on that axis: the anchor's centre, kept off the corners.
    pub arrow: f32,
}

/// Places a popup of `size` (width, height) next to `anchor` inside `viewport`.
///
/// Mirrors Floating UI as Base UI configures it: the offset, then `flip` on the side axis
/// (only when the opposite side has more room), then `flip` or `shift` on the alignment
/// axis, then a final shift so the popup stays `padding` inside the viewport.
pub fn place(
    anchor: Rect,
    size: (f32, f32),
    viewport: Rect,
    placement: Placement,
    collision: Collision,
) -> Placed {
    let (w, h) = size;
    let pad = collision.padding;
    let main_len = |side: Side| if side.is_vertical() { h } else { w };
    let room = |side: Side| match side {
        Side::Top => anchor.y - viewport.y - placement.side_offset - pad,
        Side::Bottom => viewport.bottom() - anchor.bottom() - placement.side_offset - pad,
        Side::Left => anchor.x - viewport.x - placement.side_offset - pad,
        Side::Right => viewport.right() - anchor.right() - placement.side_offset - pad,
    };

    let mut side = placement.side;
    if collision.side == Avoid::Flip
        && room(side) < main_len(side)
        && room(side.opposite()) > room(side)
    {
        side = side.opposite();
    }

    let cross_start = |side: Side, align: Align| {
        let (a0, alen, plen) = if side.is_vertical() {
            (anchor.x, anchor.w, w)
        } else {
            (anchor.y, anchor.h, h)
        };
        let offset = placement.align_offset * if align == Align::End { -1.0 } else { 1.0 };
        a0 + (alen - plen) * align.fraction() + offset
    };
    let (v0, v1, plen) = if side.is_vertical() {
        (viewport.x + pad, viewport.right() - pad, w)
    } else {
        (viewport.y + pad, viewport.bottom() - pad, h)
    };
    let overflow = |start: f32| (v0 - start).max(0.0) + (start + plen - v1).max(0.0);

    let mut align = placement.align;
    let mut cross = cross_start(side, align);
    if collision.align == Avoid::Flip && align != Align::Center && overflow(cross) > 0.0 {
        let flipped = cross_start(side, align.opposite());
        if overflow(flipped) < overflow(cross) {
            align = align.opposite();
            cross = flipped;
        }
    }
    if collision.align != Avoid::None || overflow(cross) > 0.0 {
        cross = cross.min(v1 - plen).max(v0);
    }

    let main = match side {
        Side::Top => anchor.y - placement.side_offset - h,
        Side::Bottom => anchor.bottom() + placement.side_offset,
        Side::Left => anchor.x - placement.side_offset - w,
        Side::Right => anchor.right() + placement.side_offset,
    };
    let rect = if side.is_vertical() {
        Rect::new(cross, main, w, h)
    } else {
        Rect::new(main, cross, w, h)
    };

    let (ax, ay) = anchor.center();
    let arrow = if side.is_vertical() {
        ax - rect.x
    } else {
        ay - rect.y
    };
    let origin_along = match align {
        Align::Start => {
            (if side.is_vertical() {
                anchor.x
            } else {
                anchor.y
            }) - cross
        }
        Align::Center => arrow,
        Align::End => {
            (if side.is_vertical() {
                anchor.right()
            } else {
                anchor.bottom()
            }) - cross
        }
    };
    let gap = placement.side_offset;
    let origin = match side {
        Side::Top => (origin_along, h + gap),
        Side::Bottom => (origin_along, -gap),
        Side::Left => (w + gap, origin_along),
        Side::Right => (-gap, origin_along),
    };

    Placed {
        rect,
        side,
        align,
        origin,
        arrow,
    }
}

/// A popup's `data-starting-style` and `data-ending-style` transition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopupMotion {
    /// Opening: from the starting style to rest.
    pub enter: Tween,
    /// Closing: from rest to the ending style.
    pub exit: Tween,
    /// Scale at both ends (`scale-[0.97]`), around [`Placed::origin`].
    pub scale: f32,
    /// Opening starts this far from rest, back towards the anchor (`translate-y-0.5`).
    pub enter_travel: f32,
    /// Closing ends this far from rest, back towards the anchor.
    pub exit_travel: f32,
}

impl PopupMotion {
    /// The Depth popup of Notra's menu and select: opacity and `scale-[0.97]` over 150 ms
    /// in and 100 ms out on `cubic-bezier(0.23, 1, 0.32, 1)`.
    pub const DEPTH: PopupMotion = PopupMotion {
        enter: Tween::new(Duration::from_millis(150), ease::OUT),
        exit: Tween::new(Duration::from_millis(100), ease::OUT),
        scale: 0.97,
        enter_travel: 0.0,
        exit_travel: 0.0,
    };

    /// No motion (`prefers-reduced-motion`, `data-instant`).
    pub const NONE: PopupMotion = PopupMotion {
        enter: Tween::new(Duration::ZERO, ease::LINEAR),
        exit: Tween::new(Duration::ZERO, ease::LINEAR),
        scale: 1.0,
        enter_travel: 0.0,
        exit_travel: 0.0,
    };

    /// The same timing with travel and scale dropped, as `motion-reduce:transition-opacity`.
    pub fn reduced(self) -> PopupMotion {
        PopupMotion {
            scale: 1.0,
            enter_travel: 0.0,
            exit_travel: 0.0,
            ..self
        }
    }

    /// The frame at `shown` (0 absent, 1 at rest), while opening or closing, on `side`.
    pub fn frame(&self, shown: f32, opening: bool, side: Side) -> PopupFrame {
        let gone = 1.0 - shown.clamp(0.0, 1.0);
        let travel = if opening {
            self.enter_travel
        } else {
            self.exit_travel
        };
        let (dx, dy) = side.direction();
        PopupFrame {
            opacity: shown.clamp(0.0, 1.0),
            scale: lerp(1.0, self.scale, gone),
            offset: (-dx * travel * gone, -dy * travel * gone),
        }
    }
}

/// One frame of a popup's transition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopupFrame {
    pub opacity: f32,
    pub scale: f32,
    /// Translation from the resting position, in pixels.
    pub offset: (f32, f32),
}

impl PopupFrame {
    pub const REST: PopupFrame = PopupFrame {
        opacity: 1.0,
        scale: 1.0,
        offset: (0.0, 0.0),
    };
}

/// Mount/unmount state of a popup for immediate-mode renderers: whether it is open, and
/// how far its enter or exit transition has run. A reversal mid-flight starts from where
/// the popup is, as a CSS transition does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Presence {
    open: bool,
    from: f32,
    since: Option<Instant>,
}

impl Presence {
    /// Settled open or closed, without a transition.
    pub fn new(open: bool) -> Self {
        Self {
            open,
            from: if open { 1.0 } else { 0.0 },
            since: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Opens or closes at `now`; returns whether anything changed.
    pub fn set(&mut self, open: bool, motion: &PopupMotion, now: Instant) -> bool {
        if open == self.open {
            return false;
        }
        self.from = self.shown(motion, now);
        self.open = open;
        self.since = Some(now);
        true
    }

    /// Opens without a transition (`data-instant`): a popup taking over from a neighbour.
    pub fn snap(&mut self, open: bool) {
        *self = Self::new(open);
    }

    fn tween<'a>(&self, motion: &'a PopupMotion) -> &'a Tween {
        if self.open {
            &motion.enter
        } else {
            &motion.exit
        }
    }

    /// 0 when absent, 1 at rest.
    pub fn shown(&self, motion: &PopupMotion, now: Instant) -> f32 {
        let target = if self.open { 1.0 } else { 0.0 };
        match self.since {
            Some(at) => lerp(self.from, target, self.tween(motion).progress(now - at)),
            None => target,
        }
    }

    /// Whether the popup still draws: open, or closing.
    pub fn is_visible(&self, motion: &PopupMotion, now: Instant) -> bool {
        self.open || self.is_animating(motion, now)
    }

    pub fn is_animating(&self, motion: &PopupMotion, now: Instant) -> bool {
        self.since
            .is_some_and(|at| !self.tween(motion).is_done(now - at))
    }

    /// The transition frame at `now` for a popup on `side`.
    pub fn frame(&self, motion: &PopupMotion, side: Side, now: Instant) -> PopupFrame {
        motion.frame(self.shown(motion, now), self.open, side)
    }

    /// Time since the last change, `Duration::MAX` when settled from the start.
    pub fn elapsed(&self, now: Instant) -> Duration {
        self.since.map_or(Duration::MAX, |at| now - at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW: Rect = Rect::new(0.0, 0.0, 1000.0, 700.0);

    #[test]
    fn top_centre_matches_the_live_tooltip() {
        let trigger = Rect::new(371.5625, 334.0, 45.890625, 32.0);
        let p = place(
            trigger,
            (120.234375, 26.0),
            VIEW,
            Placement::new(Side::Top, Align::Center).side_offset(4.0),
            Collision::DEFAULT,
        );
        assert!((p.rect.x - 334.39).abs() < 0.05);
        assert_eq!(p.rect.y, 304.0);
        assert_eq!(p.side, Side::Top);
        assert!((p.origin.0 - 60.117).abs() < 0.01 && p.origin.1 == 30.0);
    }

    #[test]
    fn flips_only_when_the_other_side_has_more_room() {
        let trigger = Rect::new(20.0, 650.0, 40.0, 20.0);
        let place_at = |trigger| {
            place(
                trigger,
                (100.0, 80.0),
                VIEW,
                Placement::new(Side::Bottom, Align::Start).side_offset(4.0),
                Collision::DEFAULT,
            )
        };
        let p = place_at(trigger);
        assert_eq!(p.side, Side::Top);
        assert_eq!(p.rect.bottom(), 646.0);
        assert_eq!(
            place_at(Rect::new(20.0, 100.0, 40.0, 20.0)).side,
            Side::Bottom
        );
    }

    #[test]
    fn shifts_inside_the_padding() {
        let p = place(
            Rect::new(980.0, 100.0, 10.0, 10.0),
            (100.0, 30.0),
            VIEW,
            Placement::new(Side::Bottom, Align::Center),
            Collision::FLIP_SHIFT,
        );
        assert_eq!(p.rect.right(), 995.0);
        assert!(p.arrow > 50.0);
    }

    #[test]
    fn end_alignment_flips_to_start_near_the_left_edge() {
        let p = place(
            Rect::new(10.0, 100.0, 30.0, 20.0),
            (120.0, 40.0),
            VIEW,
            Placement::new(Side::Bottom, Align::End),
            Collision::DEFAULT,
        );
        assert_eq!(p.align, Align::Start);
        assert_eq!(p.rect.x, 10.0);
    }

    #[test]
    fn presence_reverses_from_where_it_is() {
        let motion = PopupMotion::DEPTH;
        let t0 = Instant::now();
        let mut p = Presence::new(false);
        assert!(p.set(true, &motion, t0));
        let mid = t0 + Duration::from_millis(30);
        let shown = p.shown(&motion, mid);
        assert!(shown > 0.3 && shown < 1.0);
        p.set(false, &motion, mid);
        assert!((p.shown(&motion, mid) - shown).abs() < 1e-4);
        assert!(p.is_visible(&motion, mid + Duration::from_millis(50)));
        assert!(!p.is_visible(&motion, mid + Duration::from_millis(101)));
    }

    #[test]
    fn frames_scale_and_travel_towards_the_anchor() {
        let motion = PopupMotion {
            enter_travel: 2.0,
            ..PopupMotion::DEPTH
        };
        let f = motion.frame(0.0, true, Side::Top);
        assert_eq!(f.opacity, 0.0);
        assert_eq!(f.scale, 0.97);
        assert_eq!(f.offset, (0.0, 2.0));
        assert_eq!(motion.frame(0.0, false, Side::Top).offset, (0.0, 0.0));
        assert_eq!(motion.frame(1.0, true, Side::Left), PopupFrame::REST);
    }
}
