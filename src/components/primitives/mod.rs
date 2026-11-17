//! The primitive layer: Notra's marketing button, typography, shimmer and status
//! badges, plus the shadcn primitives the blocks reuse (badge, kbd, spinner,
//! skeleton, avatar, separator, collapsible, checkbox), styled as they render on
//! ui.usenotra.com.

use std::time::Duration;

use crate::motion::{CubicBezier, Loop, Tween, ease};
use crate::theme::{Color, Theme};

pub mod avatar;
pub mod badge;
pub mod checkbox;
pub mod collapsible;
pub mod kbd;
pub mod marketing_button;
pub mod separator;
pub mod shimmer;
pub mod skeleton;
pub mod spinner;
pub mod status_badge;
pub mod typography;

/// Tailwind's `ease-out` as the examples' stylesheet defines it.
pub const TW_EASE_OUT: CubicBezier = CubicBezier::new(0.0, 0.0, 0.2, 1.0);

/// `transition-colors` and `transition-all` at Tailwind's default 150 ms.
pub const COLOR_TRANSITION: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);

/// `animate-spin`: one turn a second at constant speed.
pub const SPIN: Loop = Loop::new(Duration::from_secs(1), ease::LINEAR);

/// The rotation of a spinning glyph `elapsed` into [`SPIN`], in radians.
pub fn spin_angle(elapsed: Duration) -> f32 {
    SPIN.phase(elapsed) * std::f32::consts::TAU
}

/// `focus-visible:ring-3 ring-ring/50`: the focus ring's width and colour.
pub fn focus_ring(theme: &Theme) -> (f32, Color) {
    (3.0, theme.ring.fade(0.5))
}

/// A CSS `oklab(l a b)` colour.
pub fn oklab(l: f32, a: f32, b: f32) -> Color {
    Color::oklch(l, a.hypot(b), b.atan2(a).to_degrees())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spin_turns_once_a_second() {
        assert_eq!(spin_angle(Duration::ZERO), 0.0);
        let half = spin_angle(Duration::from_millis(500));
        assert!((half - std::f32::consts::PI).abs() < 1e-3);
    }

    #[test]
    fn oklab_white_is_white() {
        let c = oklab(1.0, 0.0, 0.0);
        assert!(c.r > 0.999 && c.g > 0.999 && c.b > 0.999);
    }
}
