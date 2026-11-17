//! The step slider: a pill track that snaps to fixed values, filled with a violet
//! dotted indicator that a bright band sweeps across on every step.

use std::time::Duration;

use crate::motion::{CubicBezier, Tween, ease};
use crate::theme::{Color, Shadow, Theme};

/// Box metrics measured on ui.usenotra.com, in logical pixels.
pub mod metrics {
    /// The pill around the control (`h-10`).
    pub const TRACK_HEIGHT: f32 = 40.0;
    /// `p-1` between the pill and the control.
    pub const TRACK_PADDING: f32 = 4.0;
    /// The thumb (`size-8`), also the control's height.
    pub const THUMB: f32 = 32.0;
    /// Half the thumb: stops and labels sit `1rem` in from each end.
    pub const STOP_INSET: f32 = 16.0;
    /// Stop dots (`size-1.5`).
    pub const STOP: f32 = 6.0;
    /// Space between the track and the labels (`gap-3`).
    pub const LABEL_GAP: f32 = 12.0;
    /// The label row (`h-4`, `mx-1`).
    pub const LABEL_HEIGHT: f32 = 16.0;
    pub const LABEL_INSET: f32 = 4.0;
    pub const LABEL_FONT_SIZE: f32 = 12.0;
    pub const LABEL_LINE_HEIGHT: f32 = 16.0;
    /// `tracking-[-0.01em]` at 12 px.
    pub const LABEL_TRACKING: f32 = -0.12;
    /// The dot grid's tile (`bg-size-[0.375rem]`).
    pub const DOT_TILE: f32 = 6.0;
    /// Resting dots: solid to 0.75 px, transparent by 1.25 px.
    pub const DOT_RADII: (f32, f32) = (0.75, 1.25);
    /// Sweep dots: solid to 0.9 px, transparent by 1.4 px.
    pub const SWEEP_DOT_RADII: (f32, f32) = (0.9, 1.4);
    /// The focus ring around the thumb (`ring-[0.1875rem]`).
    pub const FOCUS_RING: f32 = 3.0;
    /// Containers narrower than this (`@lg`) show only the major labels.
    pub const MAJOR_LABELS_BELOW: f32 = 512.0;
}

/// A stop within this ratio of the thumb is hidden under it.
pub const STOP_UNDER_THUMB: f32 = 0.01;

/// Thumb and indicator glide (`duration-300 ease-[cubic-bezier(0.22,1,0.36,1)]`).
pub const GLIDE: Tween = Tween::new(Duration::from_millis(300), ease::PROGRESS);
/// Stop dots change colour (`transition-colors duration-300`).
pub const STOP_COLOR: Tween = Tween::new(Duration::from_millis(300), ease::IN_OUT);
/// Labels change colour on hover (`duration-100`).
pub const LABEL_COLOR: Tween = Tween::new(Duration::from_millis(100), ease::IN_OUT);
/// The focus ring fades in (`transition-[box-shadow] duration-200 ease-out`).
pub const FOCUS: Tween = Tween::new(
    Duration::from_millis(200),
    CubicBezier::new(0.0, 0.0, 0.2, 1.0),
);
/// One sweep of the bright band (`duration: 900, easing: "ease-out"`).
pub const SWEEP: Tween = Tween::new(Duration::from_millis(900), ease::CSS_EASE_OUT);
/// The band's `mask-position`, in percent, at the start and end of a sweep.
pub const SWEEP_FROM: f32 = -60.0;
pub const SWEEP_TO: f32 = 160.0;
/// The band's width as a share of the indicator (`mask-size: 30%`).
pub const SWEEP_BAND: f32 = 0.3;
/// Progress at which the band has passed the end of the fill; a step before it
/// lets the running sweep carry on instead of starting another.
pub const SWEEP_PAST_FILL: f32 = (100.0 - SWEEP_FROM) / (SWEEP_TO - SWEEP_FROM);

/// The step nearest `value` (the first on a tie).
pub fn nearest_step_index(steps: &[f64], value: f64) -> usize {
    let mut nearest = 0;
    for (index, step) in steps.iter().enumerate() {
        if (step - value).abs() < (steps[nearest] - value).abs() {
            nearest = index;
        }
    }
    nearest
}

