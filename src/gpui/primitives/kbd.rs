use ::gpui::{
    AnyElement, App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px, svg,
};

use crate::components::primitives::kbd::{self, KbdSkin};
use crate::fonts;
use crate::gpui::ActiveTheme;
use crate::icons::Icon;

/// A keycap: `Kbd::new("⌘")`, or `Kbd::icon(Icon::ArrowUp)`.
#[derive(IntoElement)]
pub struct Kbd {
    icon: Option<Icon>,
    children: Vec<AnyElement>,
    in_tooltip: bool,
}

impl Kbd {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            icon: None,
            children: vec![label.into().into_any_element()],
            in_tooltip: false,
        }
    }

    /// A keycap holding a glyph at `size-3`.
    pub fn icon(icon: Icon) -> Self {
        Self {
            icon: Some(icon),
            children: Vec::new(),
            in_tooltip: false,
        }
    }

    /// The inverted keycap used inside tooltips.
    pub fn in_tooltip(mut self) -> Self {
        self.in_tooltip = true;
        self
    }
}

impl ParentElement for Kbd {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Kbd {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.notra();
        let skin = if self.in_tooltip {
            KbdSkin::in_tooltip(theme)
        } else {
            KbdSkin::new(theme)
        };
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(kbd::GAP))
            .h(px(kbd::HEIGHT))
            .min_w(px(kbd::MIN_WIDTH))
            .px(px(kbd::PADDING_X))
            .rounded(px(kbd::RADIUS))
            .bg(skin.fill)
            .text_color(skin.text)
            .font_family(fonts::SANS)
            .text_size(px(kbd::FONT_SIZE))
            .line_height(px(kbd::LINE_HEIGHT))
            .font_weight(FontWeight(kbd::WEIGHT))
            .when_some(self.icon, |d, icon| {
                d.child(
                    svg()
                        .path(icon.path())
                        .size(px(kbd::ICON_SIZE))
                        .flex_none()
                        .text_color(skin.text),
                )
            })
            .children(self.children)
    }
}

/// Keys pressed together: `KbdGroup::new().child(Kbd::new("⌘")).child(Kbd::new("K"))`.
#[derive(IntoElement, Default)]
pub struct KbdGroup {
    children: Vec<AnyElement>,
}

impl KbdGroup {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for KbdGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for KbdGroup {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(kbd::GAP))
            .children(self.children)
    }
}
