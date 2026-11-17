//! The Depth button: variants, sizes, surfaces and the loading/progress motion,
//! shared by the GPUI and Rune front ends.

use std::time::Duration;

use crate::motion::{CubicBezier, Loop, Tween, ease, keyframes};
use crate::theme::{Appearance, Color, Shadow, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ButtonVariant {
    /// The violet Depth button (`variant="default"`).
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    GhostMuted,
    Destructive,
    Link,
}

impl ButtonVariant {
    pub const ALL: [ButtonVariant; 7] = [
        ButtonVariant::Primary,
        ButtonVariant::Secondary,
        ButtonVariant::Outline,
        ButtonVariant::Ghost,
        ButtonVariant::GhostMuted,
        ButtonVariant::Destructive,
        ButtonVariant::Link,
    ];

    /// Depth variants have a gradient, a top highlight and squircle corners.
    pub fn is_depth(self) -> bool {
        matches!(
            self,
            ButtonVariant::Primary
                | ButtonVariant::Secondary
                | ButtonVariant::Outline
                | ButtonVariant::Destructive
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ButtonSize {
    Xs,
    Sm,
    #[default]
    Default,
    Lg,
    Icon,
    IconXs,
    IconSm,
    IconLg,
}

/// Box metrics of a size, in logical pixels, as measured on ui.usenotra.com.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonMetrics {
    pub height: f32,
    /// `None` for square icon buttons.
    pub padding_x: Option<f32>,
    pub gap: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub icon_size: f32,
    /// Corner radius of flat (ghost, link) variants. Depth variants use [`DEPTH_RADIUS`].
    pub radius: f32,
}

/// The squircle radius of Depth variants (`rounded-[0.75rem]` with `corner-shape: squircle`).
pub const DEPTH_RADIUS: f32 = 12.0;

impl ButtonSize {
    pub fn metrics(self) -> ButtonMetrics {
        let text = |height, padding_x, gap, font_size, line_height, radius| ButtonMetrics {
            height,
            padding_x: Some(padding_x),
            gap,
            font_size,
            line_height,
            icon_size: if font_size < 13.0 { 12.0 } else { 16.0 },
            radius,
        };
        let icon = |height: f32, icon_size, radius| ButtonMetrics {
            height,
            padding_x: None,
            gap: 4.0,
            font_size: 14.0,
            line_height: 20.0,
            icon_size,
            radius,
        };
        match self {
            ButtonSize::Xs => text(24.0, 8.0, 4.0, 12.0, 16.0, 6.0),
            ButtonSize::Sm => text(28.0, 10.0, 4.0, 12.8, 19.2, 7.0),
            ButtonSize::Default => text(32.0, 10.0, 6.0, 14.0, 20.0, 8.0),
            ButtonSize::Lg => text(36.0, 10.0, 6.0, 14.0, 20.0, 8.0),
            ButtonSize::Icon => icon(32.0, 16.0, 8.0),
            ButtonSize::IconXs => icon(24.0, 12.0, 6.0),
            ButtonSize::IconSm => icon(28.0, 14.0, 7.0),
            ButtonSize::IconLg => icon(36.0, 16.0, 8.0),
        }
    }

    pub fn is_icon(self) -> bool {
        self.metrics().padding_x.is_none()
    }

    /// Padding on the side that holds an icon (`has-data-[icon=inline-*]:p*-*`), for
    /// text sizes.
    pub fn icon_side_padding(self) -> Option<f32> {
        match self {
            ButtonSize::Xs | ButtonSize::Sm => Some(6.0),
            ButtonSize::Default | ButtonSize::Lg => Some(8.0),
            _ => None,
        }
    }
}

/// What hovering changes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonHover {
    /// CSS `filter: brightness(f)` over the whole button.
    Brightness(f32),
    /// A new gradient (the secondary variant lightens its stops).
    Gradient { top: Color, bottom: Color },
    /// A fill behind the label and a new text colour (ghost variants).
    Fill { fill: Color, text: Color },
    /// An underline (link).
    Underline,
}

/// A variant's resolved paint for one theme.
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonSkin {
    /// Flat background under the gradient, already composited with it.
    pub fill: Color,
    /// `bg-linear-to-b` stops, top to bottom, interpolated in Oklab.
    pub gradient: Option<(Color, Color)>,
    pub border: Color,
    pub text: Color,
    pub shadows: Vec<Shadow>,
    pub hover: ButtonHover,
    pub radius: Option<f32>,
}

const DEPTH_SHADOWS: [Shadow; 2] = [
    Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffff47)),
    Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x0000002e)),
];

