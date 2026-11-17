//! GPUI front end.
//!
//! Call [`init`] once at startup to register the bundled fonts and the theme,
//! and wrap your asset source in [`Assets`] so icons resolve:
//!
//! ```no_run
//! use gpui::{App, AppContext as _, WindowOptions};
//! use notra_ui::gpui::{Assets, Button, init};
//!
//! gpui_platform::application()
//!     .with_assets(Assets(()))
//!     .run(|cx: &mut App| {
//!         init(cx);
//!         // cx.open_window(..)
//!     });
//! ```

use std::borrow::Cow;

use ::gpui::{
    App, AssetSource, Background, BoxShadow, ColorSpace, Fill, Global, Hsla, Rgba, SharedString,
    linear_color_stop, linear_gradient, point, px,
};

use crate::fonts;
use crate::icons::Icon;
use crate::theme::{Appearance, Color, Shadow, Theme};

mod button;
mod composer;
mod input;

pub use crate::components::button::{ButtonSize, ButtonVariant};
pub use button::Button;
pub use composer::{
    ChipLead, ComposerChip, ComposerFrame, ComposerInput, ComposerNotice, ComposerNudge,
    ComposerSend, ComposerToolbar, ComposerToolbarButton,
};
pub use input::{
    Input, InputGroup, InputGroupAddon, InputGroupButton, TextField, TextFieldEvent, Textarea,
};

static LIGHT: Theme = Theme::light();

/// The active Notra theme, stored as a GPUI global.
pub struct NotraTheme(pub Theme);

impl Global for NotraTheme {}

/// Registers the bundled fonts and installs the light theme unless one is set.
pub fn init(cx: &mut App) {
    cx.text_system()
        .add_fonts(fonts::ALL.iter().map(|face| Cow::Borrowed(*face)).collect())
        .expect("the bundled fonts are valid");
    input::bind_keys(cx);
    if !cx.has_global::<NotraTheme>() {
        cx.set_global(NotraTheme(Theme::light()));
    }
}

/// Switches every window to `appearance`.
pub fn set_appearance(cx: &mut App, appearance: Appearance) {
    cx.set_global(NotraTheme(Theme::new(appearance)));
    cx.refresh_windows();
}

/// Reads the active theme: `cx.notra().primary`.
pub trait ActiveTheme {
    fn notra(&self) -> &Theme;
}

impl ActiveTheme for App {
    fn notra(&self) -> &Theme {
        self.try_global::<NotraTheme>()
            .map_or(&LIGHT, |theme| &theme.0)
    }
}

impl From<Color> for Rgba {
    fn from(c: Color) -> Self {
        Rgba {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}

impl From<Color> for Hsla {
    fn from(c: Color) -> Self {
        Rgba::from(c).into()
    }
}

impl From<Color> for Background {
    fn from(c: Color) -> Self {
        Hsla::from(c).into()
    }
}

impl From<Color> for Fill {
    fn from(c: Color) -> Self {
        Background::from(c).into()
    }
}

/// A top-to-bottom gradient interpolated in Oklab, as Tailwind's `bg-linear-to-b`.
pub fn gradient(top: Color, bottom: Color) -> Background {
    linear_gradient(
        180.0,
        linear_color_stop(top, 0.0),
        linear_color_stop(bottom, 1.0),
    )
    .color_space(ColorSpace::Oklab)
}

/// Converts CSS shadow layers to GPUI's.
pub fn shadows(layers: &[Shadow]) -> Vec<BoxShadow> {
    layers
        .iter()
        .map(|s| BoxShadow {
            color: s.color.into(),
            offset: point(px(s.x), px(s.y)),
            blur_radius: px(s.blur),
            spread_radius: px(s.spread),
            inset: s.inset,
        })
        .collect()
}

/// Serves the crate's icons under `notra/icons/` and defers everything else to `A`.
#[derive(Default)]
pub struct Assets<A = ()>(pub A);

impl<A: AssetSource> AssetSource for Assets<A> {
    fn load(&self, path: &str) -> ::gpui::Result<Option<Cow<'static, [u8]>>> {
        let icon = path
            .strip_prefix("notra/icons/")
            .and_then(|file| file.strip_suffix(".svg"))
            .and_then(Icon::from_name);
        match icon {
            Some(icon) => Ok(Some(Cow::Borrowed(icon.svg().as_bytes()))),
            None => self.0.load(path),
        }
    }

    fn list(&self, path: &str) -> ::gpui::Result<Vec<SharedString>> {
        if path.trim_end_matches('/') == "notra/icons" {
            return Ok(Icon::ALL
                .iter()
                .map(|icon| SharedString::from(icon.path()))
                .collect());
        }
        self.0.list(path)
    }
}
