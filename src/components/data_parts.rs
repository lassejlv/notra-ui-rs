//! Small controls the data surfaces (card, table, code block, package command)
//! share: spinner, shimmer, checkbox, badge, tooltip, select, copy button, tabs,
//! skeleton and the empty state. They match the look of the Notra and shadcn
//! parts the React sources import, at the measurements of the live examples.

use std::time::{Duration, Instant};

use crate::motion::{Loop, Tween, ease};
use crate::theme::{Color, Shadow, Theme};

/// `animate-spin` on lucide's `loader-circle`.
pub mod spinner {
    use super::*;

    pub const SIZE: f32 = 16.0;
    /// One turn, linear.
    pub const TURN: Loop = Loop::new(Duration::from_secs(1), ease::LINEAR);
}

/// Notra's `Shimmer`: a bright band sweeping across muted text.
pub mod shimmer {
    use super::*;

    /// `shimmer 1.7s linear infinite`.
    pub const SWEEP: Loop = Loop::new(Duration::from_millis(1700), ease::LINEAR);
    /// The band's half width, `4ch`, in em of the text size.
    pub const SPREAD_EM: f32 = 2.288;

    /// Centre of the band, in px from the text's left edge, at `phase` of a sweep.
    ///
    /// The gradient is 250 % of the text wide and its background position runs from
    /// 100 % to 0 %, so the band enters a quarter width left of the text and leaves a
    /// quarter width past its end.
    pub fn center(phase: f32, width: f32) -> f32 {
        -0.25 * width + 1.5 * width * phase
    }

    /// How much of the highlight shows at `x` (0 outside the band, 1 at its centre).
    pub fn amount(x: f32, center: f32, spread: f32) -> f32 {
        if spread <= 0.0 {
            return 0.0;
        }
        (1.0 - (x - center).abs() / spread).clamp(0.0, 1.0)
    }

    /// The colour of text at `x`: `base` under a band of `highlight`.
    pub fn color_at(base: Color, highlight: Color, x: f32, center: f32, spread: f32) -> Color {
        let t = amount(x, center, spread);
        if t <= 0.0 {
            base
        } else {
            highlight.fade(t).over(base)
        }
    }
}

/// The base-ui checkbox (`size-4 rounded-[4px] border`).
pub mod checkbox {
    use super::*;

    pub const SIZE: f32 = 16.0;
    pub const RADIUS: f32 = 4.0;
    pub const ICON: f32 = 14.0;
    /// `transition-colors`.
    pub const TRANSITION: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);

    /// Off, on, or mixed (some rows selected).
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub enum CheckState {
        #[default]
        Off,
        On,
        Mixed,
    }

    impl CheckState {
        pub fn new(all: bool, some: bool) -> Self {
            match (all, some) {
                (true, _) => CheckState::On,
                (false, true) => CheckState::Mixed,
                _ => CheckState::Off,
            }
        }
    }

    /// Resolved paint.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct CheckboxSkin {
        pub fill: Color,
        pub border: Color,
        /// The check's colour; `None` hides it.
        pub mark: Option<Color>,
    }

    impl CheckboxSkin {
        /// `ink` is the inherited text colour, which tints the mixed state's check.
        pub fn new(theme: &Theme, state: CheckState, ink: Color) -> Self {
            match state {
                CheckState::On => Self {
                    fill: theme.primary,
                    border: theme.primary,
                    mark: Some(theme.primary_foreground),
                },
                CheckState::Off | CheckState::Mixed => Self {
                    fill: if theme.is_dark() {
                        theme.input.fade(0.3)
                    } else {
                        Color::TRANSPARENT
                    },
                    border: theme.input,
                    mark: (state == CheckState::Mixed).then_some(ink),
                },
            }
        }
    }
}

/// shadcn's badge (`h-5 rounded-4xl px-2 text-xs font-medium`).
pub mod badge {
    use super::*;

    pub const HEIGHT: f32 = 20.0;
    pub const PADDING_X: f32 = 8.0;
    pub const RADIUS: f32 = 32.0;
    pub const FONT_SIZE: f32 = 12.0;
    pub const LINE_HEIGHT: f32 = 16.0;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub enum BadgeVariant {
        #[default]
        Secondary,
        Outline,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct BadgeSkin {
        pub fill: Color,
        pub border: Color,
        pub text: Color,
    }

    impl BadgeSkin {
        pub fn new(theme: &Theme, variant: BadgeVariant) -> Self {
            match variant {
                BadgeVariant::Secondary => Self {
                    fill: theme.secondary,
                    border: Color::TRANSPARENT,
                    text: theme.secondary_foreground,
                },
                BadgeVariant::Outline => Self {
                    fill: Color::TRANSPARENT,
                    border: theme.border,
                    text: theme.foreground,
                },
            }
        }
    }
}

/// Notra's tooltip: an inverted pill with a gradient and a soft drop.
pub mod tooltip {
    use super::*;