impl ButtonSkin {
    pub fn new(theme: &Theme, variant: ButtonVariant) -> Self {
        let dark = theme.appearance == Appearance::Dark;
        let depth =
            |fill: Color, top: Color, bottom: Color, border: Color, text: Color, hover| Self {
                fill,
                gradient: Some((top.over(fill), bottom.over(fill))),
                border,
                text,
                shadows: DEPTH_SHADOWS.to_vec(),
                hover,
                radius: Some(DEPTH_RADIUS),
            };
        let flat = |text: Color, hover| Self {
            fill: Color::TRANSPARENT,
            gradient: None,
            border: Color::TRANSPARENT,
            text,
            shadows: Vec::new(),
            hover,
            radius: None,
        };
        let ghost_hover = ButtonHover::Fill {
            fill: if dark {
                theme.muted.fade(0.5)
            } else {
                theme.muted
            },
            text: theme.foreground,
        };
        match variant {
            ButtonVariant::Primary => depth(
                theme.primary,
                theme.primary.mix(Color::BLACK, 0.08),
                theme.primary.mix(Color::BLACK, 0.20),
                theme.primary.fade(0.6),
                theme.primary_foreground,
                ButtonHover::Brightness(1.1),
            ),
            ButtonVariant::Secondary => depth(
                theme.secondary,
                theme.foreground.fade(0.85),
                theme.foreground,
                theme.foreground.fade(0.6),
                theme.background,
                ButtonHover::Gradient {
                    top: theme.foreground.fade(0.70).over(theme.secondary),
                    bottom: theme.foreground.fade(0.85).over(theme.secondary),
                },
            ),
            ButtonVariant::Outline => {
                let (top, bottom) = if dark {
                    (theme.input, theme.muted)
                } else {
                    (theme.background, theme.muted)
                };
                Self {
                    shadows: if dark {
                        vec![
                            Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffff0f)),
                            Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x00000066)),
                        ]
                    } else {
                        vec![
                            Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffffb3)),
                            Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x0000000f)),
                        ]
                    },
                    ..depth(
                        theme.background,
                        top,
                        bottom,
                        theme.border,
                        theme.foreground,
                        ButtonHover::Brightness(if dark { 1.25 } else { 0.97 }),
                    )
                }
            }
            ButtonVariant::Destructive => {
                let (top, bottom) = if dark {
                    (
                        theme.destructive.mix(Color::BLACK, 0.20),
                        theme.destructive.mix(Color::BLACK, 0.30),
                    )
                } else {
                    (theme.destructive, theme.destructive.mix(Color::BLACK, 0.12))
                };
                depth(
                    theme.destructive.fade(0.1),
                    top,
                    bottom,
                    theme.destructive.fade(0.6),
                    Color::WHITE,
                    ButtonHover::Brightness(1.1),
                )
            }
            ButtonVariant::Ghost => flat(theme.foreground, ghost_hover),
            ButtonVariant::GhostMuted => flat(theme.muted_foreground, ghost_hover),
            ButtonVariant::Link => flat(theme.foreground, ButtonHover::Underline),
        }
    }

    /// The skin under the pointer.
    pub fn hovered(&self) -> Self {
        let mut skin = self.clone();
        match self.hover {
            ButtonHover::Brightness(f) => {
                skin.fill = skin.fill.brightness(f);
                skin.gradient = skin
                    .gradient
                    .map(|(t, b)| (t.brightness(f), b.brightness(f)));
                skin.border = skin.border.brightness(f);
                skin.text = skin.text.brightness(f);
            }
            ButtonHover::Gradient { top, bottom } => skin.gradient = Some((top, bottom)),
            ButtonHover::Fill { fill, text } => {
                skin.fill = fill;
                skin.text = text;
            }
            ButtonHover::Underline => {}
        }
        skin
    }
}

/// Opacity of a disabled button (`disabled:opacity-50`).
pub const DISABLED_OPACITY: f32 = 0.5;

/// Hover and colour transitions: Tailwind's `transition-all` at 150 ms.
pub const TRANSITION: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);

/// The loading and progress choreography.
pub mod loading {
    use super::*;

    /// `button-swap-in`: content rises in from 0.45em below, unblurring.
    pub const SWAP_IN: Tween = Tween::new(Duration::from_millis(280), ease::OUT);
    /// `button-swap-out`'s curve.
    pub const SWAP_OUT_EASE: CubicBezier = CubicBezier::new(0.45, 0.0, 0.25, 1.0);
    /// `button-swap-out`: content rises out 0.45em above, blurring.
    pub const SWAP_OUT: Tween = Tween::new(Duration::from_millis(360), SWAP_OUT_EASE);
    /// The label swaps back in 70 ms after loading ends.
    pub const LABEL_RETURN_DELAY: Duration = Duration::from_millis(70);
    /// Each dot's swap-in delay.
    pub const DOT_ENTER_DELAYS: [Duration; 3] = [
        Duration::from_millis(70),
        Duration::from_millis(110),
        Duration::from_millis(150),
    ];
    /// Each dot's bounce phase delay.
    pub const DOT_BOUNCE_DELAYS: [Duration; 3] = [
        Duration::from_millis(0),
        Duration::from_millis(140),
        Duration::from_millis(280),
    ];
    /// One bounce cycle (`button-dot 1s ease-in-out infinite`).
    pub const DOT_BOUNCE: Loop = Loop::new(
        Duration::from_secs(1),
        CubicBezier::new(0.42, 0.0, 0.58, 1.0),
    );
    /// Dot diameter and gap, in em of the button's font size.
    pub const DOT_SIZE_EM: f32 = 0.3;
    pub const DOT_GAP_EM: f32 = 0.2;
    /// Distance content travels during a swap, in em.
    pub const SWAP_TRAVEL_EM: f32 = 0.45;
    /// Blur at the far end of a swap, in px.
    pub const SWAP_BLUR: f32 = 4.0;

