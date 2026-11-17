//! GPUI renderings of the small parts the data surfaces share. See
//! [`crate::components::data_parts`] for the measurements.

use std::f32::consts::TAU;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use ::gpui::{
    AnimationExt as _, AnyElement, AnyView, App, AppContext as _, ClickEvent, ClipboardItem,
    Context, ElementId, Entity, FocusHandle, FontWeight, HighlightStyle, Hsla, IntoElement,
    ParentElement, Render, RenderOnce, Role, SharedString, StyledText, TextRun, Toggled,
    Transformation, Window, div, font, prelude::*, px, radians, size, svg,
};

use super::{ActiveTheme, gradient, shadows};
use crate::components::data_parts::{
    badge, checkbox, copy, empty, lerp_color, shimmer, skeleton, spinner, tabs, tooltip,
};
use crate::fonts;
use crate::icons::Icon;
use crate::motion::{Animated, Tween};
use crate::theme::{Color, Shadow, Theme};

pub use crate::components::data_parts::badge::BadgeVariant;
pub use crate::components::data_parts::checkbox::CheckState;

fn clock() -> Duration {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed()
}

/// A value easing toward a hover or focus target, kept per element.
pub(crate) struct Eased(pub(crate) Animated);

/// Reads (and keeps animating) an eased 0..1 value stored under `id`.
pub(crate) fn use_eased(
    id: impl Into<ElementId>,
    tween: Tween,
    window: &mut Window,
    cx: &mut App,
) -> (Entity<Eased>, f32) {
    let state = window.use_keyed_state(id.into(), cx, |_, _| Eased(Animated::new(0.0, tween)));
    let now = Instant::now();
    let anim = state.read(cx).0;
    if anim.is_animating(now) && !cx.reduce_motion() {
        window.request_animation_frame();
    }
    let value = if cx.reduce_motion() {
        anim.target()
    } else {
        anim.value(now)
    };
    (state, value)
}

/// A hover listener that eases `state` to 1 on enter and back to 0 on leave.
pub(crate) fn ease_on_hover(state: Entity<Eased>) -> impl Fn(&bool, &mut Window, &mut App) {
    move |hovered, _, cx| {
        let target = if *hovered { 1.0 } else { 0.0 };
        state.update(cx, |s, cx| {
            s.0.set(target, Instant::now());
            cx.notify();
        });
    }
}

/// The keyboard focus ring of Notra's controls: `ring-3 ring-ring/50`.
pub(crate) fn focus_ring(theme: &Theme) -> Vec<::gpui::BoxShadow> {
    shadows(&[Shadow::drop(0.0, 0.0, 3.0, theme.ring.fade(0.5))])
}

/// lucide's `loader-circle`, turning once a second.
#[derive(IntoElement)]
pub struct Spinner {
    size: f32,
    color: Option<Color>,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            size: spinner::SIZE,
            color: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Spinner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self.color.unwrap_or(cx.notra().muted_foreground);
        let angle = if cx.reduce_motion() {
            0.0
        } else {
            window.request_animation_frame();
            spinner::TURN.phase(clock()) * TAU
        };
        svg()
            .path(Icon::LoaderCircle.path())
            .size(px(self.size))
            .flex_none()
            .text_color(color)
            .with_transformation(Transformation::rotate(radians(angle)))
    }
}

/// Text with Notra's shimmer sweeping across it.
#[derive(IntoElement)]
pub struct Shimmer {
    id: ElementId,
    text: SharedString,
    size: f32,
    line_height: f32,
    color: Option<Color>,
}

