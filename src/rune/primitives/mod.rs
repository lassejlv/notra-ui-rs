//! Rune renderings of the primitive layer.

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
pub use collapsible::{Collapsible, CollapsibleChevron};
pub use kbd::{Kbd, KbdGroup};
pub use marketing_button::MarketingButton;
pub use separator::Separator;
pub use shimmer::Shimmer;
pub use skeleton::Skeleton;
pub use spinner::Spinner;
pub use status_badge::{StatusBadge, TrendBadge};
pub use typography::{Heading, Text};

use ::rune_motion::{LoopSpec, Spring, SpringPair, Transition};

use crate::components::button::loading::{SWAP_BLUR, SWAP_IN, SWAP_OUT};
use crate::components::primitives::SPIN;

/// `animate-spin` as a Rune loop.
fn spin() -> LoopSpec {
    LoopSpec::rotation(SPIN.period.as_secs_f64())
}

/// The loading swap shared with the button: in from `travel` below, out through
/// the top, blurring and fading at the far ends.
pub(crate) fn swap_transition(travel: f32) -> Transition {
    Transition::asymmetric(
        Transition::offset(0.0, travel).combined(Transition::blur(SWAP_BLUR)),
        Transition::offset(0.0, -travel).combined(Transition::blur(SWAP_BLUR)),
    )
    .springs(SpringPair::new(
        Spring::new(SWAP_IN.duration.as_secs_f32(), 0.0),
        Spring::new(SWAP_OUT.duration.as_secs_f32(), 0.0),
    ))
}
