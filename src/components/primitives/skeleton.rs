//! shadcn's `Skeleton`: a muted block that pulses (`animate-pulse rounded-md bg-muted`).

use std::time::Duration;

use crate::motion::{Loop, ease, keyframes};
use crate::theme::{Color, Theme};

/// `rounded-md`.
pub const RADIUS: f32 = 6.0;
/// `animate-pulse`: `pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite`.
pub const PULSE: Loop = Loop::new(Duration::from_secs(2), ease::PULSE);
/// The pulse's keyframes, `(offset, opacity)`: `50% { opacity: .5 }`.
pub const FRAMES: [(f32, f32); 3] = [(0.0, 1.0), (0.5, 0.5), (1.0, 1.0)];

/// The block's fill.
pub fn fill(theme: &Theme) -> Color {
    theme.muted
}

/// Opacity `elapsed` into the pulse.
pub fn opacity(elapsed: Duration) -> f32 {
    keyframes(&FRAMES, PULSE.easing, PULSE.phase(elapsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulse_dips_to_half_at_one_second() {
        assert_eq!(opacity(Duration::ZERO), 1.0);
        assert!((opacity(Duration::from_secs(1)) - 0.5).abs() < 1e-3);
        assert!(opacity(Duration::from_millis(1990)) > 0.99);
    }
}
