//! GPUI rendering of the dualtone card family.

use std::time::Instant;

use ::gpui::{
    AnyElement, App, BoxShadow, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce,
    SharedString, Window, div, point, prelude::*, px,
};

use super::data_parts::{Shimmer, Spinner, Tooltip, ease_on_hover, glyph, use_eased};
use super::{ActiveTheme, gradient, shadows};
use crate::components::card::{
    self, CardSkin, CardVariant, HINT_TRANSITION, empty, reveal, section,
};
use crate::components::data_parts::lerp_color;
use crate::icons::Icon;
use crate::theme::Shadow;

/// The info icon after an eyebrow, with the hint in a tooltip.
#[derive(IntoElement)]
struct Hint {
    id: ElementId,
    text: SharedString,
}

impl RenderOnce for Hint {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let skin = CardSkin::new(&theme, CardVariant::Panel);
        let (state, hover) = use_eased(self.id.clone(), HINT_TRANSITION, window, cx);
        div()
            .id(self.id)
            .flex()
            .flex_none()
            .cursor(::gpui::CursorStyle::Arrow)
            .aria_label(card::MORE_INFO)
            .child(glyph(
                Icon::HugeInformationCircle,
                card::HINT_ICON,
                lerp_color(skin.hint, skin.hint_hover, hover),
            ))
            .on_hover(ease_on_hover(state))
            .tooltip(Tooltip::text(self.text))
    }
}

/// A title with an optional hint icon, as the card and section headings draw it.
fn eyebrow(id: &ElementId, title: SharedString, hint: Option<SharedString>) -> ::gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(card::HINT_GAP))
        .min_w_0()
        .child(div().truncate().child(title))
        .when_some(hint, |d, text| {
            d.child(Hint {
                id: SharedString::from(format!("{id}-hint")).into(),
                text,
            })
        })
}

/// Notra's `InstrumentModule`: a titled module in the dualtone shell or the flat card.
///
/// ```ignore
/// Card::new("mentions", "Mention rate")
///     .variant(CardVariant::Table)
///     .hint("Share of answers that mention your brand, per engine.")
///     .readout("Last 7 days")
///     .action(Button::new("view-all", "View all").size(ButtonSize::Sm))
///     .child(list)
/// ```
#[derive(IntoElement)]
pub struct Card {
    id: ElementId,
    eyebrow: SharedString,
    variant: CardVariant,
    bare_body: bool,
    description: Option<SharedString>,
    hint: Option<SharedString>,
    readout: Option<SharedString>,
    action: Option<AnyElement>,
    children: Vec<AnyElement>,
}

impl Card {
    pub fn new(id: impl Into<ElementId>, eyebrow: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            eyebrow: eyebrow.into(),
            variant: CardVariant::Flat,
            bare_body: false,
            description: None,
            hint: None,
            readout: None,
            action: None,
            children: Vec::new(),
        }
    }

    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Drop the shell and the lifted body: the header sits over bare content.
    pub fn bare_body(mut self, bare: bool) -> Self {
        self.bare_body = bare;
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Explains the module on an info icon after the eyebrow.
    pub fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// A small muted value before the action (a time range, a count).
    pub fn readout(mut self, readout: impl Into<SharedString>) -> Self {
        self.readout = Some(readout.into());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let variant = self.variant;
        let m = variant.metrics(self.bare_body);
        let skin = CardSkin::new(&theme, variant);
        let table = variant.has_table_header();
        let flat = !variant.is_dualtone();
        let readout = self.readout.map(|text| {
            div()
                .truncate()
                .text_size(px(card::READOUT_SIZE))
                .line_height(px(card::READOUT_LINE_HEIGHT))
                .text_color(skin.readout)
                .when(flat, |d| d.child(card::capitalize_first(&text)))
                .when(!flat, |d| d.child(text))
        });
        let has_action = readout.is_some() || self.action.is_some();
        let action = has_action.then(|| {
            div()
                .flex()
                .flex_none()
                .items_center()
                .gap(px(card::ACTION_GAP))
                .min_w_0()
                .children(readout)
                .children(self.action)
        });
        let title = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .gap(px(m.header_gap))
            .child(
                eyebrow(&self.id, self.eyebrow, self.hint)
                    .text_size(px(m.title_size))
                    .line_height(px(m.title_line_height))
                    .font_weight(FontWeight(500.0))
                    .text_color(skin.title),
            )
            .when_some(self.description, |d, text| {
                d.child(div().text_color(skin.description).child(text))
            });
        let header = div()
            .flex()
            .flex_col()
            .when(m.header_centered, |d| d.justify_center())
            .min_h(px(m.header_min_height))
            .pt(px(m.header_padding_top))
            .pb(px(m.header_padding_bottom))
            .px(px(m.header_padding_x))
            .child(
                div()
                    .flex()
                    .gap(px(4.0))
                    .map(|d| {
                        if table || flat {
                            d.items_center()
                        } else {
                            d.items_start()
                        }
                    })
                    .child(title)
                    .children(action),
            );
        let bare = self.bare_body && !flat;
        let body = div()
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .when(m.body_overlap > 0.0, |d| d.mt(px(-m.body_overlap)))
            .p(px(m.body_padding))
            .px(px(m.body_padding_x))
            .when(variant.is_dualtone() && !bare, |d| {
                d.rounded(px(m.body_radius))
                    .border_1()
                    .border_color(skin.body_border)
                    .bg(skin.body)
                    .shadow(shadows(&skin.body_shadows))
            })
            .children(self.children);
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .gap(px(m.gap))
            .py(px(m.padding_y))
            .rounded(px(m.radius))
            .text_size(px(14.0))
            .line_height(px(20.0))
            .text_color(theme.foreground)
            .map(|d| {
                if flat {
                    d.bg(skin.shell).shadow(vec![BoxShadow {
                        color: skin.shell_border.into(),
                        offset: point(px(0.0), px(0.0)),
                        blur_radius: px(0.0),
                        spread_radius: px(1.0),
                        inset: false,
                    }])
                } else if bare {
                    d
                } else {
                    d.p(px(m.shell_padding))
                        .border_1()
                        .border_color(skin.shell_border)
                        .bg(skin.shell)
                }
            })
            .child(header)
            .child(body)
    }
}

