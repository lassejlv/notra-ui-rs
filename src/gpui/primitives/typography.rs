use ::gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, px,
};

use crate::components::primitives::typography::{
    HeadingSize, TextSize, TextTone, TextWeight, WIDE_BREAKPOINT,
};
use crate::fonts;
use crate::gpui::ActiveTheme;

/// A display heading in Satoshi (Inter when Satoshi isn't registered):
/// `Heading::new("Ship faster").size(HeadingSize::Hero)`.
#[derive(IntoElement)]
pub struct Heading {
    text: SharedString,
    size: HeadingSize,
    wide: Option<bool>,
}

impl Heading {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            size: HeadingSize::Section,
            wide: None,
        }
    }

    pub fn size(mut self, size: HeadingSize) -> Self {
        self.size = size;
        self
    }

    /// Pins the responsive step; by default it follows the window width.
    pub fn wide(mut self, wide: bool) -> Self {
        self.wide = Some(wide);
        self
    }
}

impl RenderOnce for Heading {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let wide = self
            .wide
            .unwrap_or_else(|| f32::from(window.viewport_size().width) >= WIDE_BREAKPOINT);
        let m = self.size.metrics(wide);
        let display = window
            .text_system()
            .all_font_names()
            .iter()
            .any(|name| name == fonts::DISPLAY);
        div()
            .font_family(if display { fonts::DISPLAY } else { fonts::SANS })
            .text_size(px(m.size))
            .line_height(px(m.line_height))
            .font_weight(FontWeight(m.weight))
            .child(self.text)
    }
}

/// Body copy in Inter: `Text::new("…").size(TextSize::Sm).tone(TextTone::Muted)`.
/// Tone and weight inherit when unset.
#[derive(IntoElement)]
pub struct Text {
    text: SharedString,
    size: TextSize,
    tone: Option<TextTone>,
    weight: Option<TextWeight>,
}

impl Text {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            size: TextSize::Base,
            tone: None,
            weight: None,
        }
    }

    pub fn size(mut self, size: TextSize) -> Self {
        self.size = size;
        self
    }

    pub fn tone(mut self, tone: TextTone) -> Self {
        self.tone = Some(tone);
        self
    }

    pub fn weight(mut self, weight: TextWeight) -> Self {
        self.weight = Some(weight);
        self
    }
}

impl RenderOnce for Text {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.notra();
        let m = self.size.metrics();
        let mut d = div()
            .font_family(fonts::SANS)
            .text_size(px(m.size))
            .line_height(px(m.line_height));
        if let Some(tone) = self.tone {
            d = d.text_color(tone.color(theme));
        }
        if let Some(weight) = self.weight {
            d = d.font_weight(FontWeight(weight.value()));
        }
        d.child(self.text)
    }
}
