//! The composer for GPUI: [`ComposerFrame`] and its parts, matching `composer.tsx`.

use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use ::gpui::{
    Anchor, AnyElement, App, ClickEvent, Context, CursorStyle, ElementId, Entity, FontWeight, Hsla,
    IntoElement, ParentElement, PathBuilder, RenderOnce, Role, SharedString, Styled,
    Transformation, Window, anchored, canvas, deferred, div, percentage, point, prelude::*, px,
    svg,
};

use super::input::{GhostButton, GhostMetrics, TextField, hover_value, lerp_color, track_hover};
use super::{ActiveTheme, gradient, shadows};
use crate::components::composer::{
    CARD_INPUT, CARD_RADIUS, ComposerShape, ComposerSize, ComposerSkin, FLAT_INPUT, HOVER, NUDGE,
    PillTray, SPIN, SendState, Tone, badge, chip, nudge, pill, send, tone_color, toolbar,
    toolbar_button, tooltip, tray,
};
use crate::components::input::DISABLED_OPACITY;
use crate::fonts;
use crate::icons::Icon;
use crate::motion::Animated;
use crate::theme::Shadow;

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type NudgeBuilder = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// The composer: a dualtone card (or a pill with `flat`) holding a [`ComposerInput`] and a
/// [`ComposerToolbar`], with an optional tray of queued messages, attachments or a notice
/// tucked under its top edge. The tray grows in and out over 300 ms.
///
/// ```ignore
/// ComposerFrame::new("chat")
///     .nudge(move |_, _| ComposerNudge::new().child(chip).into_any_element())
///     .child(ComposerInput::new(field.clone()))
///     .child(ComposerToolbar::new().child(send))
/// ```
#[derive(IntoElement)]
pub struct ComposerFrame {
    id: ElementId,
    shape: ComposerShape,
    size: ComposerSize,
    connected_top: bool,
    nudge: Option<NudgeBuilder>,
    children: Vec<AnyElement>,
}

impl ComposerFrame {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            shape: ComposerShape::Card,
            size: ComposerSize::Default,
            connected_top: false,
            nudge: None,
            children: Vec::new(),
        }
    }

    /// A round pill with a tray only while there is a nudge (React's `flat`).
    pub fn flat(mut self) -> Self {
        self.shape = ComposerShape::Flat;
        self
    }

    /// The dense pill; pair it with a `ComposerSize::Sm` send.
    pub fn size(mut self, size: ComposerSize) -> Self {
        self.size = size;
        self
    }

    /// Drops the top radius so the frame sits flush under another surface.
    pub fn connected_top(mut self, connected: bool) -> Self {
        self.connected_top = connected;
        self
    }

    /// The tray's contents, usually a [`ComposerNudge`]. It is built every frame, and once
    /// more for each frame of its exit after the frame stops passing it.
    pub fn nudge(mut self, build: impl Fn(&mut Window, &mut App) -> AnyElement + 'static) -> Self {
        self.nudge = Some(Rc::new(build));
        self
    }
}

impl ParentElement for ComposerFrame {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

struct FrameState {
    nudge: Option<NudgeBuilder>,
    open: Animated,
    height: f32,
}

impl RenderOnce for ComposerFrame {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let skin = ComposerSkin::new(&theme);
        let now = Instant::now();
        let reduce = cx.reduce_motion();
        let has_nudge = self.nudge.is_some();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| FrameState {
            nudge: None,
            open: Animated::new(if has_nudge { 1.0 } else { 0.0 }, NUDGE),
            height: 0.0,
        });
        state.update(cx, |s, _| {
            if let Some(build) = self.nudge.clone() {
                s.nudge = Some(build);
            }
            s.open.set(if has_nudge { 1.0 } else { 0.0 }, now);
            if reduce {
                s.open.snap(s.open.target());
            }
            if !has_nudge && !s.open.is_animating(now) {
                s.nudge = None;
            }
        });
        let (open, animating, height, build) = {
            let s = state.read(cx);
            (
                s.open.value(now),
                s.open.is_animating(now),
                s.height,
                s.nudge.clone(),
            )
        };
        if animating {
            window.request_animation_frame();
        }

