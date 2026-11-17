use ::gpui::{
    App, ClickEvent, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*, px, svg,
};

use crate::components::primitives::badge::{self, BadgeSkin, BadgeVariant};
use crate::fonts;
use crate::gpui::ActiveTheme;
use crate::icons::Icon;

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// shadcn's badge: `Badge::new("New").variant(BadgeVariant::Secondary)`.
#[derive(IntoElement)]
pub struct Badge {
    label: SharedString,
    variant: BadgeVariant,
    icon: Option<Icon>,
    icon_end: Option<Icon>,
    link: Option<(ElementId, ClickHandler)>,
}

impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::Default,
            icon: None,
            icon_end: None,
            link: None,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// A glyph before the label (`data-icon="inline-start"`).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// A glyph after the label (`data-icon="inline-end"`).
    pub fn icon_end(mut self, icon: Icon) -> Self {
        self.icon_end = Some(icon);
        self
    }

    /// Renders the badge as a link (`render={<a />}`), which turns on its hover style.
    pub fn on_click(
        mut self,
        id: impl Into<ElementId>,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.link = Some((id.into(), Box::new(handler)));
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let skin = BadgeSkin::new(cx.notra(), self.variant);
        let glyph = |icon: Icon| {
            svg()
                .path(icon.path())
                .size(px(badge::ICON_SIZE))
                .flex_none()
                .text_color(skin.text)
        };
        let hover = skin.hover_for(self.link.is_some());
        let underline = skin.underline_on_hover;
        let body = div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(badge::GAP))
            .h(px(badge::HEIGHT))
            .pl(px(if self.icon.is_some() {
                badge::PADDING_X_ICON
            } else {
                badge::PADDING_X
            }))
            .pr(px(if self.icon_end.is_some() {
                badge::PADDING_X_ICON
            } else {
                badge::PADDING_X
            }))
            .overflow_hidden()
            .rounded(px(badge::RADIUS))
            .border_1()
            .border_color(skin.border)
            .bg(skin.fill)
            .text_color(skin.text)
            .font_family(fonts::SANS)
            .text_size(px(badge::FONT_SIZE))
            .line_height(px(badge::LINE_HEIGHT))
            .font_weight(FontWeight(badge::WEIGHT))
            .whitespace_nowrap()
            .when_some(self.icon, |d, icon| d.child(glyph(icon)))
            .child(self.label)
            .when_some(self.icon_end, |d, icon| d.child(glyph(icon)))
            .when_some(hover, |d, (fill, text)| {
                d.hover(move |s| s.bg(fill).text_color(text))
            })
            .when(underline, |d| d.hover(|s| s.underline()));
        match self.link {
            Some((id, handler)) => body
                .id(id)
                .cursor_pointer()
                .on_click(handler)
                .into_any_element(),
            None => body.into_any_element(),
        }
    }
}
