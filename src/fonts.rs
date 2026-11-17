//! Fonts bundled with the crate, all under the SIL Open Font License (see `assets/fonts`).
//!
//! Notra sets display type in Satoshi, which Fontshare's licence keeps out of
//! this crate. Register it yourself under the family name [`DISPLAY`] and the
//! headings pick it up; without it they fall back to Inter, as the site does.

/// Body and UI text.
pub const SANS: &str = "Inter";
/// Headlines. Falls back to [`SANS`] when Satoshi isn't registered.
pub const DISPLAY: &str = "Satoshi";
/// Code, commands and terminal blocks.
pub const MONO: &str = "Geist Mono";
/// The instrument card's serif numerals.
pub const SERIF: &str = "Instrument Serif";

pub const INTER_REGULAR: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
pub const INTER_ITALIC: &[u8] = include_bytes!("../assets/fonts/Inter-Italic.ttf");
pub const INTER_MEDIUM: &[u8] = include_bytes!("../assets/fonts/Inter-Medium.ttf");
pub const INTER_SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/Inter-SemiBold.ttf");
pub const INTER_BOLD: &[u8] = include_bytes!("../assets/fonts/Inter-Bold.ttf");
pub const GEIST_MONO_REGULAR: &[u8] = include_bytes!("../assets/fonts/GeistMono-Regular.ttf");
pub const GEIST_MONO_MEDIUM: &[u8] = include_bytes!("../assets/fonts/GeistMono-Medium.ttf");
pub const GEIST_MONO_SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/GeistMono-SemiBold.ttf");
pub const INSTRUMENT_SERIF_REGULAR: &[u8] =
    include_bytes!("../assets/fonts/InstrumentSerif-Regular.ttf");
pub const INSTRUMENT_SERIF_ITALIC: &[u8] =
    include_bytes!("../assets/fonts/InstrumentSerif-Italic.ttf");

/// Every bundled face.
pub const ALL: &[&[u8]] = &[
    INTER_REGULAR,
    INTER_ITALIC,
    INTER_MEDIUM,
    INTER_SEMIBOLD,
    INTER_BOLD,
    GEIST_MONO_REGULAR,
    GEIST_MONO_MEDIUM,
    GEIST_MONO_SEMIBOLD,
    INSTRUMENT_SERIF_REGULAR,
    INSTRUMENT_SERIF_ITALIC,
];