        let flat = self.shape == ComposerShape::Flat;
        let connected = self.connected_top;
        let content = build.map(|build| {
            let inner = build(window, cx);
            if flat {
                inner
            } else {
                div()
                    .flex_none()
                    .border_l_1()
                    .border_r_1()
                    .when(!connected, |d| {
                        d.border_t_1()
                            .mx(px(tray::INSET_X))
                            .rounded_t(px(tray::RADIUS))
                    })
                    .border_color(skin.tray_border)
                    .bg(skin.tray)
                    .pt(px(tray::PADDING_TOP))
                    .pb(px(tray::PADDING_BOTTOM))
                    .child(inner)
                    .into_any_element()
            }
        });
        let measure = state.clone();
        let shell = content.map(|content| {
            div()
                .flex()
                .flex_col()
                .flex_none()
                .overflow_hidden()
                .opacity(open)
                .when(animating, |d| d.h(px(height * open)))
                .child(
                    div()
                        .relative()
                        .flex()
                        .flex_col()
                        .flex_none()
                        .child(content)
                        .child(
                            canvas(
                                move |bounds, window, cx| {
                                    let h = f32::from(bounds.size.height);
                                    let changed = measure.update(cx, |s, _| {
                                        let changed = (s.height - h).abs() > 0.5;
                                        s.height = h;
                                        changed
                                    });
                                    if changed {
                                        window.request_animation_frame();
                                    }
                                },
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full(),
                        ),
                )
        });

        let base = div()
            .id(self.id)
            .flex()
            .flex_col()
            .w_full()
            .min_w_0()
            .text_color(skin.text)
            .font_family(fonts::SANS);
        if flat {
            let t = PillTray::at(self.size, open);
            let radius = pill::card_radius(self.size);
            base.bg(skin.pill_tray.fade(t.fill))
                .p(px(t.padding))
                .rounded_b(px(t.bottom_radius))
                .when(!connected, |d| d.rounded_t(px(t.top_radius)))
                .children(shell)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .min_w_0()
                        .overflow_hidden()
                        .border_1()
                        .border_color(skin.pill_border)
                        .bg(skin.pill)
                        .rounded(px(radius))
                        .when(connected && !has_nudge, |d| {
                            d.rounded_t(px(0.)).border_t(px(0.))
                        })
                        .children(self.children),
                )
        } else {
            base.children(shell).child(
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .min_w_0()
                    .overflow_hidden()
                    .border_1()
                    .border_color(skin.card_border)
                    .bg(skin.card)
                    .shadow(shadows(skin.card_shadow))
                    .rounded(px(CARD_RADIUS))
                    .when(has_nudge, |d| d.mt(px(-tray::OVERLAP)))
                    .when(connected && !has_nudge, |d| {
                        d.rounded_t(px(0.)).border_t(px(0.))
                    })
                    .children(self.children),
            )
        }
    }
}

/// The tray's row: a title, chips, or free content, with an action at the end.
#[derive(IntoElement, Default)]
pub struct ComposerNudge {
    title: Option<AnyElement>,
    action: Option<AnyElement>,
    children: Vec<AnyElement>,
}