    pub const RADIUS: f32 = 10.0;
    pub const PADDING_X: f32 = 10.0;
    pub const PADDING_Y: f32 = 4.0;
    pub const FONT_SIZE: f32 = 12.0;
    pub const LINE_HEIGHT: f32 = 16.0;
    /// `max-w-64` on the hint content.
    pub const MAX_WIDTH: f32 = 256.0;
    /// `sideOffset`.
    pub const GAP: f32 = 4.0;
    pub const ENTER: Tween = Tween::new(Duration::from_millis(150), ease::OUT);
    pub const EXIT: Tween = Tween::new(Duration::from_millis(100), ease::OUT);
    /// The popup grows from this scale.
    pub const FROM_SCALE: f32 = 0.97;
    pub const SHADOWS: [Shadow; 3] = [
        Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffff38)),
        Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x00000029)),
        Shadow::drop(6.0, 16.0, -4.0, Color::rgba(0x0000003d)),
    ];

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct TooltipSkin {
        pub top: Color,
        pub bottom: Color,
        pub border: Color,
        pub text: Color,
    }

    impl TooltipSkin {
        pub fn new(theme: &Theme) -> Self {
            Self {
                top: theme.foreground.mix(theme.background, 0.10),
                bottom: theme.foreground,
                border: theme.foreground.fade(0.6),
                text: theme.background,
            }
        }
    }
}

/// Notra's select: a ghost trigger and the dropdown popup.
pub mod select {
    use super::*;

    pub const POPUP_RADIUS: f32 = 14.0;
    pub const POPUP_PADDING: f32 = 4.0;
    pub const POPUP_MIN_WIDTH: f32 = 144.0;
    /// `sideOffset`.
    pub const POPUP_GAP: f32 = 4.0;
    pub const ITEM_HEIGHT: f32 = 32.0;
    pub const ITEM_RADIUS: f32 = 10.0;
    pub const ITEM_PADDING_LEFT: f32 = 10.0;
    pub const ITEM_PADDING_RIGHT: f32 = 32.0;
    pub const ITEM_FONT_SIZE: f32 = 14.0;
    pub const ITEM_LINE_HEIGHT: f32 = 20.0;
    /// The check sits `right-2.5`.
    pub const INDICATOR_RIGHT: f32 = 10.0;
    pub const INDICATOR_SIZE: f32 = 16.0;
    /// The trigger's chevrons.
    pub const TRIGGER_ICON: f32 = 16.0;
    pub const ENTER: Tween = Tween::new(Duration::from_millis(150), ease::OUT);
    pub const EXIT: Tween = Tween::new(Duration::from_millis(100), ease::OUT);
    pub const FROM_SCALE: f32 = 0.97;
    /// Item highlight: `transition-colors duration-75`.
    pub const ITEM_TRANSITION: Tween = Tween::new(Duration::from_millis(75), ease::IN_OUT);

    #[derive(Clone, Debug, PartialEq)]
    pub struct PopupSkin {
        pub top: Color,
        pub bottom: Color,
        pub border: Color,
        pub text: Color,
        pub shadows: Vec<Shadow>,
        pub item_highlight: Color,
        pub item_highlight_text: Color,
    }

    impl PopupSkin {
        pub fn new(theme: &Theme) -> Self {
            if theme.is_dark() {
                Self {
                    top: theme.muted,
                    bottom: theme.shell,
                    border: theme.input,
                    text: theme.popover_foreground,
                    shadows: vec![
                        Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffff0f)),
                        Shadow::drop(2.0, 4.0, 0.0, Color::rgba(0x0000004d)),
                        Shadow::drop(16.0, 32.0, -8.0, Color::rgba(0x00000099)),
                    ],
                    item_highlight: theme.foreground.fade(0.07),
                    item_highlight_text: theme.foreground,
                }
            } else {
                Self {
                    top: theme.background,
                    bottom: theme.background.mix(theme.foreground, 0.025),
                    border: theme.border,
                    text: theme.popover_foreground,
                    shadows: vec![
                        Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffffcc)),
                        Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x0000000d)),
                        Shadow::drop(12.0, 32.0, -8.0, Color::rgba(0x00000029)),
                    ],
                    item_highlight: theme.foreground.fade(0.05),
                    item_highlight_text: theme.foreground,
                }
            }
        }
    }
}

