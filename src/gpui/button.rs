use std::time::{Duration, Instant};

use ::gpui::{
    AnyElement, App, ClickEvent, ElementId, FocusHandle, FontWeight, Hsla, IntoElement,
    ParentElement, RenderOnce, SharedString, Styled, Window, div, prelude::*, px, svg,
};

use super::{ActiveTheme, gradient, shadows};
use crate::components::button::{
    ButtonHover, ButtonSize, ButtonSkin, ButtonVariant, DISABLED_OPACITY, TRANSITION, loading,
};
use crate::fonts;
use crate::icons::Icon;
use crate::motion::Animated;
use crate::theme::{Color, Shadow};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// The Depth button.
///
/// ```ignore
/// Button::new("save", "Save changes")
///     .variant(ButtonVariant::Primary)
///     .loading(self.saving)
///     .on_click(cx.listener(|this, _, _, cx| this.save(cx)))
/// ```
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    icon: Option<Icon>,
    icon_end: Option<Icon>,
    children: Vec<AnyElement>,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    loading: bool,
    progress: Option<f32>,
    on_click: Option<ClickHandler>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            label: Some(label.into()),
            ..Self::icon_only(id, None)
        }
    }

    /// A square button showing only `icon`; pair it with an icon size.
    pub fn icon_button(id: impl Into<ElementId>, icon: Icon) -> Self {
        Self::icon_only(id, Some(icon)).size(ButtonSize::Icon)
    }

    fn icon_only(id: impl Into<ElementId>, icon: Option<Icon>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon,
            icon_end: None,
            children: Vec::new(),
            variant: ButtonVariant::Primary,
            size: ButtonSize::Default,
            disabled: false,
            loading: false,
            progress: None,
            on_click: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// An icon before the label.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// An icon after the label.
    pub fn icon_end(mut self, icon: Icon) -> Self {
        self.icon_end = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Swaps the label for bouncing dots while an action runs.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Fills the button from the left, `0.0..=100.0`. Set `None` when the work is
    /// done: the bar fills up and fades out.
    pub fn progress(mut self, progress: Option<f32>) -> Self {
        self.progress = progress;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// Per-button motion that outlives a frame.
struct ButtonState {
    focus: FocusHandle,
    hover: Animated,
    loading: bool,
    loading_since: Option<Instant>,
    has_progress: bool,
    completing_since: Option<Instant>,
    fill: Animated,
}

impl ButtonState {
    fn sync(&mut self, loading: bool, progress: Option<f32>, now: Instant) {
        if loading != self.loading {
            self.loading = loading;
            self.loading_since = Some(now);
        }
        match progress {
            Some(value) => {
                if !self.has_progress {
                    self.fill.snap(0.0);
                }
                self.has_progress = true;
                self.completing_since = None;
                self.fill.set(value.clamp(0.0, 100.0), now);
            }
            None if self.has_progress => {
                self.has_progress = false;
                self.completing_since = Some(now);
                self.fill.set(100.0, now);
            }
            None => {}
        }
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    if t <= 0.0 {
        a
    } else if t >= 1.0 {
        b
    } else {
        a.mix(b, t)
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let metrics = self.size.metrics();
        let now = Instant::now();
        let reduce_motion = cx.reduce_motion();

        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| ButtonState {
            focus: cx.focus_handle(),
            hover: Animated::new(0.0, TRANSITION),
            loading: self.loading,
            loading_since: None,
            has_progress: self.progress.is_some(),
            completing_since: None,
            fill: Animated::new(self.progress.unwrap_or(0.0), loading::PROGRESS_FILL),
        });
        state.update(cx, |s, _| s.sync(self.loading, self.progress, now));
        let s = state.read(cx);

        let since = |at: Option<Instant>| at.map_or(Duration::MAX, |at| now - at);
        let swap_elapsed = if reduce_motion {
            Duration::MAX
        } else {
            since(s.loading_since)
        };
        let swapping = swap_elapsed < loading::settle_time();
        let completing = s
            .completing_since
            .filter(|at| {
                reduce_motion || now - *at < loading::PROGRESS_FILL.total() + loading::PROGRESS_OUT
            })
            .map(|at| now - at);
        let show_loader = s.loading || swapping;
        let show_progress = s.has_progress || completing.is_some();
        let busy = self.loading || self.progress.is_some();
        let interactive = !self.disabled && !busy;

        let hover = if interactive { s.hover.value(now) } else { 0.0 };
        let fill_percent = s.fill.value(now);
        let animating = !reduce_motion
            && (s.hover.is_animating(now)
                || swapping
                || show_loader
                || s.fill.is_animating(now)
                || completing.is_some());
        let focus = s.focus.clone();
        if animating {
            window.request_animation_frame();
        }

        let rest = ButtonSkin::new(&theme, self.variant);
        let hot = rest.hovered();
        let (fill, border, text) = (
            lerp_color(rest.fill, hot.fill, hover),
            lerp_color(rest.border, hot.border, hover),
            lerp_color(rest.text, hot.text, hover),
        );
        let background = match (rest.gradient, hot.gradient) {
            (Some((t0, b0)), Some((t1, b1))) => {
                gradient(lerp_color(t0, t1, hover), lerp_color(b0, b1, hover))
            }
            _ => fill.into(),
        };
        let radius = rest.radius.unwrap_or(metrics.radius);
        let em = metrics.font_size;
        let text_hsla: Hsla = text.into();

        let icon_el = |icon: Icon| {
            svg()
                .path(icon.path())
                .size(px(metrics.icon_size))
                .flex_none()
                .text_color(text_hsla)
        };

        // The label keeps its width while hidden, so the button never jumps.
        let label_frame = if s.loading {
            if swapping {
                loading::swap_out(swap_elapsed)
            } else {
                loading::SwapFrame::HIDDEN
            }
        } else if swapping {
            loading::swap_in(swap_elapsed, loading::LABEL_RETURN_DELAY)
        } else {
            loading::SwapFrame::SHOWN
        };

        let content = div()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(metrics.gap))
            .min_w_0()
            .top(px(label_frame.offset_em * em))
            .opacity(label_frame.opacity)
            .when_some(self.icon, |d, icon| d.child(icon_el(icon)))
            .when_some(self.label.clone(), |d, label| {
                d.child(
                    div()
                        .when(
                            matches!(rest.hover, ButtonHover::Underline) && hover > 0.5,
                            |d| d.underline(),
                        )
                        .child(label),
                )
            })
            .children(self.children)
            .when_some(self.icon_end, |d, icon| d.child(icon_el(icon)));

        let loader = show_loader.then(|| {
            let dot_size = loading::DOT_SIZE_EM * em;
            let dots = (0..3).map(|i| {
                let enter = if s.loading && swapping {
                    loading::swap_in(swap_elapsed, loading::DOT_ENTER_DELAYS[i])
                } else {
                    loading::SwapFrame::SHOWN
                };
                let (bounce_opacity, bounce_offset) = if s.loading && !reduce_motion {
                    loading::dot(since(s.loading_since), i)
                } else {
                    (0.45, 0.0)
                };
                div()
                    .relative()
                    .top(px((enter.offset_em + bounce_offset) * em))
                    .opacity(enter.opacity * bounce_opacity)
                    .size(px(dot_size))
                    .rounded_full()
                    .bg(text_hsla)
            });
            let frame = if s.loading {
                loading::SwapFrame::SHOWN
            } else {
                loading::swap_out(swap_elapsed)
            };
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .gap(px(loading::DOT_GAP_EM * em))
                .top(px(frame.offset_em * em))
                .opacity(frame.opacity)
                .children(dots)
        });

        let progress = show_progress.then(|| {
            let opacity = completing
                .map(|elapsed| {
                    elapsed
                        .checked_sub(loading::PROGRESS_FILL.total())
                        .map_or(1.0, loading::progress_out)
                })
                .unwrap_or(1.0);
            div().absolute().inset_0().opacity(opacity).child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .w(::gpui::relative(fill_percent / 100.0))
                    .bg(text.fade(loading::PROGRESS_ALPHA)),
            )
        });

        let height = px(metrics.height);
        let on_click = self.on_click;
        let hover_state = state.clone();

        div()
            .id(self.id)
            .track_focus(&focus)
            .tab_stop(!self.disabled)
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .h(height)
            .map(|d| match metrics.padding_x {
                Some(padding) => d.px(px(padding)),
                None => d.w(height),
            })
            .rounded(px(radius))
            .border_1()
            .border_color(border)
            .bg(background)
            .shadow(shadows(&rest.shadows))
            .text_color(text_hsla)
            .text_size(px(metrics.font_size))
            .line_height(px(metrics.line_height))
            .font_family(fonts::SANS)
            .font_weight(FontWeight(500.0))
            .whitespace_nowrap()
            .when(show_loader || show_progress, |d| d.overflow_hidden())
            .when(self.disabled, |d| d.opacity(DISABLED_OPACITY))
            .focus_visible(move |style| {
                style
                    .border_color(theme.ring)
                    .shadow(shadows(&[Shadow::drop(
                        0.0,
                        0.0,
                        3.0,
                        theme.ring.fade(0.5),
                    )]))
            })
            .when(interactive, |d| {
                d.cursor_pointer()
                    .on_hover(move |hovered, _, cx| {
                        let target = if *hovered { 1.0 } else { 0.0 };
                        hover_state.update(cx, |s, cx| {
                            s.hover.set(target, Instant::now());
                            cx.notify();
                        });
                    })
                    .when_some(on_click, |d, handler| d.on_click(handler))
            })
            .when(busy, |d| d.cursor(::gpui::CursorStyle::OperationNotAllowed))
            .children(progress)
            .child(content)
            .children(loader)
    }
}
