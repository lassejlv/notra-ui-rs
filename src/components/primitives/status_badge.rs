//! Notra's dashboard badges (`@notra/ui` `StatusBadge` and `TrendBadge`, the Status &
//! Trend Badges page): semantic tints in three sizes and two shapes, an optional
//! glyph or filled result square, and a plain inline appearance for trend readouts.

use std::time::Duration;

use super::{TW_EASE_OUT, oklab};
use crate::icons::Icon;
use crate::motion::Tween;
use crate::theme::{Color, Theme};

/// The semantic colour of a badge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum StatusVariant {
    /// Neutral states.
    #[default]
    Secondary,
    /// Queued work.
    Info,
    /// Work in progress.
    Warning,
    /// Successful outcomes.
    Success,
    /// Failures.
    Destructive,
    /// Merged changes.
    Merged,
}

impl StatusVariant {
    pub const ALL: [StatusVariant; 6] = [
        StatusVariant::Secondary,
        StatusVariant::Info,
        StatusVariant::Warning,
        StatusVariant::Success,
        StatusVariant::Destructive,
        StatusVariant::Merged,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum BadgeSize {
    Sm,
    #[default]
    Default,
    Md,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum BadgeShape {
    /// `rounded-md`.
    #[default]
    Rounded,
    /// `rounded-full`.
    Pill,
}

impl BadgeShape {
    pub fn radius(self) -> f32 {
        match self {
            BadgeShape::Rounded => 6.0,
            BadgeShape::Pill => crate::theme::radius::FULL,
        }
    }
}

/// A tinted badge, or bare coloured text for inline readouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum BadgeAppearance {
    #[default]
    Badge,
    Plain,
}

/// Box metrics of a size, measured on the live page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BadgeMetrics {
    pub height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub icon_size: f32,
}

impl BadgeSize {
    pub fn metrics(self) -> BadgeMetrics {
        match self {
            BadgeSize::Sm => BadgeMetrics {
                height: 16.0,
                padding_x: 6.0,
                padding_y: 0.0,
                gap: 2.0,
                font_size: 10.0,
                line_height: 15.0,
                icon_size: 10.0,
            },
            BadgeSize::Default => BadgeMetrics {
                height: 20.0,
                padding_x: 8.0,
                padding_y: 2.0,
                gap: 4.0,
                font_size: 12.0,
                line_height: 16.0,
                icon_size: 12.0,
            },
            BadgeSize::Md => BadgeMetrics {
                height: 24.0,
                padding_x: 8.0,
                padding_y: 2.0,
                gap: 6.0,
                font_size: 14.0,
                line_height: 20.0,
                icon_size: 14.0,
            },
        }
    }
}

/// `font-medium`.
pub const WEIGHT: f32 = 500.0;
/// Status glyphs draw at 1.5 px, trend arrows at 2 px; both live in the icon files.
/// The filled result square: `size-3.5 rounded-xs bg-current` around a 12 px glyph.
pub const SQUARE_SIZE: f32 = 14.0;
pub const SQUARE_RADIUS: f32 = 2.0;
pub const SQUARE_GLYPH: f32 = 12.0;
/// `transition-colors duration-fast ease-out`.
pub const TRANSITION: Tween = Tween::new(Duration::from_millis(150), TW_EASE_OUT);

/// A variant's tones: the tint behind the badge and the text on it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatusSkin {
    pub fill: Color,
    pub text: Color,
}

impl StatusSkin {
    /// The neutral badge is untinted in dark mode: the site's stylesheet sets
    /// `--badge-secondary` to `color-mix(in oklab, var(--foreground) 10%, transparent)`
    /// there, but the value computes to nothing in the browser, so it paints no fill.
    pub fn new(theme: &Theme, variant: StatusVariant, appearance: BadgeAppearance) -> Self {
        let dark = theme.is_dark();
        let tint = if dark { 0.2 } else { 0.1 };
        let semantic = |tone: Color, text: Color| Self {
            fill: tone.fade(tint),
            text,
        };
        let pick = |light: (f32, f32, f32), dark_: (f32, f32, f32)| {
            let (l, c, h) = if dark { dark_ } else { light };
            Color::oklch(l, c, h)
        };
        let skin = match variant {
            StatusVariant::Secondary => Self {
                fill: if dark {
                    Color::TRANSPARENT
                } else {
                    theme.secondary
                },
                text: theme.secondary_foreground,
            },
            StatusVariant::Info => semantic(
                pick((0.55, 0.13, 250.0), (0.70, 0.12, 250.0)),
                pick((0.46, 0.13, 250.0), (0.76, 0.12, 250.0)),
            ),
            StatusVariant::Warning => semantic(
                pick((0.57, 0.113, 55.0), (0.70, 0.125, 55.0)),
                pick((0.46, 0.113, 55.0), (0.76, 0.125, 55.0)),
            ),
            StatusVariant::Success => semantic(
                pick((0.55, 0.109, 155.0), (0.70, 0.125, 155.0)),
                pick((0.46, 0.109, 155.0), (0.76, 0.125, 155.0)),
            ),
            StatusVariant::Destructive => semantic(
                if dark {
                    oklab(0.65, 0.190_325, 0.088_75)
                } else {
                    theme.destructive
                },
                pick((0.48, 0.18, 25.0), (0.76, 0.16, 25.0)),
            ),
            StatusVariant::Merged => semantic(
                pick((0.55, 0.2, 295.0), (0.68, 0.16, 295.0)),
                pick((0.47, 0.2, 295.0), (0.76, 0.13, 295.0)),
            ),
        };
        match appearance {
            BadgeAppearance::Badge => skin,
            BadgeAppearance::Plain => Self {
                fill: Color::TRANSPARENT,
                text: if variant == StatusVariant::Secondary {
                    theme.muted_foreground
                } else {
                    skin.text
                },
            },
        }
    }
}

/// Whether a change is good, bad or neither (not its sign: a better rank can be up
/// with a negative delta).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrendTone {
    Up,
    Down,
    Flat,
}

impl TrendTone {
    pub fn variant(self) -> StatusVariant {
        match self {
            TrendTone::Up => StatusVariant::Success,
            TrendTone::Down => StatusVariant::Destructive,
            TrendTone::Flat => StatusVariant::Secondary,
        }
    }

    /// Up-right, down-right, or a minus.
    pub fn icon(self) -> Icon {
        match self {
            TrendTone::Up => Icon::HugeArrowUpRight01,
            TrendTone::Down => Icon::HugeArrowDownRight01,
            TrendTone::Flat => Icon::HugeMinusSign,
        }
    }
}

/// Trend badges and plain readouts set figures in tabular numerals.
pub const TABULAR_NUMS: &str = "'tnum' 1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_success_matches_the_live_tint() {
        let skin = StatusSkin::new(
            &Theme::light(),
            StatusVariant::Success,
            BadgeAppearance::Badge,
        );
        assert!((skin.fill.a - 0.1).abs() < 1e-6);
        assert!(skin.text.g > skin.text.r && skin.text.g > skin.text.b);
    }

    #[test]
    fn plain_flat_reads_muted() {
        let theme = Theme::dark();
        let skin = StatusSkin::new(&theme, StatusVariant::Secondary, BadgeAppearance::Plain);
        assert_eq!(skin.text, theme.muted_foreground);
        assert_eq!(skin.fill, Color::TRANSPARENT);
    }

    #[test]
    fn sizes_grow() {
        let [sm, df, md] =
            [BadgeSize::Sm, BadgeSize::Default, BadgeSize::Md].map(BadgeSize::metrics);
        assert!(sm.height < df.height && df.height < md.height);
    }
}