impl Shimmer {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            size: 14.0,
            line_height: 20.0,
            color: None,
        }
    }

    pub fn text_size(mut self, size: f32, line_height: f32) -> Self {
        self.size = size;
        self.line_height = line_height;
        self
    }

    /// The text's own colour under the band (muted by default).
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl RenderOnce for Shimmer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let base = self.color.unwrap_or(theme.muted_foreground);
        let started = window.use_keyed_state(self.id, cx, |_, _| Instant::now());
        let elapsed = started.read(cx).elapsed();
        let mut text = StyledText::new(self.text.clone());
        if !cx.reduce_motion() {
            window.request_animation_frame();
            let run = TextRun {
                len: self.text.len(),
                font: font(fonts::SANS),
                color: base.into(),
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let line =
                window
                    .text_system()
                    .shape_line(self.text.clone(), px(self.size), &[run], None);
            let width = f32::from(line.width);
            let center = shimmer::center(shimmer::SWEEP.phase(elapsed), width);
            let spread = shimmer::SPREAD_EM * self.size;
            let highlights = self.text.char_indices().map(|(i, ch)| {
                let end = i + ch.len_utf8();
                let x = (f32::from(line.x_for_index(i)) + f32::from(line.x_for_index(end))) / 2.0;
                let color = shimmer::color_at(base, theme.foreground, x, center, spread);
                (
                    i..end,
                    HighlightStyle {
                        color: Some(color.into()),
                        ..Default::default()
                    },
                )
            });
            text = text.with_highlights(highlights.collect::<Vec<_>>());
        }
        div()
            .text_size(px(self.size))
            .line_height(px(self.line_height))
            .text_color(base)
            .whitespace_nowrap()
            .child(text)
    }
}

/// The base-ui checkbox.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    state: CheckState,
    ink: Option<Color>,
    on_toggle: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, state: CheckState) -> Self {
        Self {
            id: id.into(),
            state,
            ink: None,
            on_toggle: None,
        }
    }

    /// The colour the mixed state's check takes (the surrounding text colour).
    pub fn ink(mut self, ink: Color) -> Self {
        self.ink = Some(ink);
        self
    }

    pub fn on_toggle(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }
}

struct CheckboxState {
    focus: FocusHandle,
    on: Animated,
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let ink = self.ink.unwrap_or(theme.foreground);
        let target = if self.state == CheckState::On {
            1.0
        } else {
            0.0
        };
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| CheckboxState {
            focus: cx.focus_handle(),
            on: Animated::new(target, checkbox::TRANSITION),
        });
        let now = Instant::now();
        state.update(cx, |s, _| s.on.set(target, now));
        let s = state.read(cx);
        let on = if cx.reduce_motion() {
            target
        } else {
            s.on.value(now)
        };
        if s.on.is_animating(now) && !cx.reduce_motion() {
            window.request_animation_frame();
        }
        let focus = s.focus.clone();
        let off = checkbox::CheckboxSkin::new(&theme, CheckState::Off, ink);
        let lit = checkbox::CheckboxSkin::new(&theme, CheckState::On, ink);
        let mark = checkbox::CheckboxSkin::new(&theme, self.state, ink).mark;
        div()
            .id(self.id)
            .role(Role::CheckBox)
            .aria_toggled(match self.state {
                CheckState::On => Toggled::True,
                CheckState::Off => Toggled::False,
                CheckState::Mixed => Toggled::Mixed,
            })
            .track_focus(&focus)
            .tab_stop(true)
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(checkbox::SIZE))
            .rounded(px(checkbox::RADIUS))
            .border_1()
            .border_color(lerp_color(off.border, lit.border, on))
            .bg(lerp_color(off.fill, lit.fill, on))
            .cursor_pointer()
            .focus_visible(move |s| s.border_color(theme.ring).shadow(focus_ring(&theme)))
            .when_some(mark, |d, mark| {
                d.child(
                    svg()
                        .path(Icon::Check.path())
                        .size(px(checkbox::ICON))
                        .text_color(mark),
                )
            })
            .when_some(self.on_toggle, |d, toggle| {
                d.on_click(move |_: &ClickEvent, window, cx| {
                    cx.stop_propagation();
                    toggle(window, cx)
                })
            })
    }
}

/// shadcn's badge.
#[derive(IntoElement)]
pub struct Badge {
    label: SharedString,
    variant: BadgeVariant,
}

impl Badge {
    pub fn new(label: impl Into<SharedString>, variant: BadgeVariant) -> Self {
        Self {
            label: label.into(),
            variant,
        }
    }
}

