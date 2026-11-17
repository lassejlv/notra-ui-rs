use std::ops::Range;

use ::gpui::{
    Animation, AnimationExt, App, ElementId, HighlightStyle, IntoElement, ParentElement,
    RenderOnce, SharedString, StyledText, TextRun, Window, div,
};

use super::color_of;
use crate::components::primitives::shimmer::{self, END, START, ShimmerSpec};
use crate::gpui::ActiveTheme;
use crate::theme::Color;

/// Text with a highlight sweeping across it: `Shimmer::new("thinking", "Thinking...")`.
/// It takes the surrounding font and colour, as the CSS one does.
#[derive(IntoElement)]
pub struct Shimmer {
    id: ElementId,
    text: SharedString,
    spec: ShimmerSpec,
    paused: bool,
    highlight: Option<Color>,
}

impl Shimmer {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            spec: ShimmerSpec::default(),
            paused: false,
            highlight: None,
        }
    }

    /// One sweep's length.
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

    /// `--shimmer-highlight`; the foreground by default.
    pub fn highlight(mut self, color: Color) -> Self {
        self.highlight = Some(color);
        self
    }
}

impl RenderOnce for Shimmer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.paused || cx.reduce_motion() || self.text.is_empty() {
            return div().child(self.text).into_any_element();
        }
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let base = color_of(style.color);
        let highlight = self
            .highlight
            .unwrap_or_else(|| shimmer::highlight(cx.notra()));
        let run = TextRun {
            len: self.text.len(),
            font: style.font(),
            color: style.color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = window
            .text_system()
            .shape_line(self.text.clone(), font_size, &[run], None);
        let width = f32::from(line.width());
        let glyphs: Vec<(Range<usize>, f32)> = self
            .text
            .char_indices()
            .map(|(start, ch)| {
                let end = start + ch.len_utf8();
                let x =
                    (f32::from(line.x_for_index(start)) + f32::from(line.x_for_index(end))) / 2.0;
                (start..end, x)
            })
            .collect();
        let spread = self.spec.spread_px(f32::from(font_size));
        let text = self.text;
        div()
            .with_animation(
                self.id,
                Animation::new(self.spec.duration).repeat(),
                move |d, t| {
                    let center = width * (START + (END - START) * t);
                    let highlights = glyphs.iter().map(|(range, x)| {
                        let paint =
                            shimmer::paint(base, highlight, shimmer::intensity(*x, center, spread));
                        (
                            range.clone(),
                            HighlightStyle {
                                color: Some(paint.into()),
                                ..HighlightStyle::default()
                            },
                        )
                    });
                    d.child(StyledText::new(text.clone()).with_highlights(highlights))
                },
            )
            .into_any_element()
    }
}
