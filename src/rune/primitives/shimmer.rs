use ::rune::{El, el, row, text, tok::TextStyle};
use ::rune_motion::{Easing, LoopSpec};

use crate::components::primitives::shimmer::{self, END, START, ShimmerSpec};
use crate::rune::{ToRune, sans};
use crate::theme::{Color, Theme};

/// Text with a highlight sweeping across it:
/// `Shimmer::new("Thinking...").color(theme.muted_foreground).build(&theme)`.
///
/// A window `2 × spread` wide, fading linearly to both edges, slides across the label
/// while a highlighted copy inside it slides back by the same amount, so the copy stays
/// on the label: the CSS gradient's triangular band.
pub struct Shimmer {
    text: String,
    font: TextStyle,
    color: Option<Color>,
    highlight: Option<Color>,
    spec: ShimmerSpec,
    paused: bool,
}

impl Shimmer {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font: sans(14.0, 400.0, 20.0),
            color: None,
            highlight: None,
            spec: ShimmerSpec::default(),
            paused: false,
        }
    }

    pub fn font(mut self, font: TextStyle) -> Self {
        self.font = font;
        self
    }

    /// The text colour (`currentColor`); the muted foreground by default.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// `--shimmer-highlight`; the foreground by default.
    pub fn highlight(mut self, color: Color) -> Self {
        self.highlight = Some(color);
        self
    }

    pub fn duration(mut self, duration: std::time::Duration) -> Self {
        self.spec.duration = duration;
        self
    }

    /// Half the band's width, in `ch`.
    pub fn spread(mut self, spread_ch: f32) -> Self {
        self.spec.spread_ch = spread_ch;
        self
    }

    /// Stops the sweep and shows plain text.
    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = paused;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let base = text(self.text.clone())
            .font(self.font)
            .color(self.color.unwrap_or(theme.muted_foreground).rune());
        if self.paused {
            return base;
        }
        let spread = self.spec.spread_px(self.font.size);
        let band = 2.0 * spread;
        let (width, _) = ::rune::text::measure(&self.text, &[], self.font, None);
        let (from, to) = (
            (width * START - spread) / band,
            (width * END - spread) / band,
        );
        let period = self.spec.duration.as_secs_f64();
        let sweep = |from: f32, to: f32| {
            LoopSpec::slide(period, [from, 0.0], [to, 0.0]).easing(Easing::LINEAR)
        };
        let lit = el()
            .absolute(0.0, 0.0)
            .w(band)
            .h_full()
            .looping(sweep(-from, -to))
            .child(
                text(self.text)
                    .font(self.font)
                    .color(
                        self.highlight
                            .unwrap_or_else(|| shimmer::highlight(theme))
                            .rune(),
                    )
                    .absolute(0.0, 0.0),
            );
        let window = el()
            .absolute(0.0, 0.0)
            .w(band)
            .h_full()
            .clip_fade_linear(spread, 0.0, spread, 0.0)
            .looping(sweep(from, to))
            .child(lit)
            .a11y_hidden();
        row().child(base).child(window)
    }
}
