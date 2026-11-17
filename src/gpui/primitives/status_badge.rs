use ::gpui::{
    App, Div, ElementId, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*, px, svg,
};

use super::spinner::spinning_icon;
use super::tabular_nums;
use crate::components::primitives::status_badge::{
    self, BadgeAppearance, BadgeShape, BadgeSize, StatusSkin, StatusVariant, TrendTone,
};
use crate::fonts;
use crate::gpui::ActiveTheme;
use crate::icons::Icon;

/// A dashboard status: `StatusBadge::new("Successful").variant(StatusVariant::Success)
/// .icon(Icon::HugeTick02).square(true)`.
#[derive(IntoElement)]
pub struct StatusBadge {
    label: SharedString,
    variant: StatusVariant,
    size: BadgeSize,
    shape: BadgeShape,
    icon: Option<Icon>,
    square: bool,
    spin: bool,
}

impl StatusBadge {
    pub fn new(label: impl Into<SharedString>) -> Self {
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

    /// Spins the icon (`motion-safe:animate-spin`), for work in progress.
    pub fn spin(mut self, spin: bool) -> Self {
        self.spin = spin;
        self
    }
}

impl RenderOnce for StatusBadge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let skin = StatusSkin::new(&theme, self.variant, BadgeAppearance::Badge);
        let m = self.size.metrics();
        let text: Hsla = skin.text.into();
        let glyph = self.icon.map(|icon| {
            if self.square {
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(px(status_badge::SQUARE_SIZE))
                    .rounded(px(status_badge::SQUARE_RADIUS))
                    .bg(text)
                    .child(
                        svg()
                            .path(icon.path())
                            .size(px(status_badge::SQUARE_GLYPH))
                            .text_color(theme.background),
                    )
                    .into_any_element()
            } else if self.spin && !cx.reduce_motion() {
                spinning_icon(
                    ElementId::Name(format!("status-spin-{}", self.label).into()),
                    icon,
                    m.icon_size,
                    text,
                    false,
                )
                .into_any_element()
            } else {
                svg()
                    .path(icon.path())
                    .size(px(m.icon_size))
                    .flex_none()
                    .text_color(text)
                    .into_any_element()
            }
        });
        badge_box(m, self.shape, skin, BadgeAppearance::Badge)
            .children(glyph)
            .child(self.label)
    }
}

/// A metric change: `TrendBadge::new(TrendTone::Up, "+12.4%")`.
#[derive(IntoElement)]
pub struct TrendBadge {
    tone: TrendTone,
    value: SharedString,
    size: BadgeSize,
    shape: BadgeShape,
    appearance: BadgeAppearance,
    show_icon: bool,
}

impl TrendBadge {
    pub fn new(tone: TrendTone, value: impl Into<SharedString>) -> Self {
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
}

impl RenderOnce for TrendBadge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let skin = StatusSkin::new(cx.notra(), self.tone.variant(), self.appearance);
        let m = self.size.metrics();
        badge_box(m, self.shape, skin, self.appearance)
            .font_features(tabular_nums())
            .when(self.show_icon, |d| {
                d.child(
                    svg()
                        .path(self.tone.icon().path())
                        .size(px(m.icon_size))
                        .flex_none()
                        .text_color(skin.text),
                )
            })
            .child(self.value)
    }
}

fn badge_box(
    m: status_badge::BadgeMetrics,
    shape: BadgeShape,
    skin: StatusSkin,
    appearance: BadgeAppearance,
) -> Div {
    let text = div()
        .flex()
        .flex_none()
        .items_center()
        .text_color(skin.text)
        .font_family(fonts::SANS)
        .font_weight(FontWeight(status_badge::WEIGHT))
        .whitespace_nowrap();
    match appearance {
        BadgeAppearance::Plain => text
            .gap(px(4.0))
            .text_size(px(m.font_size))
            .line_height(px(m.line_height)),
        BadgeAppearance::Badge => text
            .justify_center()
            .overflow_hidden()
            .gap(px(m.gap))
            .h(px(m.height))
            .px(px(m.padding_x))
            .py(px(m.padding_y))
            .rounded(px(shape.radius()))
            .border_1()
            .border_color(::gpui::transparent_black())
            .bg(skin.fill)
            .text_size(px(m.font_size))
            .line_height(px(m.line_height)),
    }
}