/// Where step `index` of `count` sits along the track, `0.0..=1.0`.
pub fn step_ratio(index: usize, count: usize) -> f32 {
    if count > 1 {
        index as f32 / (count - 1) as f32
    } else {
        0.0
    }
}

/// Horizontal geometry of the control for one width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Track {
    /// The control's width: the pill minus its padding.
    pub width: f32,
}

impl Track {
    pub fn new(width: f32) -> Self {
        Self { width }
    }

    /// Distance the thumb travels (`thumbAlignment="edge"`).
    pub fn travel(&self) -> f32 {
        (self.width - metrics::THUMB).max(0.0)
    }

    /// The thumb's left edge at `ratio`.
    pub fn thumb_left(&self, ratio: f32) -> f32 {
        ratio * self.travel()
    }

    /// The indicator's width at `ratio`: it ends under the thumb's right edge.
    pub fn fill_width(&self, ratio: f32) -> f32 {
        self.thumb_left(ratio) + metrics::THUMB
    }

    /// Centre of a stop or label at `ratio` (`calc(1rem + ratio * (100% - 2rem))`).
    pub fn stop_center(&self, ratio: f32) -> f32 {
        metrics::STOP_INSET + self.thumb_left(ratio)
    }

    /// The step under a pointer at `x`, as Base UI maps an edge-aligned thumb.
    pub fn index_at(&self, x: f32, count: usize) -> usize {
        if count < 2 || self.travel() <= 0.0 {
            return 0;
        }
        let ratio = ((x - metrics::STOP_INSET) / self.travel()).clamp(0.0, 1.0);
        (ratio * (count - 1) as f32).round() as usize
    }
}

/// A key the focused thumb handles, as Base UI's slider does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliderKey {
    /// Arrow right or up.
    Increase,
    /// Arrow left or down.
    Decrease,
    /// Page up, or shift with an increasing arrow: `largeStep` (10) stops.
    IncreaseLarge,
    DecreaseLarge,
    Home,
    End,
}

/// Base UI's default `largeStep`.
pub const LARGE_STEP: usize = 10;

impl SliderKey {
    /// The step index after this key, from `index` among `count` steps.
    pub fn apply(self, index: usize, count: usize) -> usize {
        let last = count.saturating_sub(1);
        match self {
            SliderKey::Increase => (index + 1).min(last),
            SliderKey::Decrease => index.saturating_sub(1),
            SliderKey::IncreaseLarge => (index + LARGE_STEP).min(last),
            SliderKey::DecreaseLarge => index.saturating_sub(LARGE_STEP),
            SliderKey::Home => 0,
            SliderKey::End => last,
        }
    }
}

/// The two sweep bands: a step starts the idle one, unless the running sweep
/// has not yet passed the end of the fill.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sweeps {
    started: [Option<f64>; 2],
    /// Bumped on every new sweep; renderers that replay by key use it.
    serial: [u32; 2],
    active: usize,
}

impl Sweeps {
    /// A step landed at `now` (seconds). Returns whether a sweep started.
    pub fn step(&mut self, now: f64) -> bool {
        let running = self.started[self.active]
            .map(|start| now - start)
            .filter(|elapsed| *elapsed < SWEEP.duration.as_secs_f64());
        if running.is_some_and(|elapsed| sweep_progress(elapsed) < SWEEP_PAST_FILL) {
            return false;
        }
        let next = if running.is_some() {
            1 - self.active
        } else {
            self.active
        };
        self.started[next] = Some(now);
        self.serial[next] = self.serial[next].wrapping_add(1);
        self.active = next;
        true
    }

    /// Each band's elapsed time at `now`, while it sweeps.
    pub fn running(&self, now: f64) -> [Option<f64>; 2] {
        self.started.map(|start| {
            start
                .map(|start| now - start)
                .filter(|elapsed| *elapsed >= 0.0 && *elapsed < SWEEP.duration.as_secs_f64())
        })
    }

    /// The replay serial of each band.
    pub fn serials(&self) -> [u32; 2] {
        self.serial
    }