impl ComposerNudge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Shown when there are no children, in `text-xs font-medium`.
    pub fn title(mut self, title: impl IntoElement) -> Self {
        self.title = Some(title.into_any_element());
        self
    }

    /// Sits at the end of the row (an Upgrade button, a close button).
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl ParentElement for ComposerNudge {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ComposerNudge {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let has_children = !self.children.is_empty();
        div()
            .flex()
            .min_w_0()
            .items_center()
            .gap(px(nudge::GAP))
            .px(px(nudge::PADDING_X))
            .pt(px(nudge::PADDING_TOP))
            .pb(px(nudge::PADDING_BOTTOM))
            .when(has_children, |d| d.flex_wrap())
            .when_some(self.title.filter(|_| !has_children), |d, title| {
                d.child(
                    div()
                        .min_w_0()
                        .flex_1()
                        .text_size(px(nudge::TITLE_SIZE))
                        .line_height(px(nudge::TITLE_LINE))
                        .font_weight(FontWeight(nudge::TITLE_WEIGHT))
                        .child(title),
                )
            })
            .when(has_children, |d| {
                d.child(
                    div()
                        .flex()
                        .min_w_0()
                        .flex_1()
                        .flex_wrap()
                        .items_center()
                        .gap(px(nudge::CHIP_GAP))
                        .children(self.children),
                )
            })
            .when_some(self.action, |d, action| {
                d.child(
                    div()
                        .ml_auto()
                        .flex()
                        .flex_none()
                        .items_center()
                        .child(action),
                )
            })
    }
}

/// A notice for the tray: a tinted icon tile and its text.
#[derive(IntoElement)]
pub struct ComposerNotice {
    tone: Tone,
    icon: Icon,
    text: SharedString,
    body: bool,
}

impl ComposerNotice {
    pub fn new(tone: Tone, icon: Icon, text: impl Into<SharedString>) -> Self {
        Self {
            tone,
            icon,
            text: text.into(),
            body: false,
        }
    }

    /// Body text (`text-sm`, truncated) for a notice placed as the nudge's content rather than
    /// its title.
    pub fn body(mut self) -> Self {
        self.body = true;
        self
    }
}

impl RenderOnce for ComposerNotice {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let tone = tone_color(cx.notra(), self.tone);
        div()
            .flex()
            .min_w_0()
            .items_center()
            .gap(px(badge::GAP))
            .when(self.body, |d| {
                d.text_size(px(nudge::BODY_SIZE))
                    .line_height(px(nudge::BODY_LINE))
                    .font_weight(FontWeight::NORMAL)
            })
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(px(badge::SIZE))
                    .rounded(px(badge::RADIUS))
                    .bg(tone.fade(badge::TINT))
                    .child(
                        svg()
                            .path(self.icon.path())
                            .size(px(badge::ICON))
                            .text_color(tone),
                    ),
            )
            .child(div().min_w_0().truncate().child(self.text))
    }
}

/// What leads a chip's label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipLead {
    /// A muted `size-3.5` icon (a file).
    Icon(Icon),
    /// A `size-3` spinner, for an upload in progress.
    Spinner,
}

/// A queued message or an attachment in the tray: a dashed chip with steer, edit and remove
/// actions.
#[derive(IntoElement)]
pub struct ComposerChip {
    id: ElementId,
    label: SharedString,
    lead: Option<ChipLead>,
    pending: bool,
    on_click: Option<ClickHandler>,
    on_steer: Option<ClickHandler>,
    on_edit: Option<ClickHandler>,
    on_remove: Option<ClickHandler>,
}

impl ComposerChip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            lead: None,
            pending: false,
            on_click: None,
            on_steer: None,
            on_edit: None,
            on_remove: None,
        }
    }

    pub fn lead(mut self, lead: ChipLead) -> Self {
        self.lead = Some(lead);
        self
    }

    /// Still uploading: fainter, without steer or edit.
    pub fn pending(mut self, pending: bool) -> Self {
        self.pending = pending;
        self
    }

    /// Makes the label a button (a preview).
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Sends the queued message now.
    pub fn on_steer(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_steer = Some(Box::new(f));
        self
    }

    /// Moves the message back into the input.
    pub fn on_edit(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_edit = Some(Box::new(f));
        self
    }

    pub fn on_remove(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Box::new(f));
        self
    }
}

