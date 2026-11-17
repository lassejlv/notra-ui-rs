//! shadcn's `Kbd` and `KbdGroup`: a muted keycap.

use crate::theme::{Color, Theme};

/// `h-5`.
pub const HEIGHT: f32 = 20.0;
/// `min-w-5`.
pub const MIN_WIDTH: f32 = 20.0;
/// `px-1`.
pub const PADDING_X: f32 = 4.0;
/// `gap-1` between glyphs, and between keys of a group.
pub const GAP: f32 = 4.0;
/// `rounded-sm`.
pub const RADIUS: f32 = 4.0;
/// `text-xs`.
pub const FONT_SIZE: f32 = 12.0;
pub const LINE_HEIGHT: f32 = 16.0;
/// `font-medium`.
pub const WEIGHT: f32 = 500.0;
/// `[&_svg]:size-3`.
pub const ICON_SIZE: f32 = 12.0;

/// A keycap's paint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KbdSkin {
    pub fill: Color,
    pub text: Color,
}

impl KbdSkin {
    /// `bg-muted text-muted-foreground`.
    pub fn new(theme: &Theme) -> Self {
        Self {
            fill: theme.muted,
            text: theme.muted_foreground,
        }
    }

    /// Inside a tooltip: `bg-background/20 text-background` (`/10` in dark).
    pub fn in_tooltip(theme: &Theme) -> Self {
        Self {
            fill: theme.background.fade(if theme.is_dark() { 0.1 } else { 0.2 }),
            text: theme.background,
        }
    }
}