/// Notra's `InstrumentSection`: a heading with readout and actions over bare content.
/// Title and actions share a row while there is room and wrap otherwise.
#[derive(IntoElement)]
pub struct CardSection {
    id: ElementId,
    title: SharedString,
    description: Option<SharedString>,
    hint: Option<SharedString>,
    readout: Option<SharedString>,
    action: Option<AnyElement>,
    children: Vec<AnyElement>,
}

impl CardSection {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            hint: None,
            readout: None,
            action: None,
            children: Vec::new(),
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn readout(mut self, readout: impl Into<SharedString>) -> Self {
        self.readout = Some(readout.into());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl ParentElement for CardSection {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for CardSection {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let has_action = self.readout.is_some() || self.action.is_some();
        let described = self.description.is_some();
        let heading = div()
            .flex()
            .flex_col()
            .gap(px(section::TITLE_GAP))
            .min_w_0()
            .when(!described && has_action, |d| {
                d.h(px(section::ROW_HEIGHT)).justify_center()
            })
            .child(
                eyebrow(&self.id, self.title, self.hint)
                    .text_size(px(14.0))
                    .line_height(px(if !described && has_action { 14.0 } else { 20.0 }))
                    .font_weight(FontWeight(500.0))
                    .text_color(theme.foreground),
            )
            .when_some(self.description, |d, text| {
                d.child(div().text_color(theme.muted_foreground).child(text))
            });
        let actions = has_action.then(|| {
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(card::ACTION_GAP))
                .min_w_0()
                .when_some(self.readout, |d, text| {
                    d.child(
                        div()
                            .truncate()
                            .text_size(px(card::READOUT_SIZE))
                            .line_height(px(card::READOUT_LINE_HEIGHT))
                            .text_color(theme.muted_foreground)
                            .child(card::capitalize_first(&text)),
                    )
                })
                .children(self.action)
        });
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .gap(px(section::GAP))
            .min_w_0()
            .text_size(px(14.0))
            .line_height(px(20.0))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_between()
                    .map(|d| {
                        if described {
                            d.items_start()
                        } else {
                            d.items_center()
                        }
                    })
                    .gap(px(section::HEADER_GAP))
                    .min_w_0()
                    .child(heading)
                    .children(actions),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .children(self.children),
            )
    }
}

/// Notra's `InstrumentEmpty`: a centred message, a spinner and shimmer while busy, an
/// optional action, and an optional faded preview behind it.
#[derive(IntoElement)]
pub struct CardEmpty {
    id: ElementId,
    message: Option<SharedString>,
    busy: bool,
    action: Option<AnyElement>,
    preview: Option<AnyElement>,
}