impl RenderOnce for ComposerChip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let skin = ComposerSkin::new(&theme);
        let text = if self.pending { skin.muted } else { skin.text };
        let action_metrics = GhostMetrics {
            height: chip::ACTION,
            padding_x: None,
            gap: 0.0,
            radius: chip::ACTION_RADIUS,
            font_size: 14.0,
            line_height: 20.0,
            icon: chip::ACTION_ICON,
        };
        let action =
            |suffix: &'static str, icon: Icon, label: String, handler: Option<ClickHandler>| {
                handler.map(|handler| {
                    GhostButton::new(
                        child_id(&self.id, suffix),
                        action_metrics,
                        skin.muted,
                        (skin.chip_action_hover, skin.hover_text),
                    )
                    .icon(Some(icon))
                    .aria_label(Some(label.into()))
                    .on_click(Some(handler))
                })
            };
        let lead = self.lead.map(|lead| match lead {
            ChipLead::Icon(icon) => svg()
                .path(icon.path())
                .size(px(chip::ICON))
                .flex_none()
                .text_color(skin.muted)
                .into_any_element(),
            ChipLead::Spinner => spinner(
                child_id(&self.id, "spinner"),
                chip::SPINNER,
                text.into(),
                window,
                cx,
            )
            .into_any_element(),
        });
        let label = div()
            .max_w(px(chip::LABEL_MAX_WIDTH))
            .truncate()
            .child(self.label.clone());
        let content = div()
            .flex()
            .min_w_0()
            .items_center()
            .gap(px(chip::GAP))
            .children(lead)
            .child(label);
        let content = match self.on_click {
            Some(handler) => div()
                .id(child_id(&self.id, "preview"))
                .role(Role::Button)
                .aria_label(format!("Preview {}", self.label))
                .tab_stop(true)
                .rounded(px(chip::ACTION_RADIUS))
                .cursor_pointer()
                .focus_visible(move |s| {
                    s.shadow(shadows(&[Shadow::drop(
                        0.0,
                        0.0,
                        3.0,
                        theme.ring.fade(0.5),
                    )]))
                })
                .on_click(handler)
                .child(content)
                .into_any_element(),
            None => content.into_any_element(),
        };
        let pending = self.pending;
        let label = self.label.clone();
        div()
            .id(self.id.clone())
            .flex()
            .max_w_full()
            .items_center()
            .gap(px(chip::GAP))
            .rounded(px(chip::RADIUS))
            .border_1()
            .border_dashed()
            .border_color(if pending {
                skin.chip_pending_border
            } else {
                skin.chip_border
            })
            .bg(skin.chip)
            .pl(px(chip::PADDING_LEFT))
            .pr(px(chip::PADDING_RIGHT))
            .py(px(chip::PADDING_Y))
            .text_size(px(chip::FONT_SIZE))
            .line_height(px(chip::LINE_HEIGHT))
            .font_weight(FontWeight::NORMAL)
            .text_color(text)
            .child(content)
            .children(action(
                "steer",
                Icon::ArrowUp,
                format!("Send {label} now"),
                self.on_steer.filter(|_| !pending),
            ))
            .children(action(
                "edit",
                Icon::Pencil,
                format!("Edit {label}"),
                self.on_edit.filter(|_| !pending),
            ))
            .children(action(
                "remove",
                Icon::X,
                format!("Remove {label}"),
                self.on_remove,
            ))
    }
}

/// The composer's textarea: no box of its own, `text-sm leading-6` in the card and
/// `leading-5` in the pill, growing to 200 px (128 px) before it scrolls.
#[derive(IntoElement)]
pub struct ComposerInput {
    field: Entity<TextField>,
}

impl ComposerInput {
    pub fn new(field: Entity<TextField>) -> Self {
        Self { field }
    }

    /// A field configured for a composer of `shape`.
    pub fn field(cx: &mut Context<TextField>, shape: ComposerShape) -> TextField {
        TextField::new(cx).multiline().metrics(match shape {
            ComposerShape::Card => CARD_INPUT,
            ComposerShape::Flat => FLAT_INPUT,
        })
    }
}

impl RenderOnce for ComposerInput {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.field.read(cx).is_disabled();
        div()
            .flex_1()
            .w_full()
            .min_w_0()
            .when(disabled, |d| d.opacity(DISABLED_OPACITY))
            .child(self.field)
    }
}

