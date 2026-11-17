//! GPUI renderings of the primitive layer.

mod avatar;
mod badge;
mod checkbox;
mod collapsible;
mod kbd;
mod marketing_button;
mod separator;
mod shimmer;
mod skeleton;
mod spinner;
mod status_badge;
mod typography;

pub use crate::components::primitives::avatar::{AvatarShape, AvatarSize};
pub use crate::components::primitives::badge::BadgeVariant;
pub use crate::components::primitives::checkbox::CheckState;
pub use crate::components::primitives::marketing_button::{MarketingSize, MarketingVariant};
pub use crate::components::primitives::separator::Orientation;
pub use crate::components::primitives::shimmer::ShimmerSpec;
pub use crate::components::primitives::status_badge::{
    BadgeAppearance, BadgeShape, BadgeSize, StatusVariant, TrendTone,
};
pub use crate::components::primitives::typography::{HeadingSize, TextSize, TextTone, TextWeight};
pub use avatar::Avatar;
pub use badge::Badge;
pub use checkbox::Checkbox;
pub use collapsible::{Collapsible, CollapsibleChevron, CollapsiblePanel};
pub use kbd::{Kbd, KbdGroup};
pub use marketing_button::MarketingButton;
pub use separator::Separator;
pub use shimmer::Shimmer;
pub use skeleton::Skeleton;
pub use spinner::Spinner;
pub use status_badge::{StatusBadge, TrendBadge};
pub use typography::{Heading, Text};

use ::gpui::{FontFeatures, Hsla, Rgba};

use crate::theme::Color;

/// Tabular figures, for numbers that change in place.
fn tabular_nums() -> FontFeatures {
    FontFeatures(std::sync::Arc::new(vec![("tnum".into(), 1)]))
}

/// Reads a GPUI colour back into a token colour.
fn color_of(hsla: Hsla) -> Color {
    let c = Rgba::from(hsla);
    Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}
