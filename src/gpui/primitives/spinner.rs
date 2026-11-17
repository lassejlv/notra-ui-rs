use std::time::Duration;

use ::gpui::{
    Animation, AnimationExt, App, ElementId, Hsla, IntoElement, RenderOnce, Styled, Transformation,
    Window, percentage, px, svg,
};

use crate::components::primitives::{SPIN, spinner};
use crate::icons::Icon;
use crate::theme::Color;

/// shadcn's spinner: `Spinner::new("saving")`, in the surrounding text colour.
#[derive(IntoElement)]
pub struct Spinner {
    id: ElementId,
    size: f32,
    color: Option<Color>,
}

impl Spinner {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            size: spinner::SIZE,
            color: None,
        }
    }

    /// Diameter in px (`size-4` by default).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl RenderOnce for Spinner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self
            .color
            .map_or_else(|| window.text_style().color, Hsla::from);
        spinning_icon(self.id, spinner::ICON, self.size, color, cx.reduce_motion())
    }
}

/// A glyph on `animate-spin`, slowed to 1.5 s a turn under Reduce Motion as Notra's
/// spinners are.
pub(super) fn spinning_icon(
    id: ElementId,
    icon: Icon,
    size: f32,
    color: Hsla,
    reduce_motion: bool,
) -> impl IntoElement {
    let period = if reduce_motion {
        Duration::from_millis(1500)
    } else {
        SPIN.period
    };
    svg()
        .path(icon.path())
        .size(px(size))
        .flex_none()
        .text_color(color)
        .with_animation(id, Animation::new(period).repeat(), |svg, t| {
            svg.with_transformation(Transformation::rotate(percentage(t)))
        })
}
