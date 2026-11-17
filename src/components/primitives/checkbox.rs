//! shadcn's `Checkbox` (Base UI): a 16 px box that fills with the primary colour.

use crate::icons::Icon;
use crate::theme::{Color, Theme};

/// Checked, unchecked, or mixed (`indeterminate`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum CheckState {
    #[default]
    Unchecked,
    Checked,
    Indeterminate,
}

impl CheckState {
    pub fn is_on(self) -> bool {
        self != CheckState::Unchecked
    }

    /// What a press does: mixed and unchecked boxes check, checked ones clear.
    pub fn toggled(self) -> CheckState {
        match self {
            CheckState::Checked => CheckState::Unchecked,
            _ => CheckState::Checked,
        }
    }

    /// The indicator glyph, if any.
    pub fn icon(self) -> Option<Icon> {
        match self {
            CheckState::Unchecked => None,
            CheckState::Checked => Some(Icon::Check),
            CheckState::Indeterminate => Some(Icon::Minus),
        }
    }
}

impl From<bool> for CheckState {
    fn from(on: bool) -> Self {
        if on {
            CheckState::Checked
        } else {
            CheckState::Unchecked
        }
    }
}

/// `size-4`.
pub const SIZE: f32 = 16.0;
/// `rounded-[4px]`.
pub const RADIUS: f32 = 4.0;
/// `[&>svg]:size-3.5`.
pub const ICON_SIZE: f32 = 14.0;
/// The invisible hit area reaches this far past the box (`after:-inset-x-3 -inset-y-2`).
pub const HIT_SLOP: (f32, f32) = (12.0, 8.0);
/// `disabled:opacity-50`.
pub const DISABLED_OPACITY: f32 = 0.5;

/// The box's paint in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckboxSkin {
    pub fill: Color,
    pub border: Color,
    pub mark: Color,
}

impl CheckboxSkin {
    pub fn new(theme: &Theme, state: CheckState) -> Self {
        if state.is_on() {
            return Self {
                fill: theme.primary,
                border: theme.primary,
                mark: theme.primary_foreground,
            };
        }
        Self {
            fill: if theme.is_dark() {
                theme.input.fade(0.3)
            } else {
                Color::TRANSPARENT
            },
            border: theme.input,
            mark: theme.primary_foreground,
        }
    }

    /// Keyboard focus swaps the border for the ring colour (checked boxes keep theirs).
    pub fn focused(self, theme: &Theme, state: CheckState) -> Self {
        if state.is_on() {
            return self;
        }
        Self {
            border: theme.ring,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggling_resolves_mixed_to_checked() {
        assert_eq!(CheckState::Indeterminate.toggled(), CheckState::Checked);
        assert_eq!(CheckState::Checked.toggled(), CheckState::Unchecked);
    }

    #[test]
    fn checked_fills_with_primary() {
        let theme = Theme::light();
        assert_eq!(CheckboxSkin::new(&theme, CheckState::Checked).fill, theme.primary);
        assert_eq!(
            CheckboxSkin::new(&theme, CheckState::Unchecked).fill,
            Color::TRANSPARENT
        );
    }
}
