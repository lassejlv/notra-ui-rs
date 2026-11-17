//! Notra's motion vocabulary: CSS easing curves, durations and keyframes.
//!
//! Both front ends sample these so a GPUI button and a Rune button move on the
//! same curve the browser uses.

use std::time::Duration;

/// A CSS `cubic-bezier(x1, y1, x2, y2)` timing function.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl CubicBezier {
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Eased progress for linear progress `t` in `0.0..=1.0`.
    pub fn ease(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        if t == 0.0 || t == 1.0 {
            return t;
        }
        let x = |s: f32| bezier(s, self.x1, self.x2);
        let dx = |s: f32| bezier_slope(s, self.x1, self.x2);
        let mut s = t;
        for _ in 0..8 {
            let err = x(s) - t;
            if err.abs() < 1e-6 {
                return bezier(s, self.y1, self.y2);
            }
            let d = dx(s);
            if d.abs() < 1e-6 {
                break;
            }
            s -= err / d;
        }
        let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
        s = t;
        for _ in 0..32 {
            let v = x(s);
            if (v - t).abs() < 1e-6 {
                break;
            }
            if v < t {
                lo = s;
            } else {
                hi = s;
            }
            s = (lo + hi) / 2.0;
        }
        bezier(s, self.y1, self.y2)
    }
}

fn bezier(s: f32, p1: f32, p2: f32) -> f32 {
    let u = 1.0 - s;
    3.0 * u * u * s * p1 + 3.0 * u * s * s * p2 + s * s * s
}

fn bezier_slope(s: f32, p1: f32, p2: f32) -> f32 {
    let u = 1.0 - s;
    3.0 * u * u * p1 + 6.0 * u * s * (p2 - p1) + 3.0 * s * s * (1.0 - p2)
}

/// Named curves from the Notra stylesheet.
pub mod ease {
    use super::CubicBezier;

    pub const LINEAR: CubicBezier = CubicBezier::new(0.0, 0.0, 1.0, 1.0);
    /// CSS `ease`.
    pub const EASE: CubicBezier = CubicBezier::new(0.25, 0.1, 0.25, 1.0);
    /// CSS `ease-in-out` (keyword).
    pub const CSS_EASE_IN_OUT: CubicBezier = CubicBezier::new(0.42, 0.0, 0.58, 1.0);
    /// CSS `ease-out` (keyword).
    pub const CSS_EASE_OUT: CubicBezier = CubicBezier::new(0.0, 0.0, 0.58, 1.0);
    /// `--ease-in`.
    pub const IN: CubicBezier = CubicBezier::new(0.4, 0.0, 1.0, 1.0);
    /// `--ease-out` (and `--ease-emphasized`), Notra's default exit-free curve.
    pub const OUT: CubicBezier = CubicBezier::new(0.23, 1.0, 0.32, 1.0);
    /// `--ease-in-out` and Tailwind's default transition curve.
    pub const IN_OUT: CubicBezier = CubicBezier::new(0.4, 0.0, 0.2, 1.0);
    /// `--ease-emphasized-in-out`.
    pub const EMPHASIZED_IN_OUT: CubicBezier = CubicBezier::new(0.65, 0.0, 0.35, 1.0);
    /// `--sidebar-push-ease`.
    pub const SIDEBAR_PUSH: CubicBezier = CubicBezier::new(0.25, 0.5, 0.15, 1.0);
    /// The progress fill of a button.
    pub const PROGRESS: CubicBezier = CubicBezier::new(0.22, 1.0, 0.36, 1.0);
    /// Tailwind's `animate-pulse`.
    pub const PULSE: CubicBezier = CubicBezier::new(0.4, 0.0, 0.6, 1.0);
}

/// Durations from the stylesheet.
pub mod duration {
    use std::time::Duration;

    /// Tailwind's default `transition` duration and `--transition-duration-fast`.
    pub const FAST: Duration = Duration::from_millis(150);
    /// `--transition-duration-normal`.
    pub const NORMAL: Duration = Duration::from_millis(200);
    pub const SIDEBAR_PUSH: Duration = Duration::from_millis(400);
}

/// A one-shot CSS animation: delay, duration and curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tween {
    pub delay: Duration,
    pub duration: Duration,
    pub easing: CubicBezier,
}

impl Tween {
    pub const fn new(duration: Duration, easing: CubicBezier) -> Self {
        Self {
            delay: Duration::ZERO,
            duration,
            easing,
        }
    }

    pub const fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Total time until the tween lands.
    pub fn total(&self) -> Duration {
        self.delay + self.duration
    }

