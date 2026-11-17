//! shadcn's `Spinner`: Lucide's loader circle, turning once a second
//! ([`super::SPIN`]).

use crate::icons::Icon;

/// The glyph (`Loader2Icon`).
pub const ICON: Icon = Icon::LoaderCircle;
/// `size-4`.
pub const SIZE: f32 = 16.0;
/// `aria-label` of the `role="status"` element.
pub const LABEL: &str = "Loading";
