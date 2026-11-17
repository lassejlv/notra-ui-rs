use ::rune::{El, icon, row, text};

use crate::components::primitives::kbd::{self, KbdSkin};
use crate::icons::Icon;
use crate::rune::{ToRune, sans};
use crate::theme::Theme;

enum Cap {
    Label(String),
    Glyph(Icon),
}

/// A keycap: `Kbd::new("⌘").build(&theme)`.
pub struct Kbd {
    cap: Cap,
    in_tooltip: bool,
}

impl Kbd {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            cap: Cap::Label(label.into()),
            in_tooltip: false,
        }
    }

    /// A keycap holding a glyph at `size-3`.
    pub fn icon(icon: Icon) -> Self {
        Self {
            cap: Cap::Glyph(icon),
            in_tooltip: false,
        }
    }

    /// The inverted keycap used inside tooltips.
    pub fn in_tooltip(mut self) -> Self {
        self.in_tooltip = true;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let skin = if self.in_tooltip {
            KbdSkin::in_tooltip(theme)
        } else {
            KbdSkin::new(theme)
        };
        let ink = skin.text.rune();
        let (content, label) = match self.cap {
            Cap::Label(label) => (
                text(label.clone())
                    .font(sans(kbd::FONT_SIZE, kbd::WEIGHT, kbd::LINE_HEIGHT))
                    .color(ink),
                label,
            ),
            Cap::Glyph(glyph) => (
                icon(glyph.rune(), kbd::ICON_SIZE).color(ink),
                glyph.name().replace('-', " "),
            ),
        };
        row()
            .h(kbd::HEIGHT)
            .min_w(kbd::MIN_WIDTH)
            .px(kbd::PADDING_X)
            .gap(kbd::GAP)
            .center_y()
            .justify_center()
            .radius(kbd::RADIUS)
            .fill(skin.fill.rune())
            .label(label)
            .child(content)
    }
}

/// Keys pressed together.
pub struct KbdGroup;

impl KbdGroup {
    pub fn build(keys: impl IntoIterator<Item = El>) -> El {
        row().gap(kbd::GAP).center_y().children(keys)
    }
}
