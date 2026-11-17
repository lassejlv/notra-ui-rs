use ::rune::{El, Pixels, SurfaceFit, el, row, text};

use crate::components::primitives::avatar::{self, AvatarShape, AvatarSize, AvatarSkin};
use crate::rune::{ToRune, sans};
use crate::theme::Theme;

/// An avatar: `Avatar::new("LV").image(&pixels).build(&theme)`. The fallback shows
/// under the image until it decodes.
pub struct Avatar {
    fallback: String,
    image: Option<Pixels>,
    size: AvatarSize,
    shape: AvatarShape,
}

impl Avatar {
    pub fn new(fallback: impl Into<String>) -> Self {
        Self {
            fallback: fallback.into(),
            image: None,
            size: AvatarSize::Default,
            shape: AvatarShape::Circle,
        }
    }

    /// Pixels from an `ImageCache` (or set directly).
    pub fn image(mut self, pixels: &Pixels) -> Self {
        self.image = Some(pixels.clone());
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

    pub fn build(self, theme: &Theme) -> El {
        let skin = AvatarSkin::new(theme);
        let d = self.size.diameter();
        let r = self.shape.radius(d);
        let (font_size, line_height) = self.size.font();
        let ring = if self.image.is_some() {
            AvatarSkin::image_ring(theme.is_dark())
        } else {
            skin.ring_over(skin.fallback_fill, theme.is_dark())
        };
        let pin = |e: El| e.pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0));
        let mut e = el()
            .size(d, d)
            .radius(r)
            .clip_rounded(r)
            .role("img")
            .label(self.fallback.clone())
            .child(
                pin(row()).center().fill(skin.fallback_fill.rune()).child(
                    text(self.fallback)
                        .font(sans(font_size, 400.0, line_height))
                        .color(skin.fallback_text.rune()),
                ),
            );
        if let Some(pixels) = &self.image {
            e = e.child(pin(el()).pixels_fit(pixels, SurfaceFit::Fill).radius(r));
        }
        e.child(pin(el()).radius(r).border(avatar::RING_WIDTH, ring.rune()))
    }
}