    /// Eased progress `elapsed` after the animation started (`animation-fill-mode: both`).
    pub fn progress(&self, elapsed: Duration) -> f32 {
        let Some(active) = elapsed.checked_sub(self.delay) else {
            return 0.0;
        };
        if self.duration.is_zero() {
            return 1.0;
        }
        self.easing
            .ease(active.as_secs_f32() / self.duration.as_secs_f32())
    }

    pub fn is_done(&self, elapsed: Duration) -> bool {
        elapsed >= self.total()
    }
}

/// An infinite CSS animation: period, phase delay and per-cycle curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Loop {
    pub period: Duration,
    pub delay: Duration,
    pub easing: CubicBezier,
}

impl Loop {
    pub const fn new(period: Duration, easing: CubicBezier) -> Self {
        Self {
            period,
            delay: Duration::ZERO,
            easing,
        }
    }

    pub const fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Linear position within the current cycle, `0.0..1.0`, before the curve.
    pub fn phase(&self, elapsed: Duration) -> f32 {
        let Some(active) = elapsed.checked_sub(self.delay) else {
            return 0.0;
        };
        let period = self.period.as_secs_f32();
        (active.as_secs_f32() % period) / period
    }
}

/// A value that eases toward its target over a [`Tween`], restarting from where it
/// is when retargeted (CSS `transition` semantics). Immediate-mode renderers such as
/// GPUI keep one per animated property and sample it every frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Animated {
    from: f32,
    to: f32,
    started: Option<std::time::Instant>,
    tween: Tween,
}

impl Animated {
    pub fn new(value: f32, tween: Tween) -> Self {
        Self {
            from: value,
            to: value,
            started: None,
            tween,
        }
    }

    pub fn target(&self) -> f32 {
        self.to
    }

    /// Retargets at `now`, starting from the value shown at `now`.
    pub fn set(&mut self, target: f32, now: std::time::Instant) {
        if target == self.to {
            return;
        }
        self.from = self.value(now);
        self.to = target;
        self.started = Some(now);
    }

    /// Jumps to `value` without animating (reduced motion, first frame).
    pub fn snap(&mut self, value: f32) {
        self.from = value;
        self.to = value;
        self.started = None;
    }

    pub fn value(&self, now: std::time::Instant) -> f32 {
        match self.started {
            Some(start) => lerp(self.from, self.to, self.tween.progress(now - start)),
            None => self.to,
        }
    }

    pub fn is_animating(&self, now: std::time::Instant) -> bool {
        self.started
            .is_some_and(|start| !self.tween.is_done(now - start))
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Samples piecewise keyframes `(offset, value)` with a per-segment curve, as CSS
/// applies `animation-timing-function` between each pair of keyframes.
pub fn keyframes(frames: &[(f32, f32)], easing: CubicBezier, t: f32) -> f32 {
    let Some(&(first_at, first)) = frames.first() else {
        return 0.0;
    };
    if t <= first_at {
        return first;
    }
    for pair in frames.windows(2) {
        let ((a_at, a), (b_at, b)) = (pair[0], pair[1]);
        if t <= b_at {
            let local = if b_at > a_at {
                (t - a_at) / (b_at - a_at)
            } else {
                1.0
            };
            return lerp(a, b, easing.ease(local));
        }
    }
    frames.last().map_or(0.0, |&(_, v)| v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curves_hit_their_ends_and_stay_monotonic() {
        for curve in [ease::OUT, ease::IN, ease::IN_OUT, ease::EMPHASIZED_IN_OUT] {
            assert_eq!(curve.ease(0.0), 0.0);
            assert_eq!(curve.ease(1.0), 1.0);
            let mut last = 0.0;
            for i in 1..=100 {
                let v = curve.ease(i as f32 / 100.0);
                assert!(v + 1e-4 >= last);
                last = v;
            }
        }
    }

    #[test]
    fn ease_out_front_loads_progress() {
        assert!(ease::OUT.ease(0.25) > 0.75);
    }

    #[test]
    fn tween_honours_its_delay() {
        let t =
            Tween::new(Duration::from_millis(200), ease::LINEAR).delay(Duration::from_millis(100));
        assert_eq!(t.progress(Duration::from_millis(50)), 0.0);
        assert!((t.progress(Duration::from_millis(200)) - 0.5).abs() < 1e-3);
        assert_eq!(t.progress(Duration::from_millis(400)), 1.0);
    }

    #[test]
    fn keyframes_interpolate_each_segment() {
        let frames = [(0.0, 0.45), (0.3, 1.0), (0.6, 0.45), (1.0, 0.45)];
        assert!((keyframes(&frames, ease::LINEAR, 0.15) - 0.725).abs() < 1e-3);
        assert_eq!(keyframes(&frames, ease::LINEAR, 0.8), 0.45);
    }
}