/// The copy button of the code block and package command.
pub mod copy {
    use super::*;

    /// What the last copy did.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub enum CopyStatus {
        #[default]
        Idle,
        Copied,
        Failed,
    }

    /// How long the tick (or the alert) shows before the copy icon returns.
    pub const FEEDBACK: Duration = Duration::from_millis(1500);
    /// `transition-[opacity,scale] duration-150`.
    pub const SWAP: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);
    /// Hidden icons sit at half size.
    pub const HIDDEN_SCALE: f32 = 0.5;
    /// `transition-colors` on hover.
    pub const HOVER: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);

    /// Box and icon size of a copy button.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct CopyMetrics {
        pub size: f32,
        pub radius: f32,
        pub icon: f32,
    }

    /// The code block's button (`size-7 rounded-lg`, 14 px icons).
    pub const SNIPPET: CopyMetrics = CopyMetrics {
        size: 28.0,
        radius: 8.0,
        icon: 14.0,
    };
    /// The package command's button (`size-8 rounded-[10px]`, 15 px icons).
    pub const COMMAND: CopyMetrics = CopyMetrics {
        size: 32.0,
        radius: 10.0,
        icon: 15.0,
    };

    /// A copy and its feedback timer: `record` notes the outcome and when it
    /// happened, and the status falls back to idle [`FEEDBACK`] later.
    #[derive(Clone, Copy, Debug, PartialEq, Default)]
    pub struct CopyFeedback {
        previous: CopyStatus,
        status: CopyStatus,
        since: Option<Instant>,
    }

    impl CopyFeedback {
        pub fn record(&mut self, copied: bool, now: Instant) {
            self.previous = self.status(now);
            self.status = if copied {
                CopyStatus::Copied
            } else {
                CopyStatus::Failed
            };
            self.since = Some(now);
        }

        pub fn status(&self, now: Instant) -> CopyStatus {
            match self.since {
                Some(at) if now.duration_since(at) < FEEDBACK => self.status,
                _ => CopyStatus::Idle,
            }
        }

        /// Opacity and scale of the copy, tick and alert icons at `now`, cross-fading
        /// into each new status and back to idle when the feedback ends.
        pub fn icons(&self, now: Instant) -> [(f32, f32); 3] {
            let Some(at) = self.since else {
                return icon_targets(CopyStatus::Idle);
            };
            let back = at + FEEDBACK;
            let (from, to, start) = if now < back {
                (self.previous, self.status, at)
            } else {
                (self.status, CopyStatus::Idle, back)
            };
            let p = SWAP.progress(now.duration_since(start));
            let (a, b) = (icon_targets(from), icon_targets(to));
            std::array::from_fn(|i| {
                (
                    crate::motion::lerp(a[i].0, b[i].0, p),
                    crate::motion::lerp(a[i].1, b[i].1, p),
                )
            })
        }

        /// Whether the icons are still moving or a status is still showing.
        pub fn is_animating(&self, now: Instant) -> bool {
            self.since
                .is_some_and(|at| now < at + FEEDBACK + SWAP.duration)
        }
    }

    /// The code block's accessible name for `status`.
    pub fn snippet_label(status: CopyStatus, label: &str) -> String {
        match status {
            CopyStatus::Idle => format!("Copy {label}"),
            CopyStatus::Copied => format!("{label} copied"),
            CopyStatus::Failed => format!("Copying {label} failed"),
        }
    }

    /// The package command's accessible name for `status`.
    pub fn command_label(status: CopyStatus) -> &'static str {
        match status {
            CopyStatus::Idle => "Copy command",
            CopyStatus::Copied => "Copied",
            CopyStatus::Failed => "Copy failed",
        }
    }

    /// Opacity and scale of each icon (copy, tick, alert) for `status`.
    pub fn icon_targets(status: CopyStatus) -> [(f32, f32); 3] {
        let show = |on: bool| if on { (1.0, 1.0) } else { (0.0, HIDDEN_SCALE) };
        [
            show(status == CopyStatus::Idle),
            show(status == CopyStatus::Copied),
            show(status == CopyStatus::Failed),
        ]
    }
}

/// base-ui tabs on the grey shell, with a sliding underline.
pub mod tabs {
    use super::*;

