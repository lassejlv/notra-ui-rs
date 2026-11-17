//! Notra UI for native Rust apps.
//!
//! A port of [Notra UI](https://ui.usenotra.com) with the same tokens, surfaces
//! and motion, rendered by either [GPUI](https://www.gpui.rs) or
//! [Rune](https://github.com/anomalyco/rune). Pick one front end with a Cargo
//! feature:
//!
//! - `gpui`: [`gpui`] module, `RenderOnce` components and stateful views.
//! - `rune`: [`rune`] module, builders that return `rune::El`.
//!
//! The framework-agnostic parts (theme, motion curves, component models and
//! demo data) are always available.

#[cfg(all(feature = "gpui", feature = "rune"))]
compile_error!(
    "enable one of the `gpui` and `rune` features per build: their font stacks link fontconfig in incompatible ways"
);

pub mod components;
pub mod fonts;
pub mod icons;
pub mod motion;
pub mod theme;

#[cfg(feature = "gpui")]
pub mod gpui;
#[cfg(feature = "rune")]
pub mod rune;

pub use icons::Icon;
pub use theme::{Appearance, Color, Shadow, Theme};