impl RenderOnce for Badge {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let skin = badge::BadgeSkin::new(cx.notra(), self.variant);
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .h(px(badge::HEIGHT))
            .px(px(badge::PADDING_X))
            .rounded(px(badge::RADIUS))
            .border_1()
            .border_color(skin.border)
            .bg(skin.fill)
            .text_color(skin.text)
            .text_size(px(badge::FONT_SIZE))
            .line_height(px(badge::LINE_HEIGHT))
            .font_weight(FontWeight(500.0))
            .whitespace_nowrap()
            .child(self.label)
    }
}

/// Notra's tooltip, for `.tooltip(Tooltip::text(..))`.
pub struct Tooltip {
    text: SharedString,
}

impl Tooltip {
    /// A tooltip builder showing `text`.
    pub fn text(text: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView {
        let text = text.into();
        move |_, cx| cx.new(|_| Tooltip { text: text.clone() }).into()
    }
}

impl Render for Tooltip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.notra();
        let skin = tooltip::TooltipSkin::new(&theme);
        let enter = tooltip::ENTER;
        let body = div()
            .max_w(px(tooltip::MAX_WIDTH + 2.0 * tooltip::PADDING_X))
            .px(px(tooltip::PADDING_X))
            .py(px(tooltip::PADDING_Y))
            .rounded(px(tooltip::RADIUS))
            .border_1()
            .border_color(skin.border)
            .bg(gradient(skin.top, skin.bottom))
            .shadow(shadows(&tooltip::SHADOWS))
            .text_color(skin.text)
            .text_size(px(tooltip::FONT_SIZE))
            .line_height(px(tooltip::LINE_HEIGHT))
            .font_family(fonts::SANS)
            .child(self.text.clone());
        div()
            .pl(px(4.0))
            .pt(px(tooltip::GAP))
            .child(if cx.reduce_motion() {
                body.into_any_element()
            } else {
                body.with_animation(
                    "notra-tooltip-enter",
                    ::gpui::Animation::new(enter.duration)
                        .with_easing(move |t| enter.easing.ease(t)),
                    |d, t| d.opacity(t),
                )
                .into_any_element()
            })
    }
}

/// How a copy button names itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CopyLabel {
    /// The code block's `Copy {label}` / `{label} copied`.
    Snippet(SharedString),
    /// The package command's `Copy command` / `Copied`.
    Command,
}

/// A copy button that swaps its icon for a tick for 1.5 s.
#[derive(IntoElement)]
pub struct CopyButton {
    id: ElementId,
    text: SharedString,
    metrics: copy::CopyMetrics,
    label: CopyLabel,
}

impl CopyButton {
    pub fn new(
        id: impl Into<ElementId>,
        text: impl Into<SharedString>,
        metrics: copy::CopyMetrics,
        label: CopyLabel,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            metrics,
            label,
        }
    }
}

struct CopyState {
    focus: FocusHandle,
    hover: Animated,
    feedback: copy::CopyFeedback,
}

