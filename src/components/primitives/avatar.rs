//! shadcn's `Avatar`: an image or a fallback in a circle, with a hairline ring.

use crate::theme::{Color, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AvatarSize {
    Sm,
    #[default]
    Default,
    Lg,
}

impl AvatarSize {
    /// `size-6`, `size-8`, `size-10`.
    pub fn diameter(self) -> f32 {
        match self {
            AvatarSize::Sm => 24.0,
            AvatarSize::Default => 32.0,
            AvatarSize::Lg => 40.0,
        }
    }

    /// The fallback's `text-sm`, or `text-xs` when small: `(size, line height)`.
    pub fn font(self) -> (f32, f32) {
        match self {
            AvatarSize::Sm => (12.0, 16.0),
            _ => (14.0, 20.0),
        }
    }
}

/// The corner treatment. Blocks round some avatars (`rounded-lg` in the sidebar).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum AvatarShape {
    #[default]
    Circle,
    Rounded(f32),
}

impl AvatarShape {
    pub fn radius(self, diameter: f32) -> f32 {
        match self {
            AvatarShape::Circle => diameter / 2.0,
            AvatarShape::Rounded(r) => r,
        }
    }
}

/// An avatar's paint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarSkin {
    /// The fallback's `bg-muted`.
    pub fallback_fill: Color,
    /// The fallback's `text-muted-foreground`.
    pub fallback_text: Color,
    /// The `after:` ring, blended `darken` in light and `lighten` in dark.
    pub ring: Color,
}

impl AvatarSkin {
    pub fn new(theme: &Theme) -> Self {
        Self {
            fallback_fill: theme.muted,
            fallback_text: theme.muted_foreground,
            ring: theme.border,
        }
    }

    /// The ring as painted over `under`: CSS `mix-blend-mode: darken` (light) or
    /// `lighten` (dark) per channel.
    pub fn ring_over(&self, under: Color, dark: bool) -> Color {
        let pick = |a: f32, b: f32| if dark { a.max(b) } else { a.min(b) };
        Color {
            r: pick(self.ring.r, under.r),
            g: pick(self.ring.g, under.g),
            b: pick(self.ring.b, under.b),
            a: 1.0,
        }
    }
}

/// The ring's width.
pub const RING_WIDTH: f32 = 1.0;

/// Initials for a fallback: the first letter of the first two words, uppercased.
pub fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .flat_map(char::to_uppercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_take_two_words() {
        assert_eq!(initials("lasse vestergaard jensen"), "LV");
        assert_eq!(initials("Notra"), "N");
        assert_eq!(initials(""), "");
    }

    #[test]
    fn ring_darkens_in_light() {
        let skin = AvatarSkin::new(&Theme::light());
        let over = skin.ring_over(Color::rgb(0x202020), false);
        assert_eq!(over.r, Color::rgb(0x202020).r);
    }
}
