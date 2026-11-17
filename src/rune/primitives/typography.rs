use ::rune::{El, text, tok::Family, tok::TextStyle};

use crate::components::primitives::typography::{
    HeadingSize, TextSize, TextTone, TextWeight, TypeMetrics,
};
use crate::fonts;
use crate::rune::ToRune;
use crate::theme::Theme;

fn style(m: TypeMetrics, family: &'static str, weight: f32) -> TextStyle {
    TextStyle::new(m.size, weight)
        .family(Family::Named(family))
        .lh(m.line_height)
        .tracking(m.tracking())
}

/// A display heading: `Heading::new("Ship faster").size(HeadingSize::Hero).build(&theme)`.
pub struct Heading {
    text: String,
    size: HeadingSize,
    wide: bool,
}

impl Heading {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            size: HeadingSize::Section,
            wide: true,
        }
    }

    pub fn size(mut self, size: HeadingSize) -> Self {
        self.size = size;
        self
    }

    /// The responsive step: `true` at or above 640 px (the default).
    pub fn wide(mut self, wide: bool) -> Self {
        self.wide = wide;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let m = self.size.metrics(self.wide);
        text(self.text)
            .font(style(m, fonts::DISPLAY, m.weight))
            .color(theme.foreground.rune())
            .role("heading")
            .wrap()
    }
}

/// Body copy in Inter: `Text::new("…").size(TextSize::Sm).tone(TextTone::Muted).build(&theme)`.
pub struct Text {
    text: String,
    size: TextSize,
    tone: TextTone,
    weight: TextWeight,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            size: TextSize::Base,
            tone: TextTone::Default,
            weight: TextWeight::Regular,
        }
    }

    pub fn size(mut self, size: TextSize) -> Self {
        self.size = size;
        self
    }

    pub fn tone(mut self, tone: TextTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn weight(mut self, weight: TextWeight) -> Self {
        self.weight = weight;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let m = self.size.metrics();
        text(self.text)
            .font(style(m, fonts::SANS, self.weight.value()))
            .color(self.tone.color(theme).rune())
            .wrap()
    }
}