    /// Whether either band is still sweeping.
    pub fn is_animating(&self, now: f64) -> bool {
        self.running(now).iter().any(Option::is_some)
    }
}

/// Eased progress of a sweep `elapsed` seconds in.
pub fn sweep_progress(elapsed: f64) -> f32 {
    SWEEP.progress(Duration::from_secs_f64(elapsed.max(0.0)))
}

/// Left edge of the band over an indicator `width` wide at sweep `progress`:
/// `mask-position: p% 0` places it at `p% * (width - band)`.
pub fn band_left(width: f32, progress: f32) -> f32 {
    let position = SWEEP_FROM + (SWEEP_TO - SWEEP_FROM) * progress;
    width * (1.0 - SWEEP_BAND) * position / 100.0
}

/// The band's mask at `x` across a band `band` wide: clear at the edges,
/// opaque at the middle (`linear-gradient(90deg, transparent, #000 50%, transparent)`).
pub fn band_mask(x: f32, band: f32) -> f32 {
    if band <= 0.0 || x <= 0.0 || x >= band {
        return 0.0;
    }
    1.0 - (2.0 * x / band - 1.0).abs()
}

/// Dot centres of a `width` × `height` grid tiled from its centre
/// (`bg-size-[0.375rem] bg-center`).
pub fn dot_centers(width: f32, height: f32) -> impl Iterator<Item = [f32; 2]> {
    let axis = |len: f32| {
        let mid = len / 2.0;
        let reach = (mid / metrics::DOT_TILE).ceil() as i32;
        (-reach..=reach)
            .map(move |k| mid + k as f32 * metrics::DOT_TILE)
            .filter(move |c| *c > -metrics::DOT_TILE && *c < len + metrics::DOT_TILE)
    };
    axis(height).flat_map(move |y| axis(width).map(move |x| [x, y]))
}

/// Colours for one theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepSliderSkin {
    /// `bg-[#F1F1F2] dark:bg-white/[0.06]`.
    pub track: Color,
    pub track_shadow: Shadow,
    /// `from-violet-500 to-violet-600`.
    pub fill: (Color, Color),
    pub fill_highlight: Shadow,
    pub dot: Color,
    pub sweep_dot: Color,
    /// Stops behind the thumb (`bg-white/70`).
    pub stop_filled: Color,
    /// Stops ahead of it.
    pub stop: Color,
    /// `from-white to-[#F2F2F2]`.
    pub thumb: (Color, Color),
    pub thumb_shadows: [Shadow; 2],
    /// `ring-violet-500/40`.
    pub focus_ring: Color,
    pub label: Color,
    /// The current step's label, and any label under the pointer.
    pub label_active: Color,
}

impl StepSliderSkin {
    pub fn new(theme: &Theme) -> Self {
        let dark = theme.is_dark();
        let violet_500 = Color::oklch(0.606, 0.25, 292.717);
        Self {
            track: if dark {
                Color::WHITE.alpha(0.06)
            } else {
                Color::rgb(0xf1f1f2)
            },
            track_shadow: Shadow::inset(1.0, 2.0, 0.0, Color::rgba(0x1e1e_1e0f)),
            fill: (violet_500, Color::oklch(0.541, 0.281, 293.009)),
            fill_highlight: Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffff_ff33)),
            dot: Color::rgba(0xffff_ff2e),
            sweep_dot: Color::rgba(0xffff_ffe6),
            stop_filled: Color::WHITE.alpha(0.7),
            stop: if dark {
                Color::WHITE.alpha(0.3)
            } else {
                Color::rgba(0x1e1e_1e33)
            },
            thumb: (Color::WHITE, Color::rgb(0xf2f2f2)),
            thumb_shadows: [
                Shadow::drop(1.0, 4.0, 0.0, Color::rgba(0x2828_2840)),
                Shadow::drop(0.0, 0.0, 1.0, Color::rgba(0x1e1e_1e0d)),
            ],
            focus_ring: violet_500.alpha(0.4),
            label: if dark {
                Color::WHITE.alpha(0.45)
            } else {
                Color::rgba(0x1e1e_1e80)
            },
            label_active: if dark {
                Color::WHITE
            } else {
                Color::rgb(0x1e1e1e)
            },
        }
    }
}