/// The row under the input: `flex items-center gap-1 px-2 pt-1 pb-2`.
#[derive(IntoElement, Default)]
pub struct ComposerToolbar {
    children: Vec<AnyElement>,
}

impl ComposerToolbar {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for ComposerToolbar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ComposerToolbar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(toolbar::GAP))
            .px(px(toolbar::PADDING_X))
            .pt(px(toolbar::PADDING_TOP))
            .pb(px(toolbar::PADDING_BOTTOM))
            .children(self.children)
    }
}

/// A muted ghost button in the toolbar: a model or effort picker, or an icon action.
#[derive(IntoElement)]
pub struct ComposerToolbarButton {
    id: ElementId,
    label: Option<SharedString>,
    icon: Option<Icon>,
    aria_label: Option<SharedString>,
    chevron: bool,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl ComposerToolbarButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: Some(label.into()),
            icon: None,
            aria_label: None,
            chevron: false,
            disabled: false,
            on_click: None,
        }
    }

    /// A square `icon-sm` button; name it for assistive tech.
    pub fn icon_button(
        id: impl Into<ElementId>,
        icon: Icon,
        aria_label: impl Into<SharedString>,
    ) -> Self {
        Self {
            icon: Some(icon),
            label: None,
            aria_label: Some(aria_label.into()),
            ..Self::new(id, "")
        }
    }

    /// An icon before the label.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// A `size-3` chevron after the label, for pickers.
    pub fn chevron(mut self) -> Self {
        self.chevron = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl RenderOnce for ComposerToolbarButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let skin = ComposerSkin::new(cx.notra());
        let icon_only = self.label.is_none();
        let metrics = GhostMetrics {
            height: toolbar_button::HEIGHT,
            padding_x: (!icon_only).then_some(toolbar_button::PADDING_X),
            gap: toolbar_button::GAP,
            radius: toolbar_button::RADIUS,
            font_size: toolbar_button::FONT_SIZE,
            line_height: toolbar_button::LINE_HEIGHT,
            icon: if icon_only {
                toolbar_button::ICON_ONLY_ICON
            } else {
                toolbar_button::ICON
            },
        };
        GhostButton::new(
            self.id,
            metrics,
            skin.muted,
            (skin.hover_fill, skin.hover_text),
        )
        .icon(self.icon)
        .label(self.label)
        .icon_end(
            self.chevron
                .then_some((Icon::ChevronDown, toolbar_button::CHEVRON)),
        )
        .aria_label(self.aria_label)
        .disabled(self.disabled)
        .on_click(self.on_click)
    }
}

/// The send button: violet when there is something to send, the dark Depth button when
/// there is not, a spinner while busy. Stop with `Icon::SquareSolid`.
#[derive(IntoElement)]
pub struct ComposerSend {
    id: ElementId,
    label: SharedString,
    icon: Icon,
    busy: bool,
    disabled: bool,
    size: ComposerSize,
    round: bool,
    tooltip: Option<SharedString>,
    on_click: Option<ClickHandler>,
}

impl ComposerSend {
    /// `label` names the action for assistive tech (`aria-label`).
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: Icon::ArrowUp,
            busy: false,
            disabled: false,
            size: ComposerSize::Default,
            round: false,
            tooltip: None,
            on_click: None,
        }
    }

    /// `Icon::ArrowUp` (the default) or `Icon::SquareSolid` to stop.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }

    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: ComposerSize) -> Self {
        self.size = size;
        self
    }

    /// A circle, as inside a `flat` frame.
    pub fn round(mut self, round: bool) -> Self {
        self.round = round;
        self
    }

    /// A tooltip shown above after a hover of 600 ms.
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

struct SendTooltip {
    hovered_since: Option<Instant>,
    shown_at: Option<Instant>,
    hidden_at: Option<Instant>,
}

