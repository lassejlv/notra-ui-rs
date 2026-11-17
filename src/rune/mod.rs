//! Rune front end.
//!
//! Components are builders that end in `.build(&theme)` and return a `rune::El`,
//! with handlers over your app state, as rune-kit's do:
//!
//! ```ignore
//! use notra_ui::rune::{Button, ButtonVariant, FONTS};
//! use notra_ui::Theme;
//!
//! fn view(s: &State, cx: &mut rune::Cx<State>) -> rune::El {
//!     let theme = Theme::new(if cx.system_dark() { Appearance::Dark } else { Appearance::Light });
//!     Button::new("Save changes")
//!         .loading(s.saving)
//!         .on_press(|s: &mut State| s.save())
//!         .build(&theme)
//!         .key("save")
//! }
//! ```
//!
//! Register the bundled faces with `AppSpec { fonts: notra_ui::rune::FONTS, families: notra_ui::rune::FAMILIES, .. }`.

use ::rune::tok::{Family, TextStyle};
use ::rune_motion::{CubicBezier as RuneBezier, Curve, Easing};

use crate::fonts;
use crate::motion::{CubicBezier, Tween};
use crate::theme::{Appearance, Color, Shadow, Theme};

mod button;
mod navigation;

pub use crate::components::button::{ButtonSize, ButtonVariant};
pub use button::Button;
pub use navigation::*;

/// Every bundled face, for `AppSpec::fonts`.
pub const FONTS: &[&[u8]] = fonts::ALL;
/// `AppSpec::families`: Inter for UI text, Geist Mono for code.
pub const FAMILIES: (&str, &str) = (fonts::SANS, fonts::MONO);

/// The theme for the system appearance.
pub fn system_theme<S>(cx: &::rune::Cx<S>) -> Theme {
    Theme::new(if cx.system_dark() {
        Appearance::Dark
    } else {
        Appearance::Light
    })
}

/// Converts a token (sRGB) to Rune's linear colour.
pub trait ToRune {
    type Output;
    fn rune(&self) -> Self::Output;
}

impl ToRune for Color {
    type Output = ::rune::Color;
    fn rune(&self) -> ::rune::Color {
        ::rune::Color {
            r: ::rune_motion::srgb_decode(self.r),
            g: ::rune_motion::srgb_decode(self.g),
            b: ::rune_motion::srgb_decode(self.b),
            a: self.a,
        }
    }
}

impl ToRune for Shadow {
    type Output = ::rune::Shadow;
    fn rune(&self) -> ::rune::Shadow {
        ::rune::Shadow {
            x: self.x,
            y: self.y,
            blur: self.blur,
            spread: self.spread,
            color: self.color.rune(),
            inset: self.inset,
        }
    }
}

impl ToRune for CubicBezier {
    type Output = Easing;
    fn rune(&self) -> Easing {
        Easing::CubicBezier(RuneBezier::new(self.x1, self.y1, self.x2, self.y2))
    }
}

/// A CSS transition as a Rune curve.
pub fn tween(t: Tween) -> Curve {
    Curve::tween(t.duration.as_secs_f32(), t.easing.rune())
}

/// Inter at `size` and `weight` with a CSS line height.
pub fn sans(size: f32, weight: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size, weight)
        .family(Family::Named(fonts::SANS))
        .lh(line_height)
}

/// Geist Mono at `size` and `weight` with a CSS line height.
pub fn mono(size: f32, weight: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size, weight)
        .family(Family::Named(fonts::MONO))
        .lh(line_height)
}

/// Approximates CSS `filter: brightness(f)` on `base` as a translucent overlay,
/// for Rune's hover and press tints.
pub fn brightness_overlay(base: Color, factor: f32) -> Color {
    if factor >= 1.0 {
        let gain = [base.r, base.g, base.b].map(|c| {
            if c < 1.0 {
                ((c * factor).min(1.0) - c) / (1.0 - c)
            } else {
                0.0
            }
        });
        Color::WHITE.alpha((gain.iter().sum::<f32>() / 3.0).clamp(0.0, 1.0))
    } else {
        Color::BLACK.alpha(1.0 - factor)
    }
}
