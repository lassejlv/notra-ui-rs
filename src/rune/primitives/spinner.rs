use ::rune::{El, icon};

use super::spin;
use crate::components::primitives::spinner;
use crate::rune::ToRune;
use crate::theme::{Color, Theme};

/// shadcn's spinner: `Spinner::new().build(&theme)`.
pub struct Spinner {
    size: f32,
    color: Option<Color>,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            size: spinner::SIZE,
            color: None,
        }
    }

    /// Diameter in px (`size-4` by default).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// The stroke colour; the foreground by default.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        icon(spinner::ICON.rune(), self.size)
            .color(self.color.unwrap_or(theme.foreground).rune())
            .looping(spin())
            .role("status")
            .label(spinner::LABEL)
    }
}
