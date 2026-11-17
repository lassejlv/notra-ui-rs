use std::time::{Duration, Instant};

use ::gpui::{
    AnyElement, App, ClickEvent, ElementId, FocusHandle, FontWeight, Hsla, IntoElement,
    MouseButton, ParentElement, RenderOnce, SharedString, Styled, Window, div, prelude::*, px, svg,
};

use super::spinner::spinning_icon;
use crate::components::button::loading::{SwapFrame, swap_in, swap_out};
use crate::components::primitives::focus_ring;
use crate::components::primitives::marketing_button::{
    DISABLED_OPACITY, GAP, MarketingSize, MarketingSkin, MarketingVariant, TRANSITION, WEIGHT,
    loading,
};
use crate::fonts;
use crate::gpui::{ActiveTheme, gradient, shadows};
use crate::icons::Icon;
use crate::motion::Animated;
use crate::theme::Shadow;

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Notra's marketing call to action.
///
/// ```ignore
/// MarketingButton::new("start", "Start for free").icon_end(Icon::ArrowRight)
/// ```
#[derive(IntoElement)]
pub struct MarketingButton {
    id: ElementId,
    label: SharedString,
    icon: Option<Icon>,
    icon_end: Option<Icon>,
    variant: MarketingVariant,
    size: MarketingSize,
    disabled: bool,
    loading: bool,
    on_click: Option<ClickHandler>,
}

impl MarketingButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            icon_end: None,
            variant: MarketingVariant::Primary,
            size: MarketingSize::Default,
            disabled: false,
            loading: false,
            on_click: None,
        }
    }

    pub fn variant(mut self, variant: MarketingVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: MarketingSize) -> Self {
        self.size = size;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn icon_end(mut self, icon: Icon) -> Self {
        self.icon_end = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Swaps the label for a spinner; the button keeps its width.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
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

struct MarketingState {
    focus: FocusHandle,
    hover: Animated,
    press: Animated,
    loading: bool,
    loading_since: Option<Instant>,
}

impl RenderOnce for MarketingButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let m = self.size.metrics();
        let skin = MarketingSkin::new(self.variant);
        let now = Instant::now();
        let reduce_motion = cx.reduce_motion();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| MarketingState {
            focus: cx.focus_handle(),
            hover: Animated::new(0.0, TRANSITION),
            press: Animated::new(0.0, TRANSITION),
            loading: self.loading,
            loading_since: None,
        });
        state.update(cx, |s, _| {
            if s.loading != self.loading {
                s.loading = self.loading;
                s.loading_since = Some(now);
            }
        });
        let s = state.read(cx);
        let swap_elapsed = match s.loading_since {
            Some(at) if !reduce_motion => now - at,
            _ => Duration::MAX,
        };
        let swapping = swap_elapsed < loading::settle_time();
        let interactive = !self.disabled && !self.loading;
        let (hover, press) = if interactive {
            (s.hover.value(now), s.press.value(now))
        } else {
            (0.0, 0.0)
        };
        if swapping || s.hover.is_animating(now) || s.press.is_animating(now) {
            window.request_animation_frame();
        }
        let focus = s.focus.clone();
        let stops = skin.stops(hover, press);
        let text: Hsla = skin.text.into();
        let em = m.font_size;

        let label_frame = match (s.loading, swapping) {
            (true, true) => swap_out(swap_elapsed),
            (true, false) => SwapFrame::HIDDEN,
            (false, true) => swap_in(swap_elapsed, loading::ENTER_DELAY),
            (false, false) => SwapFrame::SHOWN,
        };
        let glyph = |icon: Icon| {
            svg()
                .path(icon.path())
                .size(px(m.icon_size))
                .flex_none()
                .text_color(text)
        };
        let label = div()
            .relative()
            .flex()
            .items_center()
            .gap(px(GAP))
            .top(px(label_frame.offset_em * em))
            .opacity(label_frame.opacity)
            .when_some(self.icon, |d, icon| d.child(glyph(icon)))
            .child(self.label.clone())
            .when_some(self.icon_end, |d, icon| d.child(glyph(icon)));

        let spinner: Option<AnyElement> = (s.loading || swapping).then(|| {
            let frame = if s.loading {
                if swapping {
                    swap_in(swap_elapsed, loading::ENTER_DELAY)
                } else {
                    SwapFrame::SHOWN
                }
            } else {
                swap_out(swap_elapsed)
            };
            let size = loading::SPINNER_EM * em;
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .top(px(frame.offset_em * em))
                .opacity(frame.opacity)
                .child(
                    div()
                        .relative()
                        .size(px(size))
                        .child(
                            svg()
                                .path(Icon::SpinnerTrack.path())
                                .absolute()
                                .inset_0()
                                .size(px(size))
                                .text_color(skin.text.fade(loading::TRACK_OPACITY)),
                        )
                        .child(div().absolute().inset_0().child(spinning_icon(
                            ElementId::Name(format!("{}-spinner", self.id).into()),
                            Icon::SpinnerArc,
                            size,
                            text,
                            reduce_motion,
                        ))),
                )
                .into_any_element()
        });

        let (ring_width, ring) = focus_ring(&theme);
        let mut focused = skin.shadows.clone();
        focused.push(Shadow::drop(0.0, 0.0, ring_width, ring));
        let rest_shadows = shadows(&skin.shadows);
        let hover_state = state.clone();
        let press_state = state.clone();
        let release_state = state.clone();
        let release_out_state = state.clone();
        let set = |state: ::gpui::Entity<MarketingState>,
                   pick: fn(&mut MarketingState) -> &mut Animated,
                   to: f32,
                   cx: &mut App| {
            state.update(cx, |s, cx| {
                pick(s).set(to, Instant::now());
                cx.notify();
            });
        };
        div()
            .id(self.id)
            .track_focus(&focus)
            .tab_stop(!self.disabled)
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(GAP))
            .h(px(m.height))
            .px(px(m.padding_x))
            .rounded_full()
            .bg(gradient(stops.from, stops.to))
            .shadow(rest_shadows)
            .text_color(text)
            .text_size(px(m.font_size))
            .line_height(px(m.line_height))
            .font_family(fonts::SANS)
            .font_weight(FontWeight(WEIGHT))
            .whitespace_nowrap()
            .when(s.loading || swapping, |d| d.overflow_hidden())
            .when(self.disabled, |d| d.opacity(DISABLED_OPACITY))
            .focus_visible(move |style| style.shadow(shadows(&focused)))
            .when(interactive, |d| {
                d.cursor_pointer()
                    .on_hover(move |hovered, _, cx| {
                        let to = if *hovered { 1.0 } else { 0.0 };
                        set(hover_state.clone(), |s| &mut s.hover, to, cx);
                    })
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        set(press_state.clone(), |s| &mut s.press, 1.0, cx);
                    })
                    .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                        set(release_state.clone(), |s| &mut s.press, 0.0, cx);
                    })
                    .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                        set(release_out_state.clone(), |s| &mut s.press, 0.0, cx);
                    })
                    .when_some(self.on_click, |d, handler| d.on_click(handler))
            })
            .child(label)
            .children(spinner)
    }
}