impl RenderOnce for ComposerSend {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let state = SendState::new(self.busy, self.disabled);
        let rest = state.skin(&theme);
        let hot = rest.hovered();
        let side = send::size(self.size);
        let shape = if self.round {
            ComposerShape::Flat
        } else {
            ComposerShape::Card
        };
        let radius = SendState::radius(shape, self.size);
        let (hover, hover_state) = hover_value(self.id.clone(), HOVER, window, cx);
        let pressable = state.is_pressable();
        let hover = if pressable { hover } else { 0.0 };
        let (top, bottom) = match (rest.gradient, hot.gradient) {
            (Some((t0, b0)), Some((t1, b1))) => {
                (lerp_color(t0, t1, hover), lerp_color(b0, b1, hover))
            }
            _ => (rest.fill, rest.fill),
        };
        let border = lerp_color(rest.border, hot.border, hover);
        let text: Hsla = lerp_color(rest.text, hot.text, hover).into();
        let content = if self.busy {
            spinner(
                child_id(&self.id, "spinner"),
                send::spinner(self.size),
                text,
                window,
                cx,
            )
            .into_any_element()
        } else {
            let icon_size = if self.icon == Icon::SquareSolid {
                send::STOP_ICON
            } else {
                send::icon(self.size)
            };
            svg()
                .path(self.icon.path())
                .size(px(icon_size))
                .text_color(text)
                .into_any_element()
        };

        let tip_state = self.tooltip.as_ref().map(|_| {
            window.use_keyed_state(child_id(&self.id, "tooltip"), cx, |_, _| SendTooltip {
                hovered_since: None,
                shown_at: None,
                hidden_at: None,
            })
        });
        let tip_view = match (self.tooltip.clone(), tip_state.clone()) {
            (Some(label), Some(tip_state)) => {
                let now = Instant::now();
                let progress = tip_state.update(cx, |s, _| {
                    if let Some(since) = s.hovered_since
                        && s.shown_at.is_none()
                        && now - since >= tooltip::DELAY
                    {
                        s.shown_at = Some(now);
                        s.hidden_at = None;
                    }
                    match (s.shown_at, s.hidden_at) {
                        (Some(at), _) => tooltip::ENTER.progress(now - at),
                        (None, Some(at)) => 1.0 - tooltip::EXIT.progress(now - at),
                        (None, None) => 0.0,
                    }
                });
                let s = tip_state.read(cx);
                let waiting = s.hovered_since.is_some() && s.shown_at.is_none();
                let moving = s
                    .shown_at
                    .is_some_and(|at| now - at < tooltip::ENTER.total())
                    || s.hidden_at
                        .is_some_and(|at| now - at < tooltip::EXIT.total());
                if waiting || moving {
                    window.request_animation_frame();
                }
                let progress = match (cx.reduce_motion(), s.shown_at.is_some()) {
                    (true, true) => 1.0,
                    (true, false) => 0.0,
                    (false, _) => progress,
                };
                (progress > 0.0).then(|| tooltip_bubble(label, progress, side, &theme))
            }
            _ => None,
        };

        let focus_ring = theme.ring;
        div()
            .id(self.id)
            .role(Role::Button)
            .aria_label(self.label)
            .tab_stop(true)
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(side))
            .rounded(px(radius))
            .border_1()
            .border_color(border)
            .bg(gradient(top, bottom))
            .shadow(shadows(&rest.shadows))
            .opacity(state.opacity())
            .focus_visible(move |style| {
                style
                    .border_color(focus_ring)
                    .shadow(shadows(&[Shadow::drop(
                        0.0,
                        0.0,
                        3.0,
                        focus_ring.fade(0.5),
                    )]))
            })
            .on_hover(move |hovered, window, cx| {
                track_hover(hover_state.clone())(hovered, window, cx);
                if let Some(tip) = tip_state.clone() {
                    tip.update(cx, |s, cx| {
                        let now = Instant::now();
                        if *hovered {
                            s.hovered_since = Some(now);
                        } else {
                            s.hovered_since = None;
                            if s.shown_at.take().is_some() {
                                s.hidden_at = Some(now);
                            }
                        }
                        cx.notify();
                    });
                }
            })
            .map(|d| {
                if pressable {
                    d.cursor_pointer()
                        .when_some(self.on_click, |d, handler| d.on_click(handler))
                } else {
                    d.cursor(CursorStyle::Arrow)
                }
            })
            .child(content)
            .children(tip_view)
    }
}