/// The registry example: monthly prompts.
pub mod demo {
    /// `PROMPT_STEPS`.
    pub const STEPS: [f64; 9] = [5.0, 10.0, 25.0, 50.0, 100.0, 150.0, 250.0, 500.0, 1000.0];
    /// Labels that stay in a narrow container.
    pub const MAJOR: [f64; 5] = [5.0, 25.0, 100.0, 250.0, 1000.0];
    pub const DEFAULT: f64 = 10.0;
    pub const LABEL: &str = "Prompts";

    /// `formatPrompts`: thousands as `1K`.
    pub fn format(value: f64) -> String {
        if value >= 1000.0 {
            format!("{}K", value / 1000.0)
        } else {
            format!("{value}")
        }
    }

    /// `getValueText`.
    pub fn value_text(value: f64) -> String {
        format!("{value} prompts")
    }

    /// The example's header: `text-lg font-medium` and `text-4xl tabular-nums`,
    /// `gap-5` above the slider, in a `max-w-3xl p-6` column.
    pub const TITLE_SIZE: (f32, f32) = (18.0, 28.0);
    pub const VALUE_SIZE: (f32, f32) = (36.0, 40.0);
    pub const GAP: f32 = 20.0;
    pub const MAX_WIDTH: f32 = 768.0;
    pub const PADDING: f32 = 24.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_step_prefers_the_first_on_ties() {
        let steps = [5.0, 10.0, 25.0];
        assert_eq!(nearest_step_index(&steps, 7.5), 0);
        assert_eq!(nearest_step_index(&steps, 20.0), 2);
    }

    #[test]
    fn geometry_matches_the_measured_control() {
        let track = Track::new(712.0);
        let ratio = step_ratio(1, 9);
        assert!((track.thumb_left(ratio) - 85.0).abs() < 1e-3);
        assert!((track.fill_width(ratio) - 117.0).abs() < 1e-3);
        assert!((track.stop_center(0.0) - 16.0).abs() < 1e-3);
        assert_eq!(track.index_at(101.0, 9), 1);
        assert_eq!(track.index_at(-40.0, 9), 0);
        assert_eq!(track.index_at(900.0, 9), 8);
    }

    #[test]
    fn keys_clamp_to_the_ends() {
        assert_eq!(SliderKey::Increase.apply(8, 9), 8);
        assert_eq!(SliderKey::DecreaseLarge.apply(3, 9), 0);
        assert_eq!(SliderKey::End.apply(0, 9), 8);
    }

    #[test]
    fn a_step_mid_sweep_waits_until_the_band_passes_the_fill() {
        let mut sweeps = Sweeps::default();
        assert!(sweeps.step(0.0));
        assert!(!sweeps.step(0.05));
        let late = (5..90)
            .map(|ms| f64::from(ms) / 100.0)
            .find(|t| sweep_progress(*t) >= SWEEP_PAST_FILL)
            .unwrap_or(0.9);
        assert!(sweeps.step(late));
        let running = sweeps.running(late + 0.01);
        assert!(running[0].is_some() && running[1].is_some());
    }

    #[test]
    fn band_spans_from_before_to_past_the_fill() {
        assert!(band_left(100.0, 0.0) < 0.0);
        assert!(band_left(100.0, 1.0) > 100.0);
        assert!((band_left(100.0, SWEEP_PAST_FILL) - 70.0).abs() < 1e-3);
        assert_eq!(band_mask(15.0, 30.0), 1.0);
        assert_eq!(band_mask(0.0, 30.0), 0.0);
    }

    #[test]
    fn dots_tile_from_the_centre() {
        let rows: Vec<f32> = dot_centers(12.0, 32.0)
            .filter(|c| (c[0] - 6.0).abs() < 1e-3)
            .map(|c| c[1])
            .collect();
        assert!(rows.contains(&16.0) && rows.contains(&4.0) && rows.contains(&28.0));
    }

    #[test]
    fn prompts_format_thousands() {
        assert_eq!(demo::format(1000.0), "1K");
        assert_eq!(demo::format(250.0), "250");
    }
}
