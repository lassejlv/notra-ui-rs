//! Notra's `Heading` (Satoshi display steps) and `Text` (Inter body copy).

use crate::theme::{Color, Theme};

/// Type metrics in px; `tracking_em` is CSS `letter-spacing` in em.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeMetrics {
    pub size: f32,
    pub line_height: f32,
    pub tracking_em: f32,
    pub weight: f32,
}

impl TypeMetrics {
    const fn display(size: f32, line_height: f32, tracking_em: f32) -> Self {
        Self {
            size,
            line_height,
            tracking_em,
            weight: 500.0,
        }
    }

    const fn body(size: f32, line_height: f32) -> Self {
        Self {
            size,
            line_height,
            tracking_em: 0.0,
            weight: 400.0,
        }
    }

    /// Letter spacing in px.
    pub fn tracking(&self) -> f32 {
        self.tracking_em * self.size
    }
}

/// The display scale (`text-display-*`).
pub mod display {
    use super::TypeMetrics;

    pub const XL2: TypeMetrics = TypeMetrics::display(48.0, 52.8, -0.02);
    pub const XL: TypeMetrics = TypeMetrics::display(46.0, 51.52, -0.02);
    pub const LG: TypeMetrics = TypeMetrics::display(36.0, 39.6, -0.02);
    pub const MD: TypeMetrics = TypeMetrics::display(32.0, 35.84, -0.02);
    pub const SM: TypeMetrics = TypeMetrics::display(28.0, 33.88, -0.02);
    pub const XS: TypeMetrics = TypeMetrics::display(18.0, 24.0, -0.015);
}

/// Tailwind's `sm` breakpoint: hero and section headings grow a step at or above it.
pub const WIDE_BREAKPOINT: f32 = 640.0;

/// A heading's display step; independent of its level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum HeadingSize {
    Hero,
    #[default]
    Section,
    Title,
    Subhead,
}

impl HeadingSize {
    pub const ALL: [HeadingSize; 4] = [
        HeadingSize::Hero,
        HeadingSize::Section,
        HeadingSize::Title,
        HeadingSize::Subhead,
    ];

    /// Metrics for a viewport at least [`WIDE_BREAKPOINT`] wide (`wide`) or narrower.
    pub fn metrics(self, wide: bool) -> TypeMetrics {
        match (self, wide) {
            (HeadingSize::Hero, true) => display::XL2,
            (HeadingSize::Hero, false) => display::LG,
            (HeadingSize::Section, true) => display::XL,
            (HeadingSize::Section, false) => display::MD,
            (HeadingSize::Title, _) => display::SM,
            (HeadingSize::Subhead, _) => display::XS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TextSize {
    Lg,
    #[default]
    Base,
    Sm,
    Xs,
    /// `text-2xs` (11 px), which sets no line height and inherits the body's 1.5.
    Xs2,
}

impl TextSize {
    pub const ALL: [TextSize; 5] = [
        TextSize::Lg,
        TextSize::Base,
        TextSize::Sm,
        TextSize::Xs,
        TextSize::Xs2,
    ];

    pub fn metrics(self) -> TypeMetrics {
        match self {
            TextSize::Lg => TypeMetrics::body(18.0, 28.0),
            TextSize::Base => TypeMetrics::body(16.0, 24.0),
            TextSize::Sm => TypeMetrics::body(14.0, 20.0),
            TextSize::Xs => TypeMetrics::body(12.0, 16.0),
            TextSize::Xs2 => TypeMetrics::body(11.0, 16.5),
        }
    }
}

/// Text colour; unset inherits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextTone {
    Default,
    Muted,
}

impl TextTone {
    pub fn color(self, theme: &Theme) -> Color {
        match self {
            TextTone::Default => theme.foreground,
            TextTone::Muted => theme.muted_foreground,
        }
    }
}

/// Weight; unset inherits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextWeight {
    Regular,
    Medium,
}

impl TextWeight {
    pub fn value(self) -> f32 {
        match self {
            TextWeight::Regular => 400.0,
            TextWeight::Medium => 500.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hero_grows_a_step_when_wide() {
        assert_eq!(HeadingSize::Hero.metrics(false).size, 36.0);
        assert_eq!(HeadingSize::Hero.metrics(true).size, 48.0);
        assert!((display::XL2.tracking() + 0.96).abs() < 1e-4);
    }
}