/// The tooltip above a trigger `side` points wide: a dark Depth chip with an arrow.
fn tooltip_bubble(
    label: SharedString,
    progress: f32,
    side: f32,
    theme: &crate::Theme,
) -> AnyElement {
    let skin = tooltip::TooltipSkin::new(theme);
    let span = 320.0;
    let tip = badge_arrow_tip();
    let arrow_color = skin.arrow;
    div()
        .absolute()
        .top_0()
        .left_0()
        .child(deferred(
            anchored()
                .anchor(Anchor::BottomLeft)
                .offset(point(
                    px(side / 2.0 - span / 2.0),
                    px(tip - tooltip::SIDE_OFFSET),
                ))
                .child(
                    div()
                        .w(px(span))
                        .flex()
                        .flex_col()
                        .items_center()
                        .opacity(progress)
                        .top(px(tooltip::FROM_OFFSET * (1.0 - progress)))
                        .child(
                            div()
                                .px(px(tooltip::PADDING_X))
                                .py(px(tooltip::PADDING_Y))
                                .rounded(px(tooltip::RADIUS))
                                .border_1()
                                .border_color(skin.border)
                                .bg(gradient(skin.top, skin.bottom))
                                .shadow(shadows(&skin.shadows))
                                .text_size(px(tooltip::FONT_SIZE))
                                .line_height(px(tooltip::LINE_HEIGHT))
                                .font_weight(FontWeight::NORMAL)
                                .font_family(fonts::SANS)
                                .text_color(skin.text)
                                .whitespace_nowrap()
                                .child(label),
                        )
                        .child(
                            canvas(
                                |_, _, _| {},
                                move |bounds, _, window, _| {
                                    let half = tip;
                                    let c = bounds.center().x;
                                    let top = bounds.top() - px(1.0);
                                    let mut path = PathBuilder::fill();
                                    path.move_to(point(c - px(half), top));
                                    path.line_to(point(c + px(half), top));
                                    path.line_to(point(c, top + px(half)));
                                    path.close();
                                    if let Ok(path) = path.build() {
                                        window.paint_path(path, arrow_color);
                                    }
                                },
                            )
                            .w(px(2.0 * tip))
                            .h(px(tip)),
                        ),
                ),
        ))
        .into_any_element()
}

/// How far the tooltip's arrow pokes out of the chip: the rotated square's half-diagonal
/// beyond the 2 px it sits inside, less the rounding of its corner.
fn badge_arrow_tip() -> f32 {
    let half_diagonal = tooltip::ARROW * std::f32::consts::FRAC_1_SQRT_2;
    let rounding = tooltip::ARROW_RADIUS * (std::f32::consts::SQRT_2 - 1.0);
    half_diagonal - tooltip::ARROW_INSET - rounding
}

struct SpinState {
    started: Instant,
}

/// A spinning `loader-circle` (`animate-spin`).
fn spinner(
    id: ElementId,
    size: f32,
    color: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let started = window
        .use_keyed_state(id, cx, |_, _| SpinState {
            started: Instant::now(),
        })
        .read(cx)
        .started;
    let turn = if cx.reduce_motion() {
        0.0
    } else {
        window.request_animation_frame();
        SPIN.phase(Instant::now() - started)
    };
    svg()
        .path(Icon::LoaderCircle.path())
        .size(px(size))
        .flex_none()
        .text_color(color)
        .with_transformation(Transformation::rotate(percentage(turn)))
}

fn child_id(id: &ElementId, name: &'static str) -> ElementId {
    ElementId::NamedChild(Arc::new(id.clone()), name.into())
}