    /// The progress fill's width transition.
    pub const PROGRESS_FILL: Tween = Tween::new(Duration::from_millis(300), ease::PROGRESS);
    /// `button-progress-out`: the full bar holds, then fades.
    pub const PROGRESS_OUT: Duration = Duration::from_millis(600);
    /// The fill is the text colour at 15 %.
    pub const PROGRESS_ALPHA: f32 = 0.15;

    /// One frame of a swap: opacity, vertical offset in em, blur in px.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct SwapFrame {
        pub opacity: f32,
        pub offset_em: f32,
        pub blur: f32,
    }

    impl SwapFrame {
        pub const SHOWN: SwapFrame = SwapFrame {
            opacity: 1.0,
            offset_em: 0.0,
            blur: 0.0,
        };
        pub const HIDDEN: SwapFrame = SwapFrame {
            opacity: 0.0,
            offset_em: 0.0,
            blur: 0.0,
        };
    }

    /// Content swapping in, `elapsed` after its delay began counting.
    pub fn swap_in(elapsed: Duration, delay: Duration) -> SwapFrame {
        let p = SWAP_IN.delay(delay).progress(elapsed);
        SwapFrame {
            opacity: p,
            offset_em: SWAP_TRAVEL_EM * (1.0 - p),
            blur: SWAP_BLUR * (1.0 - p),
        }
    }

    /// Content swapping out.
    pub fn swap_out(elapsed: Duration) -> SwapFrame {
        let p = SWAP_OUT.progress(elapsed);
        SwapFrame {
            opacity: 1.0 - p,
            offset_em: -SWAP_TRAVEL_EM * p,
            blur: SWAP_BLUR * p,
        }
    }

    /// A bouncing dot: opacity and vertical offset in em.
    pub fn dot(elapsed: Duration, index: usize) -> (f32, f32) {
        let phase = DOT_BOUNCE.delay(DOT_BOUNCE_DELAYS[index]).phase(elapsed);
        let curve = DOT_BOUNCE.easing;
        let opacity = keyframes(
            &[(0.0, 0.45), (0.3, 1.0), (0.6, 0.45), (1.0, 0.45)],
            curve,
            phase,
        );
        let offset = keyframes(
            &[(0.0, 0.0), (0.3, -0.3), (0.6, 0.0), (1.0, 0.0)],
            curve,
            phase,
        );
        (opacity, offset)
    }

    /// Opacity of the completed progress bar `elapsed` into its fade.
    pub fn progress_out(elapsed: Duration) -> f32 {
        let t = elapsed.as_secs_f32() / PROGRESS_OUT.as_secs_f32();
        keyframes(&[(0.0, 1.0), (0.5, 1.0), (1.0, 0.0)], ease::CSS_EASE_OUT, t)
    }

    /// How long the swap out of the dots takes after loading ends.
    pub fn settle_time() -> Duration {
        SWAP_OUT
            .total()
            .max(SWAP_IN.delay(LABEL_RETURN_DELAY).total())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_variants_have_a_gradient_and_squircle_radius() {
        let theme = Theme::light();
        for variant in ButtonVariant::ALL {
            let skin = ButtonSkin::new(&theme, variant);
            assert_eq!(skin.gradient.is_some(), variant.is_depth(), "{variant:?}");
            assert_eq!(skin.radius.is_some(), variant.is_depth(), "{variant:?}");
        }
    }

    #[test]
    fn primary_gradient_darkens_downwards() {
        let skin = ButtonSkin::new(&Theme::light(), ButtonVariant::Primary);
        let (top, bottom) = skin.gradient.unwrap();
        assert!(top.r > bottom.r && top.b > bottom.b);
    }

    #[test]
    fn dots_bounce_in_sequence() {
        let at = Duration::from_millis(300);
        let (first, _) = loading::dot(at, 0);
        let (last, _) = loading::dot(at, 2);
        assert!(first > last);
    }

    #[test]
    fn swap_lands_shown() {
        let f = loading::swap_in(Duration::from_secs(1), Duration::ZERO);
        assert_eq!(f, loading::SwapFrame::SHOWN);
    }
}
