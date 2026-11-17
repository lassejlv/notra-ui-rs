//! shadcn's `Separator`: a one-pixel `bg-border` rule.

use crate::theme::{Color, Theme};

/// Which way the rule runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Orientation {
    /// Full width, one pixel tall.
    #[default]
    Horizontal,
    /// One pixel wide, stretched to the row's height.
    Vertical,
}

impl Orientation {
    /// The `aria-orientation` value.
    pub fn name(self) -> &'static str {
        match self {
            Orientation::Horizontal => "horizontal",
            Orientation::Vertical => "vertical",
        }
    }
}

/// `h-px` / `w-px`.
pub const THICKNESS: f32 = 1.0;

/// The rule's colour.
pub fn color(theme: &Theme) -> Color {
    theme.border
}
