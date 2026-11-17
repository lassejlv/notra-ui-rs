use std::time::Instant;

use ::gpui::{
    App, ElementId, FocusHandle, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::*, px, svg,
};

use crate::components::primitives::checkbox::{self, CheckState, CheckboxSkin};
use crate::components::primitives::{COLOR_TRANSITION, focus_ring};
use crate::gpui::{ActiveTheme, shadows};
use crate::motion::Animated;
use crate::theme::Shadow;

type ChangeHandler = Box<dyn Fn(&CheckState, &mut Window, &mut App) + 'static>;

/// A checkbox: `Checkbox::new("terms", self.agreed.into()).on_change(..)`. Space and
/// clicks toggle it; the handler receives the next state.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    state: CheckState,
    disabled: bool,
    on_change: Option<ChangeHandler>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, state: CheckState) -> Self {
        Self {
            id: id.into(),
            state,
            disabled: false,
            on_change: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(&CheckState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
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
        let now = Instant::now();
        let target = if self.state.is_on() { 1.0 } else { 0.0 };
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| CheckboxState {
            focus: cx.focus_handle(),
            on: Animated::new(target, COLOR_TRANSITION),
        });
        let reduce_motion = cx.reduce_motion();
        state.update(cx, |s, _| {
            if reduce_motion {
                s.on.snap(target);
            } else {
                s.on.set(target, now);
            }
        });
        let s = state.read(cx);
        let t = s.on.value(now);
        if s.on.is_animating(now) {
            window.request_animation_frame();
        }
        let focus = s.focus.clone();
        let off = CheckboxSkin::new(&theme, CheckState::Unchecked);
        let on = CheckboxSkin::new(&theme, CheckState::Checked);
        let fill = off.fill.mix(on.fill, t);
        let border = off.border.mix(on.border, t);
        let focus_border = CheckboxSkin::new(&theme, self.state)
            .focused(&theme, self.state)
            .border;
        let (ring_width, ring) = focus_ring(&theme);
        let next = self.state.toggled();
        div()
            .id(self.id)
            .track_focus(&focus)
            .tab_stop(!self.disabled)
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(checkbox::SIZE))
            .rounded(px(checkbox::RADIUS))
            .border_1()
            .border_color(border)
            .bg(fill)
            .focus_visible(move |s| {
                s.border_color(focus_border)
                    .shadow(shadows(&[Shadow::drop(0.0, 0.0, ring_width, ring)]))
            })
            .when_some(self.state.icon(), |d, icon| {
                d.child(
                    svg()
                        .path(icon.path())
                        .size(px(checkbox::ICON_SIZE))
                        .text_color(on.mark),
                )
            })
            .map(|d| {
                if self.disabled {
                    d.opacity(checkbox::DISABLED_OPACITY)
                        .cursor(::gpui::CursorStyle::OperationNotAllowed)
                } else {
                    d.cursor_pointer()
                        .when_some(self.on_change, move |d, handler| {
                            d.on_click(move |_, window, cx| handler(&next, window, cx))
                        })
                }
            })
    }
}