    pub const HEIGHT: f32 = 36.0;
    pub const GAP: f32 = 14.0;
    pub const FONT_SIZE: f32 = 12.0;
    pub const LINE_HEIGHT: f32 = 16.0;
    pub const ICON: f32 = 14.0;
    pub const ICON_GAP: f32 = 6.0;
    pub const INDICATOR_HEIGHT: f32 = 2.0;
    /// `bottom-1`.
    pub const INDICATOR_BOTTOM: f32 = 4.0;
    /// `transition-[left,width] duration-200 ease-[cubic-bezier(0.22,1,0.36,1)]`.
    pub const SLIDE: Tween = Tween::new(Duration::from_millis(200), ease::PROGRESS);
    /// Label colour transition.
    pub const COLOR: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);
}

/// shadcn's skeleton: `animate-pulse rounded-md bg-muted`.
pub mod skeleton {
    use super::*;

    /// Tailwind's `pulse`: 2 s, opacity 1 → 0.5 → 1.
    pub const PULSE: Loop = Loop::new(Duration::from_secs(2), ease::PULSE);

    pub fn opacity(elapsed: Duration) -> f32 {
        crate::motion::keyframes(
            &[(0.0, 1.0), (0.5, 0.5), (1.0, 1.0)],
            PULSE.easing,
            PULSE.phase(elapsed),
        )
    }
}

/// shadcn's `Empty` with an icon, title, description and actions.
pub mod empty {
    pub const PADDING: f32 = 24.0;
    pub const GAP: f32 = 16.0;
    pub const HEADER_GAP: f32 = 8.0;
    pub const MEDIA_SIZE: f32 = 32.0;
    pub const MEDIA_RADIUS: f32 = 8.0;
    pub const MEDIA_ICON: f32 = 16.0;
    /// The media keeps `mb-2` under it, on top of the header gap.
    pub const MEDIA_MARGIN: f32 = 8.0;
    pub const TITLE_TRACKING: f32 = -0.35;
    pub const DESCRIPTION_MAX_WIDTH: f32 = 384.0;
    pub const DESCRIPTION_LINE_HEIGHT: f32 = 22.75;
    pub const CONTENT_GAP: f32 = 10.0;
}

/// Linear interpolation between colours in Oklab, as CSS transitions do.
pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    if t <= 0.0 {
        a
    } else if t >= 1.0 {
        b
    } else {
        a.mix(b, t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shimmer_band_crosses_the_text() {
        let w = 120.0;
        assert!(shimmer::center(0.0, w) < 0.0);
        assert!(shimmer::center(1.0, w) > w);
        assert_eq!(shimmer::amount(50.0, 50.0, 32.0), 1.0);
        assert_eq!(shimmer::amount(100.0, 50.0, 32.0), 0.0);
    }

    #[test]
    fn copy_feedback_returns_to_idle() {
        let now = Instant::now();
        let mut f = copy::CopyFeedback::default();
        f.record(true, now);
        assert_eq!(f.status(now), copy::CopyStatus::Copied);
        assert_eq!(f.status(now + copy::FEEDBACK), copy::CopyStatus::Idle);
        f.record(false, now);
        assert_eq!(f.status(now), copy::CopyStatus::Failed);
        let mid = now + copy::SWAP.duration / 2;
        let icons = f.icons(mid);
        assert!(icons[2].0 > 0.0 && icons[1].0 < 1.0);
        assert_eq!(
            f.icons(now + copy::FEEDBACK + copy::SWAP.duration)[0],
            (1.0, 1.0)
        );
    }

    #[test]
    fn copy_icons_show_one_at_a_time() {
        for status in [
            copy::CopyStatus::Idle,
            copy::CopyStatus::Copied,
            copy::CopyStatus::Failed,
        ] {
            let shown = copy::icon_targets(status)
                .iter()
                .filter(|(o, _)| *o == 1.0)
                .count();
            assert_eq!(shown, 1);
        }
        assert_eq!(copy::icon_targets(copy::CopyStatus::Copied)[1], (1.0, 1.0));
    }

    #[test]
    fn checkbox_mixed_shows_the_inherited_ink() {
        let t = Theme::light();
        let ink = t.muted_foreground;
        let skin = checkbox::CheckboxSkin::new(&t, checkbox::CheckState::Mixed, ink);
        assert_eq!(skin.mark, Some(ink));
        let on = checkbox::CheckboxSkin::new(&t, checkbox::CheckState::On, ink);
        assert_eq!(on.fill, t.primary);
    }

    #[test]
    fn skeleton_pulses_to_half() {
        assert!((skeleton::opacity(Duration::from_secs(1)) - 0.5).abs() < 1e-3);
        assert!((skeleton::opacity(Duration::ZERO) - 1.0).abs() < 1e-3);
    }
}
