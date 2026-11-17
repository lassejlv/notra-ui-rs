//! Overlay building blocks for Rune.
//!
//! A popup is two elements: the trigger, keyed, where the view puts it, and the surface,
//! [`popup`]-anchored to that key and mounted later in the tree (ideally at the window's
//! root through [`root`]) so it paints above everything before it and escapes the
//! trigger's clipping ancestors. Opening and closing is the app's state; Rune runs the
//! enter and exit transitions when the surface mounts and unmounts.
//!
//! ```ignore
//! let mut layer = Vec::new();
//! let trigger = Button::new("Open menu").on_press(|s: &mut App| s.menu = true).build(&theme).key("menu-trigger");
//! if s.menu {
//!     layer.push(popup(surface, "menu-trigger", Placement::new(Side::Bottom, Align::Start).side_offset(4.0), &PopupMotion::DEPTH)
//!         .modal(true, |s: &mut App| s.menu = false));
//! }
//! overlays::root(page_with(trigger), layer)
//! ```

use ::rune::{El, el};
use ::rune_motion::{Effect, Spring, SpringPair, Transition, Vec2};

pub use crate::components::overlays::{
    Align, Avoid, Collision, Placed, Placement, PopupMotion, Rect, Side,
};
use crate::motion::Tween;

mod menu;
mod select;

pub use crate::components::overlays::menu::{ItemKind, MenuEntry, MenuItem};
pub use crate::components::overlays::select::{
    SelectEntry, SelectOption, SelectSize, SelectVariant,
};
pub use menu::{DropdownMenu, MenuState, depth_surface};
pub use select::{Select, SelectState};

/// Rune's side for a placement side.
pub fn side(side: Side) -> ::rune::Side {
    match side {
        Side::Top => ::rune::Side::Top,
        Side::Right => ::rune::Side::Right,
        Side::Bottom => ::rune::Side::Bottom,
        Side::Left => ::rune::Side::Left,
    }
}

/// Rune's alignment for a placement alignment.
pub fn align(align: Align) -> ::rune::Align {
    match align {
        Align::Start => ::rune::Align::Start,
        Align::Center => ::rune::Align::Center,
        Align::End => ::rune::Align::End,
    }
}

/// A critically damped spring that reaches 90 % when `tween` does: Rune runs presence on
/// springs, so a CSS enter or exit tween becomes the spring that feels the same.
pub fn spring(tween: Tween) -> Spring {
    let duration = tween.duration.as_secs_f32();
    if duration <= 0.0 {
        return Spring::NONE;
    }
    let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        if tween.easing.ease(mid) < 0.9 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    const CRITICAL_T90: f32 = 3.890_2;
    let t90 = hi * duration;
    Spring::new((t90 * std::f32::consts::TAU / CRITICAL_T90).max(0.001), 0.0)
}

/// The point a popup on `side` with `align` grows from, in unit coordinates of the popup.
pub fn origin(side: Side, align: Align) -> Vec2 {
    let along = match align {
        Align::Start => 0.0,
        Align::Center => 0.5,
        Align::End => 1.0,
    };
    match side {
        Side::Top => Vec2::new(along, 1.0),
        Side::Bottom => Vec2::new(along, 0.0),
        Side::Left => Vec2::new(1.0, along),
        Side::Right => Vec2::new(0.0, along),
    }
}

/// `motion`'s starting and ending styles as a Rune presence transition for a popup on
/// `side` with `align`.
pub fn transition(motion: &PopupMotion, side: Side, align: Align) -> Transition {
    let anchor = origin(side, align);
    let (dx, dy) = side.direction();
    let effect = |travel: f32| Effect {
        opacity: 0.0,
        scale: motion.scale,
        anchor,
        offset: Vec2::new(-dx * travel, -dy * travel),
        ..Effect::IDENTITY
    };
    Transition {
        insertion: effect(motion.enter_travel),
        removal: effect(motion.exit_travel),
        pop_layout: true,
        springs: SpringPair::new(spring(motion.enter), spring(motion.exit)),
    }
}

/// Anchors `surface` to the element keyed `anchor` on `placement`, entering and leaving with
/// `motion`. Rune flips it to the opposite side when it doesn't fit and keeps it inside the
/// window. Above or below its anchor it is at most as tall as the room it has, so a long
/// list inside should scroll (`.shrink().min_h(0.0)` on the viewport and its columns).
pub fn popup(surface: El, anchor: &str, placement: Placement, motion: &PopupMotion) -> El {
    let (s, a) = (side(placement.side), align(placement.align));
    let shift = placement.align_offset;
    let surface = if placement.side.is_vertical() {
        surface.anchored_fit(anchor, s, a, placement.side_offset)
    } else {
        surface.anchored(anchor, s, a, placement.side_offset)
    };
    let surface = if shift != 0.0 {
        let (x, y) = if placement.side.is_vertical() {
            (shift, 0.0)
        } else {
            (0.0, shift)
        };
        surface.offset(x, y)
    } else {
        surface
    };
    surface
        .anchored_round()
        .transition(transition(motion, placement.side, placement.align))
}

/// A full-window press target under an open popup: a press outside closes it.
pub fn scrim<S: 'static>(key: impl Into<String>, close: impl Fn(&mut S) + 'static) -> El {
    el().key(key)
        .role("none")
        .absolute(0.0, 0.0)
        .w_full()
        .h_full()
        .on_press(close)
}

/// The window's content with its overlay layer on top: `base` fills the window and each
/// layer element paints above it in order.
pub fn root(base: El, layer: impl IntoIterator<Item = El>) -> El {
    el().key("overlay-root")
        .w_full()
        .h_full()
        .child(base.w_full().h_full())
        .children(layer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion::ease;
    use std::time::Duration;

    #[test]
    fn springs_match_the_tween_at_ninety_percent() {
        let s = spring(Tween::new(Duration::from_millis(150), ease::OUT));
        assert!(s.duration > 0.05 && s.duration < 0.15, "{s:?}");
        let linear = spring(Tween::new(Duration::from_millis(100), ease::LINEAR));
        assert!((linear.duration - 0.09 * std::f32::consts::TAU / 3.8902).abs() < 1e-3);
    }
}
