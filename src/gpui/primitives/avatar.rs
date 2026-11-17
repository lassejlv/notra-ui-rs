use ::gpui::{
    App, ImageSource, IntoElement, ObjectFit, ParentElement, RenderOnce, SharedString, Styled,
    StyledImage, Window, div, img, prelude::*, px,
};

use crate::components::primitives::avatar::{self, AvatarShape, AvatarSize, AvatarSkin};
use crate::fonts;
use crate::gpui::ActiveTheme;

/// An avatar: `Avatar::new("LV").image("https://…/me.png")`. The fallback shows until
/// the image loads, and stays if it fails.
#[derive(IntoElement)]
pub struct Avatar {
    fallback: SharedString,
    image: Option<ImageSource>,
    size: AvatarSize,
    shape: AvatarShape,
}

impl Avatar {
    /// `fallback` is usually initials (see [`crate::components::primitives::avatar::initials`]).
    pub fn new(fallback: impl Into<SharedString>) -> Self {
        Self {
            fallback: fallback.into(),
            image: None,
            size: AvatarSize::Default,
            shape: AvatarShape::Circle,
        }
    }

    pub fn image(mut self, source: impl Into<ImageSource>) -> Self {
        self.image = Some(source.into());
        self
    }

    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    pub fn shape(mut self, shape: AvatarShape) -> Self {
        self.shape = shape;
        self
    }
}

impl RenderOnce for Avatar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.notra();
        let skin = AvatarSkin::new(theme);
        let d = self.size.diameter();
        let r = px(self.shape.radius(d));
        let (font_size, line_height) = self.size.font();
        let ring = if self.image.is_some() {
            AvatarSkin::image_ring(theme.is_dark())
        } else {
            skin.ring_over(skin.fallback_fill, theme.is_dark())
        };
        div()
            .relative()
            .flex()
            .flex_none()
            .size(px(d))
            .rounded(r)
            .child(
                div()
                    .flex()
                    .size_full()
                    .items_center()
                    .justify_center()
                    .rounded(r)
                    .bg(skin.fallback_fill)
                    .text_color(skin.fallback_text)
                    .font_family(fonts::SANS)
                    .text_size(px(font_size))
                    .line_height(px(line_height))
                    .child(self.fallback),
            )
            .when_some(self.image, |d, source| {
                d.child(
                    img(source)
                        .absolute()
                        .inset_0()
                        .size_full()
                        .rounded(r)
                        .object_fit(ObjectFit::Cover),
                )
            })
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded(r)
                    .border(px(avatar::RING_WIDTH))
                    .border_color(ring),
            )
    }
}
