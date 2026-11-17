use std::time::Instant;

use ::gpui::{
    AnyElement, App, ClickEvent, ElementId, Hsla, IntoElement, ParentElement, RenderOnce, Styled,
    Transformation, Window, div, prelude::*, px, radians, svg,
};

use crate::components::primitives::collapsible::{CHEVRON, PANEL, panel_frame};
use crate::icons::Icon;
use crate::motion::Animated;
use crate::theme::Color;

type ToggleHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

/// A disclosure: a trigger that opens and closes an animated panel.
///
/// ```ignore
/// Collapsible::new("sources", self.open)
///     .trigger(div().child("Sources").child(CollapsibleChevron::new("sources-chevron", self.open)))
///     .child(sources_list)
///     .on_open_change(cx.listener(|this, open, _, cx| { this.open = *open; cx.notify() }))
/// ```
#[derive(IntoElement)]
pub struct Collapsible {
    id: ElementId,
    open: bool,
    disabled: bool,
    trigger: Option<AnyElement>,
    children: Vec<AnyElement>,
    on_open_change: Option<ToggleHandler>,
}

impl Collapsible {
    pub fn new(id: impl Into<ElementId>, open: bool) -> Self {
        Self {
            id: id.into(),
            open,
            disabled: false,
            trigger: None,
            children: Vec::new(),
            on_open_change: None,
        }
    }

    pub fn trigger(mut self, trigger: impl IntoElement) -> Self {
        self.trigger = Some(trigger.into_any_element());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Called with the requested state when the trigger is pressed.
    pub fn on_open_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Box::new(handler));
        self
    }
}

impl ParentElement for Collapsible {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Collapsible {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let next = !self.open;
        let trigger_id = ElementId::Name(format!("{}-trigger", self.id).into());
        let panel_id = ElementId::Name(format!("{}-panel", self.id).into());
        let handler = self.on_open_change;
        let interactive = !self.disabled && handler.is_some();
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .id(trigger_id)
                    .tab_stop(interactive)
                    .when(interactive, |d| d.cursor_pointer())
                    .when_some(handler.filter(|_| interactive), move |d, handler| {
                        d.on_click(move |_: &ClickEvent, window, cx| handler(&next, window, cx))
                    })
                    .children(self.trigger),
            )
            .child(CollapsiblePanel::new(panel_id, self.open).children(self.children))
    }
}

/// The animated panel alone, for triggers built elsewhere.
#[derive(IntoElement)]
pub struct CollapsiblePanel {
    id: ElementId,
    open: bool,
    children: Vec<AnyElement>,
}

impl CollapsiblePanel {
    pub fn new(id: impl Into<ElementId>, open: bool) -> Self {
        Self {
            id: id.into(),
            open,
            children: Vec::new(),
        }
    }
}

impl ParentElement for CollapsiblePanel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

struct PanelState {
    progress: Animated,
    height: f32,
}

impl RenderOnce for CollapsiblePanel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let now = Instant::now();
        let target = if self.open { 1.0 } else { 0.0 };
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| PanelState {
            progress: Animated::new(target, PANEL),
            height: 0.0,
        });
        let reduce_motion = cx.reduce_motion();
        state.update(cx, |s, _| {
            if reduce_motion {
                s.progress.snap(target);
            } else {
                s.progress.set(target, now);
            }
        });
        let s = state.read(cx);
        let progress = s.progress.value(now);
        let animating = s.progress.is_animating(now);
        if animating {
            window.request_animation_frame();
        }
        if progress <= 0.0 && !self.open {
            return div().id(self.id);
        }
        let (height, opacity) = panel_frame(s.height, progress);
        let measure = state.clone();
        div()
            .on_children_prepainted(move |bounds, _, cx| {
                let Some(first) = bounds.first() else {
                    return;
                };
                let measured = f32::from(first.size.height);
                measure.update(cx, |s, cx| {
                    if (s.height - measured).abs() > 0.5 {
                        s.height = measured;
                        cx.notify();
                    }
                });
            })
            .id(self.id)
            .flex()
            .flex_col()
            .overflow_hidden()
            .opacity(opacity)
            .when(animating || progress < 1.0, |d| d.h(px(height)))
            .child(div().flex().flex_col().flex_none().children(self.children))
    }
}

/// A chevron that turns half a turn while its collapsible is open.
#[derive(IntoElement)]
pub struct CollapsibleChevron {
    id: ElementId,
    open: bool,
    icon: Icon,
    size: f32,
    color: Option<Color>,
}

impl CollapsibleChevron {
    pub fn new(id: impl Into<ElementId>, open: bool) -> Self {
        Self {
            id: id.into(),
            open,
            icon: Icon::ChevronDown,
            size: 16.0,
            color: None,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
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

impl RenderOnce for CollapsibleChevron {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let now = Instant::now();
        let target = if self.open { 1.0 } else { 0.0 };
        let turn = window.use_keyed_state(self.id, cx, |_, _| Animated::new(target, CHEVRON));
        let reduce_motion = cx.reduce_motion();
        turn.update(cx, |a, _| {
            if reduce_motion {
                a.snap(target);
            } else {
                a.set(target, now);
            }
        });
        let a = turn.read(cx);
        if a.is_animating(now) {
            window.request_animation_frame();
        }
        let angle = a.value(now) * std::f32::consts::PI;
        let color = self
            .color
            .map_or_else(|| window.text_style().color, Hsla::from);
        svg()
            .path(self.icon.path())
            .size(px(self.size))
            .flex_none()
            .text_color(color)
            .with_transformation(Transformation::rotate(radians(angle)))
    }
}
