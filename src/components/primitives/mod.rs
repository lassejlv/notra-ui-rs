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

/// A CSS `oklab(l a b)` colour, brought into sRGB as browsers paint it: chroma is
/// reduced at constant lightness and hue until the colour fits (CSS Color 4 gamut
/// mapping), rather than clipping each channel.
pub fn oklab(l: f32, a: f32, b: f32) -> Color {
    let hue = b.atan2(a);
    let (sin, cos) = hue.sin_cos();
    let fits = |c: f32| in_srgb_gamut(l, c * cos, c * sin);
    let mut chroma = a.hypot(b);
    if !fits(chroma) {
        let (mut lo, mut hi) = (0.0_f32, chroma);
        for _ in 0..24 {
            let mid = (lo + hi) / 2.0;
            if fits(mid) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        chroma = lo;
    }
    Color::oklch(l, chroma, hue.to_degrees())
}

fn in_srgb_gamut(l: f32, a: f32, b: f32) -> bool {
    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;
    let (l3, m3, s3) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    [
        4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_94 * s3,
        -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_38 * s3,
        -0.004_196_086_3 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3,
    ]
    .iter()
    .all(|c| (-1e-4..=1.0001).contains(c))
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
