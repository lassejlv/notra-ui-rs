use ::rune::{El, el, icon, row, text};

use super::spin;
use crate::components::primitives::status_badge::{
    self, BadgeAppearance, BadgeMetrics, BadgeShape, BadgeSize, StatusSkin, StatusVariant,
    TABULAR_NUMS, TrendTone,
};
use crate::icons::Icon;
use crate::rune::{ToRune, sans};
use crate::theme::Theme;

/// A dashboard status: `StatusBadge::new("Queued").variant(StatusVariant::Info)
/// .icon(Icon::HugeClock01).build(&theme)`.
pub struct StatusBadge {
    label: String,
    variant: StatusVariant,
    size: BadgeSize,
    shape: BadgeShape,
    icon: Option<Icon>,
    square: bool,
    spin: bool,
}

impl StatusBadge {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: StatusVariant::Secondary,
            size: BadgeSize::Md,
            shape: BadgeShape::Rounded,
            icon: None,
            square: false,
            spin: false,
        }
    }

    pub fn variant(mut self, variant: StatusVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }

    pub fn shape(mut self, shape: BadgeShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Draws the icon knocked out of a filled square, for final results.
    pub fn square(mut self, square: bool) -> Self {
        self.square = square;
        self
    }

    /// Spins the icon, for work in progress.
    pub fn spin(mut self, spin: bool) -> Self {
        self.spin = spin;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let skin = StatusSkin::new(theme, self.variant, BadgeAppearance::Badge);
        let m = self.size.metrics();
        let ink = skin.text.rune();
        let glyph = self.icon.map(|i| {
            if self.square {
                el().size(status_badge::SQUARE_SIZE, status_badge::SQUARE_SIZE)
                    .radius(status_badge::SQUARE_RADIUS)
                    .fill(ink)
                    .center()
                    .child(
                        icon(i.rune(), status_badge::SQUARE_GLYPH).color(theme.background.rune()),
                    )
            } else {
                let g = icon(i.rune(), m.icon_size).color(ink);
                if self.spin { g.looping(spin()) } else { g }
            }
        });
        badge_box(m, self.shape, skin, BadgeAppearance::Badge)
            .children(glyph)
            .child(label(self.label.clone(), m, skin, false))
            .label(self.label)
    }
}

/// A metric change: `TrendBadge::new(TrendTone::Up, "+12.4%").build(&theme)`.
pub struct TrendBadge {
    tone: TrendTone,
    value: String,
    size: BadgeSize,
    shape: BadgeShape,
    appearance: BadgeAppearance,
    show_icon: bool,
}

impl TrendBadge {
    pub fn new(tone: TrendTone, value: impl Into<String>) -> Self {
        Self {
            tone,
            value: value.into(),
            size: BadgeSize::Default,
            shape: BadgeShape::Rounded,
            appearance: BadgeAppearance::Badge,
            show_icon: true,
        }
    }

    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }

    pub fn shape(mut self, shape: BadgeShape) -> Self {
        self.shape = shape;
        self
    }

    /// Bare coloured text for inline readouts.
    pub fn plain(mut self) -> Self {
        self.appearance = BadgeAppearance::Plain;
        self
    }

    pub fn show_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let skin = StatusSkin::new(theme, self.tone.variant(), self.appearance);
        let m = self.size.metrics();
        let glyph = self
            .show_icon
            .then(|| icon(self.tone.icon().rune(), m.icon_size).color(skin.text.rune()));
        badge_box(m, self.shape, skin, self.appearance)
            .children(glyph)
            .child(label(self.value.clone(), m, skin, true))
            .label(self.value)
    }
}

fn label(content: String, m: BadgeMetrics, skin: StatusSkin, tabular: bool) -> El {
    let font = sans(m.font_size, status_badge::WEIGHT, m.line_height);
    text(content)
        .font(if tabular {
            font.features(TABULAR_NUMS)
        } else {
            font
        })
        .color(skin.text.rune())
}

fn badge_box(
    m: BadgeMetrics,
    shape: BadgeShape,
    skin: StatusSkin,
    appearance: BadgeAppearance,
) -> El {
    match appearance {
        BadgeAppearance::Plain => row().gap(4.0).center_y(),
        BadgeAppearance::Badge => row()
            .h(m.height)
            .px(m.padding_x)
            .gap(m.gap)
            .center_y()
            .justify_center()
            .radius(shape.radius().min(m.height / 2.0))
            .fill(skin.fill.rune()),
    }
}
