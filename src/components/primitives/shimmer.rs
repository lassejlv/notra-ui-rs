//! Notra's `Shimmer`: a highlight band that sweeps across text.
//!
//! The text paints two backgrounds clipped to its glyphs: `currentColor`, and over it
//! a 250 %-wide gradient whose middle is the highlight, fading to transparent
//! `spread` either side, moved from `background-position: 100%` to `0%`. The band's
//! centre therefore travels from `-0.25` to `1.25` of the text's width each cycle.

use std::time::Duration;

use crate::theme::{Color, Theme};

/// One sweep (`--shimmer-duration`, default 1.7 s), linear and infinite.
pub const DURATION: Duration = Duration::from_millis(1700);
/// Half the band's width in `ch` (`spread = 4`).
pub const SPREAD_CH: f32 = 4.0;
/// Where the band's centre starts and ends, in fractions of the text's width.
pub const START: f32 = -0.25;
pub const END: f32 = 1.25;
/// Width of Inter's `0` per em: the `ch` unit for the bundled face.
pub const CH_EM: f32 = 0.631;

/// A sweep's settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShimmerSpec {
    pub duration: Duration,
    pub spread_ch: f32,
}

impl Default for ShimmerSpec {
    fn default() -> Self {
        Self {
            duration: DURATION,
            spread_ch: SPREAD_CH,
        }
    }
}

impl ShimmerSpec {
    /// Half the band's width in px for text at `font_size`.
    pub fn spread_px(&self, font_size: f32) -> f32 {
        self.spread_ch * CH_EM * font_size
    }

    /// Linear position within the sweep, `0.0..1.0`.
    pub fn phase(&self, elapsed: Duration) -> f32 {
        let period = self.duration.as_secs_f32();
        if period <= 0.0 {
            return 0.0;
        }
        (elapsed.as_secs_f32() % period) / period
    }

    /// The band's centre, in px from the text's left edge, for text `width` wide.
    pub fn center(&self, elapsed: Duration, width: f32) -> f32 {
        width * (START + (END - START) * self.phase(elapsed))
    }
}

/// The highlight's strength at `x`, 1 at the band's centre and 0 `spread` away.
pub fn intensity(x: f32, center: f32, spread: f32) -> f32 {
    if spread <= 0.0 {
        return 0.0;
    }
    (1.0 - (x - center).abs() / spread).max(0.0)
}

/// The colour text paints at a point with `intensity`: the highlight, faded by it, over
/// the base colour (gradients interpolate premultiplied, so the fade is linear in alpha).
pub fn paint(base: Color, highlight: Color, intensity: f32) -> Color {
    if intensity <= 0.0 {
        return base;
    }
    highlight.fade(intensity.min(1.0)).over(base)
}

/// `--shimmer-highlight`, defaulting to the foreground.
pub fn highlight(theme: &Theme) -> Color {
    theme.foreground
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_sweeps_past_both_edges() {
        let spec = ShimmerSpec::default();
        assert_eq!(spec.center(Duration::ZERO, 100.0), -25.0);
        let late = spec.center(Duration::from_millis(1699), 100.0);
        assert!(late > 124.0);
    }

    #[test]
    fn band_is_triangular() {
        assert_eq!(intensity(10.0, 10.0, 4.0), 1.0);
        assert_eq!(intensity(12.0, 10.0, 4.0), 0.5);
        assert_eq!(intensity(20.0, 10.0, 4.0), 0.0);
    }

    #[test]
    fn full_intensity_paints_the_highlight() {
        let c = paint(Color::rgb(0x737373), Color::rgb(0x171717), 1.0);
        assert!((c.r - Color::rgb(0x171717).r).abs() < 1e-6);
    }
}
