//! shadcn's `Badge` (base-nova): a 20 px pill in six variants.

use crate::theme::{Color, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}

impl BadgeVariant {
    pub const ALL: [BadgeVariant; 6] = [
        BadgeVariant::Default,
        BadgeVariant::Secondary,
        BadgeVariant::Destructive,
        BadgeVariant::Outline,
        BadgeVariant::Ghost,
        BadgeVariant::Link,
    ];
}

/// `h-5`.
pub const HEIGHT: f32 = 20.0;
/// `px-2`.
pub const PADDING_X: f32 = 8.0;
/// `has-data-[icon=inline-*]:p*-1.5`: the side with an icon.
pub const PADDING_X_ICON: f32 = 6.0;
/// `gap-1`.
pub const GAP: f32 = 4.0;
/// `rounded-4xl`.
pub const RADIUS: f32 = 32.0;
/// `text-xs font-medium`.
pub const FONT_SIZE: f32 = 12.0;
pub const LINE_HEIGHT: f32 = 16.0;
pub const WEIGHT: f32 = 500.0;
/// `[&>svg]:size-3!`.
pub const ICON_SIZE: f32 = 12.0;

/// A variant's resolved paint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BadgeSkin {
    pub fill: Color,
    pub border: Color,
    pub text: Color,
    /// Fill and text under the pointer.
    pub hover: Option<(Color, Color)>,
    /// Whether [`Self::hover`] needs the badge to be a link (`[a]:hover:`).
    pub hover_needs_link: bool,
    /// `hover:underline` (the link variant).
    pub underline_on_hover: bool,
}

impl BadgeSkin {
    pub fn new(theme: &Theme, variant: BadgeVariant) -> Self {
        let dark = theme.is_dark();
        let plain = |fill: Color, text: Color, hover| Self {
            fill,
            border: Color::TRANSPARENT,
            text,
            hover,
            hover_needs_link: true,
            underline_on_hover: false,
        };
        match variant {
            BadgeVariant::Default => plain(
                theme.primary,
                theme.primary_foreground,
                Some((theme.primary.fade(0.8), theme.primary_foreground)),
            ),
            BadgeVariant::Secondary => plain(
                theme.secondary,
                theme.secondary_foreground,
                Some((theme.secondary.fade(0.8), theme.secondary_foreground)),
            ),
            BadgeVariant::Destructive => plain(
                theme.destructive.fade(if dark { 0.2 } else { 0.1 }),
                theme.destructive,
                Some((theme.destructive.fade(0.2), theme.destructive)),
            ),
            BadgeVariant::Outline => Self {
                border: theme.border,
                ..plain(
                    Color::TRANSPARENT,
                    theme.foreground,
                    Some((theme.muted, theme.muted_foreground)),
                )
            },
            BadgeVariant::Ghost => Self {
                hover_needs_link: false,
                ..plain(
                    Color::TRANSPARENT,
                    theme.foreground,
                    Some((
                        if dark {
                            theme.muted.fade(0.5)
                        } else {
                            theme.muted
                        },
                        theme.muted_foreground,
                    )),
                )
            },
            BadgeVariant::Link => Self {
                hover_needs_link: false,
                underline_on_hover: true,
                ..plain(Color::TRANSPARENT, theme.primary, None)
            },
        }
    }

    /// The hover paint for a badge that is (`link`) or isn't a link.
    pub fn hover_for(&self, link: bool) -> Option<(Color, Color)> {
        self.hover.filter(|_| link || !self.hover_needs_link)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_ghost_and_link_react_without_a_link() {
        let theme = Theme::light();
        for variant in BadgeVariant::ALL {
            let skin = BadgeSkin::new(&theme, variant);
            let reacts = skin.hover_for(false).is_some() || skin.underline_on_hover;
            assert_eq!(
                reacts,
                matches!(variant, BadgeVariant::Ghost | BadgeVariant::Link),
                "{variant:?}"
            );
        }
    }
}
