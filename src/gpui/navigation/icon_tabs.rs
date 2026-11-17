use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use ::gpui::{
    App, Bounds, ElementId, FocusHandle, FontWeight, IntoElement, ParentElement, Pixels,
    RenderOnce, SharedString, Styled, Transformation, Window, canvas, div, prelude::*, px, size,
    svg,
};

use super::{mix, seconds};
use crate::components::navigation::icon_tabs::{
    COLOR, ICON_REVEAL, IconTabsSkin, IndicatorSlide, TabKey, metrics,
};
use crate::fonts;
use crate::gpui::{ActiveTheme, shadows};
use crate::icons::Icon;
use crate::motion::Animated;
use crate::theme::Shadow;

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Segmented tabs where only the active tab shows its icon.
///
/// ```ignore
/// IconTabs::new("views", self.tab)
///     .tab(Icon::Send, "Deliveries")
///     .tab(Icon::Link, "Endpoints")
///     .width(px(256.0))
///     .on_change(cx.listener(|this, ix, _, cx| { this.tab = *ix; cx.notify() }))
/// ```
#[derive(IntoElement)]
pub struct IconTabs {
    id: ElementId,
    tabs: Vec<(Icon, SharedString, bool)>,
    selected: usize,
    width: Option<Pixels>,
    on_change: Option<ChangeHandler>,
}

impl IconTabs {
    pub fn new(id: impl Into<ElementId>, selected: usize) -> Self {
        Self {
            id: id.into(),
            tabs: Vec::new(),
            selected,
            width: None,
            on_change: None,
        }
    }

    /// Adds a tab; `icon` slides in beside `label` while it is active.
    pub fn tab(mut self, icon: Icon, label: impl Into<SharedString>) -> Self {
        self.tabs.push((icon, label.into(), false));
        self
    }

    /// Adds a tab whose icon stays visible when inactive (`iconPinned`).
    pub fn pinned_tab(mut self, icon: Icon, label: impl Into<SharedString>) -> Self {
        self.tabs.push((icon, label.into(), true));
        self
    }

    /// A fixed list width; the tabs then share it equally.
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(move |ix, window, cx| handler(&ix, window, cx)));
        self
    }
}

struct TabMotion {
    focus: FocusHandle,
    active: Animated,
    reveal: Animated,
    hover: Animated,
    bounds: Option<Bounds<Pixels>>,
}

struct IconTabsState {
    selected: usize,
    tabs: Vec<TabMotion>,
    list: Option<Bounds<Pixels>>,
    slide: Option<IndicatorSlide>,
}

impl IconTabsState {
    fn sync(&mut self, selected: usize, count: usize, now: Instant, cx: &mut App) {
        while self.tabs.len() < count {
            let on = self.tabs.len() == selected;
            let value = if on { 1.0 } else { 0.0 };
            self.tabs.push(TabMotion {
                focus: cx.focus_handle(),
                active: Animated::new(value, COLOR),
                reveal: Animated::new(value, ICON_REVEAL),
                hover: Animated::new(0.0, COLOR),
                bounds: None,
            });
        }
        self.tabs.truncate(count);
        self.selected = selected;
        for (ix, tab) in self.tabs.iter_mut().enumerate() {
            let target = if ix == selected { 1.0 } else { 0.0 };
            tab.active.set(target, now);
            tab.reveal.set(target, now);
        }
    }

    fn target(&self) -> Option<[f32; 2]> {
        let list = self.list?;
        let tab = self.tabs.get(self.selected)?.bounds?;
        Some([
            f32::from(tab.origin.x - list.origin.x),
            f32::from(tab.size.width),
        ])
    }
}