impl CardEmpty {
    pub fn new(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            message: Some(message.into()).filter(|m: &SharedString| !m.is_empty()),
            busy: false,
            action: None,
            preview: None,
        }
    }

    /// Show a spinner and sweep the message while data loads; hides the action.
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }

    /// Content shown faded behind the message (a sample of what will appear).
    pub fn preview(mut self, preview: impl IntoElement) -> Self {
        self.preview = Some(preview.into_any_element());
        self
    }
}

impl RenderOnce for CardEmpty {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let has_preview = self.preview.is_some();
        let message = (self.message.is_some() || self.busy).then(|| {
            let text = self.message.clone().map(|m| card::capitalize_first(&m));
            div()
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .gap(px(empty::MESSAGE_GAP))
                .when(has_preview, |d| {
                    d.px(px(empty::PILL_PADDING_X))
                        .py(px(empty::PILL_PADDING_Y))
                        .rounded_full()
                        .bg(theme.background.fade(empty::PILL_ALPHA))
                        .shadow(shadows(&[Shadow::drop(
                            1.0,
                            2.0,
                            0.0,
                            crate::Color::rgba(0x0000000d),
                        )]))
                })
                .when(self.busy, |d| d.child(Spinner::new()))
                .when_some(text, |d, text| {
                    if self.busy {
                        d.child(Shimmer::new(
                            SharedString::from(format!("{}-shimmer", self.id)),
                            text,
                        ))
                    } else {
                        d.child(div().text_color(theme.muted_foreground).child(text))
                    }
                })
        });
        let fade = |from: crate::Color, to: crate::Color, top: f32, height: f32| {
            div()
                .absolute()
                .left_0()
                .right_0()
                .top(::gpui::relative(top))
                .h(::gpui::relative(height))
                .bg(gradient(from, to))
        };
        let surface = theme.table_card;
        div()
            .id(self.id.clone())
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .items_center()
            .justify_center()
            .gap(px(empty::GAP))
            .min_h(px(empty::MIN_HEIGHT))
            .when(has_preview, |d| d.overflow_hidden())
            .when_some(self.preview, |d, preview| {
                d.child(
                    div()
                        .absolute()
                        .inset_0()
                        .opacity(empty::PREVIEW_OPACITY)
                        .child(preview)
                        .child(fade(surface, surface.fade(0.0), 0.0, 0.24))
                        .child(fade(surface.fade(0.0), surface, 0.70, 0.30)),
                )
            })
            .children(message)
            .when(!self.busy, |d| {
                d.children(self.action.map(|a| div().relative().child(a)))
            })
    }
}

/// Notra's `InstrumentReveal`: content that rises 6 px and fades in when
/// `active` turns on, `order` × 45 ms after its siblings.
#[derive(IntoElement)]
pub struct CardReveal {
    id: ElementId,
    active: bool,
    order: u32,
    children: Vec<AnyElement>,
}

impl CardReveal {
    pub fn new(id: impl Into<ElementId>, active: bool) -> Self {
        Self {
            id: id.into(),
            active,
            order: 0,
            children: Vec::new(),
        }
    }

    pub fn order(mut self, order: u32) -> Self {
        self.order = order;
        self
    }
}

impl ParentElement for CardReveal {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

struct RevealState {
    active: bool,
    since: Option<Instant>,
}

impl RenderOnce for CardReveal {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| RevealState {
            active: self.active,
            since: None,
        });
        let now = Instant::now();
        state.update(cx, |s, _| {
            if s.active != self.active {
                s.active = self.active;
                s.since = Some(now);
            }
        });
        let since = state.read(cx).since;
        let elapsed = since
            .filter(|_| !cx.reduce_motion())
            .map(|at| now.duration_since(at));
        if elapsed.is_some_and(|e| e < reveal::tween(self.order).total()) {
            window.request_animation_frame();
        }
        let (opacity, offset) = reveal::frame(self.order, self.active, elapsed);
        div()
            .id(self.id)
            .relative()
            .top(px(offset))
            .opacity(opacity)
            .flex()
            .flex_col()
            .min_w_0()
            .children(self.children)
    }
}

/// Notra's `InstrumentGrid`: modules in equal columns, 12 px apart.
#[derive(IntoElement)]
pub struct CardGrid {
    columns: u16,
    children: Vec<AnyElement>,
}

impl CardGrid {
    pub fn new(columns: u16) -> Self {
        Self {
            columns: columns.max(1),
            children: Vec::new(),
        }
    }
}

impl ParentElement for CardGrid {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for CardGrid {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .grid()
            .grid_cols(self.columns)
            .gap(px(card::GRID_GAP))
            .w_full()
            .children(self.children)
    }
}
