use ::gpui::{App, IntoElement, RenderOnce, Styled, Window, div, px};

use crate::components::primitives::separator::{self, Orientation};
use crate::gpui::ActiveTheme;
use crate::theme::Color;

/// A one-pixel rule: `Separator::horizontal()` fills the width, `vertical()` the height.
#[derive(IntoElement)]
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

    /// Overrides `bg-border` (blocks tint it with their palette).
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl RenderOnce for Separator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self.color.unwrap_or_else(|| separator::color(cx.notra()));
        let rule = div().flex_none().bg(color);
        match self.orientation {
            Orientation::Horizontal => rule.h(px(separator::THICKNESS)).w_full(),
            Orientation::Vertical => rule.w(px(separator::THICKNESS)).h_full(),
        }
    }
}