impl RenderOnce for CopyButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| CopyState {
            focus: cx.focus_handle(),
            hover: Animated::new(0.0, copy::HOVER),
            feedback: copy::CopyFeedback::default(),
        });
        let now = Instant::now();
        let s = state.read(cx);
        let reduce = cx.reduce_motion();
        if !reduce && (s.hover.is_animating(now) || s.feedback.is_animating(now)) {
            window.request_animation_frame();
        }
        let hover = if reduce {
            s.hover.target()
        } else {
            s.hover.value(now)
        };
        let icons = if reduce {
            copy::icon_targets(s.feedback.status(now))
        } else {
            s.feedback.icons(now)
        };
        let focus = s.focus.clone();
        let m = self.metrics;
        let ink = lerp_color(theme.muted_foreground, theme.foreground, hover);
        let glyph = |icon: Icon, (opacity, scale): (f32, f32), color: Color| {
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .opacity(opacity)
                .child(
                    svg()
                        .path(icon.path())
                        .size(px(m.icon))
                        .text_color(color)
                        .with_transformation(Transformation::scale(size(scale, scale))),
                )
        };
        let text = self.text.clone();
        let click_state = state.clone();
        let hover_state = state.clone();
        let status = s.feedback.status(now);
        let label = match &self.label {
            CopyLabel::Snippet(label) => copy::snippet_label(status, label),
            CopyLabel::Command => copy::command_label(status).to_owned(),
        };
        div()
            .id(self.id)
            .role(Role::Button)
            .aria_label(label)
            .track_focus(&focus)
            .tab_stop(true)
            .relative()
            .flex_none()
            .size(px(m.size))
            .rounded(px(m.radius))
            .bg(theme.muted.fade(hover))
            .cursor_pointer()
            .focus_visible(move |s| s.border_color(theme.ring).shadow(focus_ring(&theme)))
            .on_hover(move |hovered, _, cx| {
                let target = if *hovered { 1.0 } else { 0.0 };
                hover_state.update(cx, |s, cx| {
                    s.hover.set(target, Instant::now());
                    cx.notify();
                });
            })
            .on_click(move |_, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
                click_state.update(cx, |s, cx| {
                    s.feedback.record(true, Instant::now());
                    cx.notify();
                });
            })
            .child(glyph(Icon::HugeCopy01, icons[0], ink))
            .child(glyph(Icon::HugeTick02, icons[1], ink))
            .child(glyph(Icon::HugeAlert02, icons[2], theme.destructive))
    }
}

/// One tab of [`Tabs`].
#[derive(Clone)]
pub struct TabOption {
    pub label: SharedString,
    pub icon: Option<Icon>,
}

impl TabOption {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}

type TabHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// base-ui tabs with the sliding underline, as the code block and package
/// command draw them on their shell.
#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    options: Vec<TabOption>,
    selected: usize,
    mono: bool,
    on_change: Option<TabHandler>,
}

impl Tabs {
    pub fn new(id: impl Into<ElementId>, options: Vec<TabOption>, selected: usize) -> Self {
        Self {
            id: id.into(),
            options,
            selected,
            mono: false,
            on_change: None,
        }
    }