impl RenderOnce for IconTabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let skin = IconTabsSkin::new(&theme);
        let now = Instant::now();
        let t = seconds(now);
        let reduce_motion = cx.reduce_motion();
        let count = self.tabs.len();
        let selected = self.selected.min(count.saturating_sub(1));

        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| IconTabsState {
            selected,
            tabs: Vec::new(),
            list: None,
            slide: None,
        });
        state.update(cx, |s, cx| {
            s.sync(selected, count, now, cx);
            if let Some(target) = s.target() {
                match s.slide.as_mut() {
                    Some(slide) if reduce_motion => slide.snap(target),
                    Some(slide) => slide.slide_to(target, t),
                    None => s.slide = Some(IndicatorSlide::new(target)),
                }
            }
            if reduce_motion {
                for tab in &mut s.tabs {
                    tab.active.snap(tab.active.target());
                    tab.reveal.snap(tab.reveal.target());
                    tab.hover.snap(tab.hover.target());
                }
            }
        });
        let s = state.read(cx);
        let animating = s.slide.is_some_and(|slide| slide.is_animating(t))
            || s.slide.is_none()
            || s.tabs.iter().any(|tab| {
                tab.active.is_animating(now)
                    || tab.reveal.is_animating(now)
                    || tab.hover.is_animating(now)
            });
        if animating {
            window.request_animation_frame();
        }

        let indicator = s.slide.map(|slide| {
            let [left, width] = slide.value(t);
            div()
                .absolute()
                .top(px(metrics::LIST_PADDING))
                .bottom(px(metrics::LIST_PADDING))
                .left(px(left))
                .w(px(width))
                .rounded(px(metrics::TAB_RADIUS))
                .bg(skin.indicator)
                .shadow(shadows(&[skin.indicator_shadow]))
        });

        let id = self.id.clone();
        let focus_handles: Vec<FocusHandle> = s.tabs.iter().map(|tab| tab.focus.clone()).collect();
        let tabs = self
            .tabs
            .into_iter()
            .enumerate()
            .map(|(ix, (icon, label, pinned))| {
                let motion = &s.tabs[ix];
                let reveal = if pinned {
                    1.0
                } else {
                    motion.reveal.value(now)
                };
                let color = mix(
                    mix(skin.text, skin.text_hover, motion.hover.value(now)),
                    skin.text_active,
                    motion.active.value(now),
                );
                let scale =
                    metrics::ICON_HIDDEN_SCALE + (1.0 - metrics::ICON_HIDDEN_SCALE) * reveal;
                let measure_state = state.clone();
                let hover_state = state.clone();
                let on_change = self.on_change.clone();
                let key_change = self.on_change.clone();
                let handles = focus_handles.clone();
                let ring = skin.ring;
                div()
                    .id(ElementId::NamedChild(
                        Arc::new(id.clone()),
                        format!("tab-{ix}").into(),
                    ))
                    .track_focus(&motion.focus)
                    .tab_stop(ix == selected)
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(metrics::TAB_GAP))
                    .h_full()
                    .px(px(metrics::TAB_PADDING_X))
                    .when(self.width.is_some(), |d| d.flex_1())
                    .rounded(px(metrics::TAB_RADIUS))
                    .text_size(px(metrics::FONT_SIZE))
                    .line_height(px(metrics::LINE_HEIGHT))
                    .font_weight(FontWeight(metrics::FONT_WEIGHT))
                    .whitespace_nowrap()
                    .text_color(color)
                    .cursor_pointer()
                    .focus_visible(move |style| {
                        style.shadow(shadows(&[Shadow::drop(
                            0.0,
                            0.0,
                            metrics::FOCUS_OUTLINE,
                            ring,
                        )]))
                    })
                    .on_hover(move |hovered, _, cx| {
                        hover_state.update(cx, |s, cx| {
                            if let Some(tab) = s.tabs.get_mut(ix) {
                                tab.hover
                                    .set(if *hovered { 1.0 } else { 0.0 }, Instant::now());
                            }
                            cx.notify();
                        });
                    })
                    .on_click(move |_, window, cx| {
                        if let Some(handler) = &on_change {
                            handler(ix, window, cx);
                        }
                    })
                    .on_key_down(move |event, window, cx| {
                        let key = match event.keystroke.key.as_str() {
                            "right" => TabKey::Next,
                            "left" => TabKey::Previous,
                            "home" => TabKey::First,
                            "end" => TabKey::Last,
                            "enter" | "space" => {
                                if let Some(handler) = &key_change {
                                    handler(ix, window, cx);
                                }
                                cx.stop_propagation();
                                return;
                            }
                            _ => return,
                        };
                        if let Some(handle) = handles.get(key.apply(ix, handles.len())) {
                            window.focus(handle, cx);
                        }
                        cx.stop_propagation();
                    })
                    .child(
                        canvas(
                            move |bounds, _, cx| {
                                measure_state.update(cx, |s, _| {
                                    if let Some(tab) = s.tabs.get_mut(ix) {
                                        tab.bounds = Some(bounds);
                                    }
                                });
                            },
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .size_full(),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .justify_center()
                            .overflow_hidden()
                            .h(px(metrics::ICON_SIZE))
                            .w(px(metrics::ICON_SLOT * reveal))
                            .mr(px(metrics::ICON_TUCK * (1.0 - reveal)))
                            .opacity(reveal)
                            .child(
                                svg()
                                    .path(icon.path())
                                    .flex_none()
                                    .size(px(metrics::ICON_SIZE))
                                    .text_color(color)
                                    .with_transformation(Transformation::scale(size(scale, scale))),
                            ),
                    )
                    .child(label)
            });

        let list_state = state.clone();
        div()
            .id(self.id)
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(metrics::LIST_GAP))
            .h(px(metrics::LIST_HEIGHT))
            .p(px(metrics::LIST_PADDING))
            .rounded(px(metrics::LIST_RADIUS))
            .bg(skin.list)
            .font_family(fonts::SANS)
            .map(|d| match self.width {
                Some(width) => d.w(width),
                None => d.flex_none(),
            })
            .child(
                canvas(
                    move |bounds, window, cx| {
                        let measured = list_state.update(cx, |s, _| {
                            let fresh = s.list != Some(bounds);
                            s.list = Some(bounds);
                            fresh
                        });
                        if measured {
                            window.request_animation_frame();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .children(indicator)
            .children(tabs)
    }
}
