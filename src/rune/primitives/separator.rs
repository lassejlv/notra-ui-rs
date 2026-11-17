use ::rune::{El, el};

use crate::components::primitives::separator::{self, Orientation};
use crate::rune::ToRune;
use crate::theme::{Color, Theme};

/// A one-pixel rule: `Separator::horizontal().build(&theme)`.
pub struct Separator {
    orientation: Orientation,
    color: Option<Color>,
}

impl Separator {
    pub fn horizontal() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            color: None,
        }
    }

    pub fn vertical() -> Self {
        Self {
            orientation: Orientation::Vertical,
            ..Self::horizontal()
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let rule = el()
            .fill(self.color.unwrap_or_else(|| separator::color(theme)).rune())
            .role("separator")
            .label(self.orientation.name());
        match self.orientation {
            Orientation::Horizontal => rule.h(separator::THICKNESS).w_full(),
            Orientation::Vertical => rule.w(separator::THICKNESS).self_stretch(),
        }
    }
}