    /// Set labels in the mono family (the package command).
    pub fn mono(mut self, mono: bool) -> Self {
        self.mono = mono;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

struct TabsState {
    left: Animated,
    width: Animated,
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let family = if self.mono { fonts::MONO } else { fonts::SANS };
        let widths: Vec<f32> = self
            .options
            .iter()
            .map(|option| {
                let run = TextRun {
                    len: option.label.len(),
                    font: font(family),
                    color: theme.foreground.into(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let line = window.text_system().shape_line(
                    option.label.clone(),
                    px(tabs::FONT_SIZE),
                    &[run],
                    None,
                );
                let icon = option.icon.map_or(0.0, |_| tabs::ICON + tabs::ICON_GAP);
                f32::from(line.width) + icon
            })
            .collect();
        let selected = self.selected.min(self.options.len().saturating_sub(1));
        let left: f32 = widths[..selected].iter().map(|w| w + tabs::GAP).sum();
        let width = widths.get(selected).copied().unwrap_or(0.0);
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| TabsState {
            left: Animated::new(left, tabs::SLIDE),
            width: Animated::new(width, tabs::SLIDE),
        });
        let now = Instant::now();
        state.update(cx, |s, _| {
            s.left.set(left, now);
            s.width.set(width, now);
        });
        let s = state.read(cx);
        let reduce = cx.reduce_motion();
        if !reduce && (s.left.is_animating(now) || s.width.is_animating(now)) {
            window.request_animation_frame();
        }
        let (x, w) = if reduce {
            (left, width)
        } else {
            (s.left.value(now), s.width.value(now))
        };
        let id = self.id.clone();
        let tabs_el = self.options.into_iter().enumerate().map(|(i, option)| {
            let active = i == selected;
            let (hover_state, hover) = use_eased(
                SharedString::from(format!("{id}-tab-{i}")),
                tabs::COLOR,
                window,
                cx,
            );
            let ink = if active {
                theme.foreground
            } else {
                lerp_color(theme.muted_foreground, theme.foreground, hover)
            };
            let on_change = self.on_change.clone();
            div()
                .id(SharedString::from(format!("{id}-tab-{i}-hit")))
                .role(Role::Tab)
                .aria_selected(active)
                .flex()
                .flex_none()
                .items_center()
                .gap(px(tabs::ICON_GAP))
                .rounded(px(4.0))
                .cursor_pointer()
                .text_color(ink)
                .when_some(option.icon, |d, icon| {
                    d.child(
                        svg()
                            .path(icon.path())
                            .size(px(tabs::ICON))
                            .flex_none()
                            .text_color(ink),
                    )
                })
                .child(option.label)
                .on_hover(ease_on_hover(hover_state))
                .when_some(on_change, |d, handler| {
                    d.on_click(move |_, window, cx| handler(i, window, cx))
                })
        });
        div()
            .id(self.id)
            .role(Role::TabList)
            .relative()
            .flex()
            .h(px(tabs::HEIGHT))
            .gap(px(tabs::GAP))
            .text_size(px(tabs::FONT_SIZE))
            .line_height(px(tabs::LINE_HEIGHT))
            .font_family(family)
            .children(tabs_el)
            .child(
                div()
                    .absolute()
                    .left(px(x))
                    .bottom(px(tabs::INDICATOR_BOTTOM))
                    .w(px(w))
                    .h(px(tabs::INDICATOR_HEIGHT))
                    .rounded_full()
                    .bg(theme.foreground),
            )
    }
}

/// shadcn's skeleton bar, pulsing.
#[derive(IntoElement)]
pub struct Skeleton {
    width: Option<::gpui::Length>,
    height: f32,
    radius: f32,
}

impl Skeleton {
    pub fn new(height: f32) -> Self {
        Self {
            width: None,
            height,
            radius: 6.0,
        }
    }

    pub fn w(mut self, width: impl Into<::gpui::Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
}

impl RenderOnce for Skeleton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let opacity = if cx.reduce_motion() {
            1.0
        } else {
            window.request_animation_frame();
            skeleton::opacity(clock())
        };
        div()
            .flex_none()
            .h(px(self.height))
            .when_some(self.width, |d, w| d.w(w))
            .rounded(px(self.radius))
            .bg(cx.notra().muted)
            .opacity(opacity)
    }
}

/// shadcn's `Empty`: an icon tile, a title, a description and actions.
#[derive(IntoElement)]
pub struct EmptyState {
    icon: Icon,
    title: SharedString,
    description: Option<SharedString>,
    children: Vec<AnyElement>,
}

impl EmptyState {
    pub fn new(icon: Icon, title: impl Into<SharedString>) -> Self {
        Self {
            icon,
            title: title.into(),
            description: None,
            children: Vec::new(),
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl ParentElement for EmptyState {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for EmptyState {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.notra();
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(empty::GAP))
            .p(px(empty::PADDING))
            .text_size(px(14.0))
            .line_height(px(20.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(empty::HEADER_GAP))
                    .max_w(px(empty::DESCRIPTION_MAX_WIDTH))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .mb(px(empty::MEDIA_MARGIN))
                            .size(px(empty::MEDIA_SIZE))
                            .rounded(px(empty::MEDIA_RADIUS))
                            .bg(theme.muted)
                            .child(
                                svg()
                                    .path(self.icon.path())
                                    .size(px(empty::MEDIA_ICON))
                                    .text_color(theme.foreground),
                            ),
                    )
                    .child(
                        div()
                            .text_color(theme.foreground)
                            .font_weight(FontWeight(500.0))
                            .child(self.title),
                    )
                    .when_some(self.description, |d, text| {
                        d.child(
                            div()
                                .text_center()
                                .text_color(theme.muted_foreground)
                                .line_height(px(empty::DESCRIPTION_LINE_HEIGHT))
                                .child(text),
                        )
                    }),
            )
            .when(!self.children.is_empty(), |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(empty::CONTENT_GAP))
                        .children(self.children),
                )
            })
    }
}

/// An inline icon in `currentColor`, for cells and toolbars.
pub fn glyph(icon: Icon, size_px: f32, color: impl Into<Hsla>) -> ::gpui::Svg {
    svg()
        .path(icon.path())
        .size(px(size_px))
        .flex_none()
        .text_color(color)
}
