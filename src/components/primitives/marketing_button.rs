//! Notra's `MarketingButton`: a pill with an oklab gradient whose stops ease on hover
//! and press, and a spinner that swaps in for the label while loading.

use std::time::Duration;

use super::oklab;
use crate::components::button::loading::{SWAP_IN, SWAP_OUT};
use crate::motion::{Tween, ease};
use crate::theme::{Color, Shadow};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum MarketingVariant {
    /// Violet gradient with a soft violet halo.
    #[default]
    Primary,
    /// The violet gradient without the halo.
    Flat,
    /// White to light grey with dark text.
    Light,
}

impl MarketingVariant {
    pub const ALL: [MarketingVariant; 3] = [
        MarketingVariant::Primary,
        MarketingVariant::Flat,
        MarketingVariant::Light,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum MarketingSize {
    Sm,
    #[default]
    Default,
    Lg,
}

/// Box metrics in px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarketingMetrics {
    pub height: f32,
    pub padding_x: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub icon_size: f32,
}

impl MarketingSize {
    pub fn metrics(self) -> MarketingMetrics {
        match self {
            MarketingSize::Sm => MarketingMetrics {
                height: 40.0,
                padding_x: 22.0,
                font_size: 14.0,
                line_height: 20.0,
                icon_size: 16.0,
            },
            MarketingSize::Default => MarketingMetrics {
                height: 44.0,
                padding_x: 24.0,
                font_size: 16.0,
                line_height: 24.0,
                icon_size: 16.0,
            },
            MarketingSize::Lg => MarketingMetrics {
                height: 48.0,
                padding_x: 32.0,
                font_size: 18.0,
                line_height: 28.0,
                icon_size: 20.0,
            },
        }
    }
}

/// `gap-2`.
pub const GAP: f32 = 8.0;
/// `tracking-marketing`: -0.015em.
pub const TRACKING_EM: f32 = -0.015;
/// `font-medium`.
pub const WEIGHT: f32 = 500.0;
/// `disabled:opacity-50`.
pub const DISABLED_OPACITY: f32 = 0.5;
/// The gradient stops' `transition: 200ms ease-out`.
pub const TRANSITION: Tween = Tween::new(Duration::from_millis(200), ease::CSS_EASE_OUT);

/// A top-to-bottom gradient (`--marketing-gradient-from` / `-to`), interpolated in Oklab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stops {
    pub from: Color,
    pub to: Color,
}

impl Stops {
    /// Stops `t` of the way to `other`, as the registered custom properties animate.
    pub fn lerp(self, other: Stops, t: f32) -> Stops {
        if t <= 0.0 {
            return self;
        }
        if t >= 1.0 {
            return other;
        }
        Stops {
            from: self.from.mix(other.from, t),
            to: self.to.mix(other.to, t),
        }
    }
}

/// A variant's paint at rest, under the pointer and while pressed.
#[derive(Clone, Debug, PartialEq)]
pub struct MarketingSkin {
    pub rest: Stops,
    pub hover: Stops,
    pub active: Stops,
    pub text: Color,
    pub shadows: Vec<Shadow>,
}

impl MarketingSkin {
    /// Both appearances paint the same: the button is a brand surface.
    pub fn new(variant: MarketingVariant) -> Self {
        let lift = Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x2828_2814));
        let edge = Shadow::drop(0.0, 0.0, 1.0, Color::rgba(0x1e1e_1e40));
        let violet = |shadows| Self {
            rest: Stops {
                from: oklab(0.699, 0.067, -0.16),
                to: oklab(0.526, 0.113, -0.27),
            },
            hover: Stops {
                from: oklab(0.699, 0.067, -0.16),
                to: oklab(0.339, 0.073, -0.174),
            },
            active: Stops {
                from: oklab(0.526, 0.113, -0.27),
                to: oklab(0.248, 0.053, -0.127),
            },
            text: Color::WHITE,
            shadows,
        };
        match variant {
            MarketingVariant::Primary => violet(vec![
                Shadow::drop(0.0, 0.0, 8.0, Color::rgba(0x8b5c_f640)),
                lift,
                edge,
            ]),
            MarketingVariant::Flat => violet(vec![lift, edge]),
            MarketingVariant::Light => Self {
                rest: Stops {
                    from: oklab(1.0, 0.0, 0.0),
                    to: oklab(0.955, 0.0, 0.0),
                },
                hover: Stops {
                    from: oklab(1.0, 0.0, 0.0),
                    to: oklab(0.89, 0.0, 0.0),
                },
                active: Stops {
                    from: oklab(0.955, 0.0, 0.0),
                    to: oklab(0.85, 0.0, 0.0),
                },
                text: Color::rgb(0x1e1e1e),
                shadows: vec![Shadow::drop(0.0, 0.0, 1.0, Color::rgb(0xececec)), lift],
            },
        }
    }

    /// The stops for hover progress `hover` and press progress `press`, each `0..=1`.
    pub fn stops(&self, hover: f32, press: f32) -> Stops {
        self.rest.lerp(self.hover, hover).lerp(self.active, press)
    }
}

/// The loading swap: the label leaves as the button's does, and a spinner rises in.
pub mod loading {
    use super::*;

    /// The spinner's swap in waits this long.
    pub const ENTER_DELAY: Duration = Duration::from_millis(70);
    /// Spinner diameter in em (`size-[1.125em]`).
    pub const SPINNER_EM: f32 = 1.125;
    /// Opacity of the spinner's track ring.
    pub const TRACK_OPACITY: f32 = 0.3;
    /// The spinner's arc and the track are drawn at 2.5 px in a 20 px box.
    pub const STROKE_EM: f32 = 2.5 / 20.0 * SPINNER_EM;

    /// How long a swap in either direction takes to settle.
    pub fn settle_time() -> Duration {
        SWAP_OUT.total().max(SWAP_IN.delay(ENTER_DELAY).total())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hover_darkens_only_the_bottom_stop() {
        let skin = MarketingSkin::new(MarketingVariant::Primary);
        let hot = skin.stops(1.0, 0.0);
        assert_eq!(hot.from, skin.rest.from);
        assert!(hot.to.r + hot.to.g + hot.to.b < skin.rest.to.r + skin.rest.to.g + skin.rest.to.b);
    }

    #[test]
    fn only_primary_has_the_halo() {
        for variant in MarketingVariant::ALL {
            let halo = MarketingSkin::new(variant)
                .shadows
                .iter()
                .any(|s| s.spread == 8.0);
            assert_eq!(halo, variant == MarketingVariant::Primary);
        }
    }
}
