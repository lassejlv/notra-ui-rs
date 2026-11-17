use ::gpui::{
    Animation, AnimationExt, App, Div, ElementId, IntoElement, RenderOnce, StyleRefinement, Styled,
    Window, div, px,
};

use crate::components::primitives::skeleton;
use crate::gpui::ActiveTheme;

/// A pulsing placeholder; size it like a div: `Skeleton::new("line").h(px(16.)).w(px(120.))`.
#[derive(IntoElement)]
pub struct Skeleton {
    id: ElementId,
    base: Div,
}

impl Skeleton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            base: div().flex_none().rounded(px(skeleton::RADIUS)),
        }
    }
}

impl Styled for Skeleton {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Skeleton {
    fn render(mut self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let fill = skeleton::fill(cx.notra());
        let base = if self.base.style().background.is_none() {
            self.base.bg(fill)
        } else {
            self.base
        };
        if cx.reduce_motion() {
            return base.into_any_element();
        }
        let period = skeleton::PULSE.period;
        base.with_animation(self.id, Animation::new(period).repeat(), move |d, t| {
            d.opacity(skeleton::opacity(period.mul_f32(t)))
        })
        .into_any_element()
    }
}
