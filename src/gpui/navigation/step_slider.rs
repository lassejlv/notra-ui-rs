use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use ::gpui::{
    App, BorderStyle, Bounds, Corners, ElementId, FocusHandle, FontWeight, IntoElement,
    MouseButton, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, RenderOnce, SharedString,
    Styled, Window, canvas, div, point, prelude::*, px, quad, size, transparent_black,
};

use super::{mix, seconds};
use crate::components::navigation::step_slider::{
    FOCUS, GLIDE, LABEL_COLOR, STOP_COLOR, STOP_UNDER_THUMB, SWEEP_BAND, SliderKey, StepSliderSkin,
    Sweeps, Track, band_left, band_mask, dot_centers, metrics, nearest_step_index, step_ratio,
    sweep_progress,
};
use crate::fonts;
use crate::gpui::{ActiveTheme, gradient, shadows};
use crate::motion::Animated;
use crate::theme::{Color, Shadow};

type ValueHandler = Rc<dyn Fn(f64, &mut Window, &mut App)>;

/// A slider that snaps to fixed values, with a dotted fill swept on every step.
///
/// ```ignore
/// StepSlider::new("prompts", &[5.0, 10.0, 25.0], self.prompts)
///     .format(|v| format!("{v}"))
///     .on_change(cx.listener(|this, v, _, cx| { this.prompts = *v; cx.notify() }))
/// ```
#[derive(IntoElement)]
pub struct StepSlider {
    id: ElementId,
    steps: Vec<f64>,
    value: f64,
    labels: Vec<SharedString>,
    major: Option<Vec<f64>>,
    hide_labels: bool,
    disabled: bool,
    on_change: Option<ValueHandler>,
    on_commit: Option<ValueHandler>,
}

impl StepSlider {
    /// `steps` in ascending order; `value` snaps to the nearest one.
    pub fn new(id: impl Into<ElementId>, steps: &[f64], value: f64) -> Self {
        Self {
            id: id.into(),
            steps: steps.to_vec(),
            value,
            labels: steps
                .iter()
                .map(|s| SharedString::from(format!("{s}")))
                .collect(),
            major: None,
            hide_labels: false,
            disabled: false,
            on_change: None,
            on_commit: None,
        }
    }

    /// The label under each stop (`formatLabel`).
    pub fn format(mut self, format: impl Fn(f64) -> String) -> Self {
        self.labels = self.steps.iter().map(|s| format(*s).into()).collect();
        self
    }

    /// Labels that stay when the slider is narrower than 512 px.
    pub fn major_steps(mut self, major: &[f64]) -> Self {
        self.major = Some(major.to_vec());
        self
    }

    pub fn hide_labels(mut self, hide: bool) -> Self {
        self.hide_labels = hide;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Fires with each new step value while dragging, on keys and on label clicks.
    pub fn on_change(mut self, handler: impl Fn(&f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(move |v, window, cx| handler(&v, window, cx)));
        self
    }

    /// Fires when a drag, key press or label click ends.
    pub fn on_commit(mut self, handler: impl Fn(&f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_commit = Some(Rc::new(move |v, window, cx| handler(&v, window, cx)));
        self
    }
}

struct SliderState {
    focus: FocusHandle,
    index: Option<usize>,
    glide: Animated,
    stops: Vec<Animated>,
    labels: Vec<Animated>,
    ring: Animated,
    sweeps: Sweeps,
    control: Option<Bounds<Pixels>>,
    dragging: bool,
}

impl SliderState {
    fn sync(&mut self, index: usize, count: usize, now: Instant, reduce: bool) {
        let ratio = step_ratio(index, count);
        if self.stops.len() != count {
            self.stops = (0..count)
                .map(|i| Animated::new(filled(step_ratio(i, count), ratio), STOP_COLOR))
                .collect();
            self.labels = (0..count)
                .map(|_| Animated::new(0.0, LABEL_COLOR))
                .collect();
        }
        if self.index != Some(index) {
            if self.index.is_some() && !reduce {
                self.sweeps.step(seconds(now));
                self.glide.set(ratio, now);
            } else {
                self.glide.snap(ratio);
            }
            self.index = Some(index);
        }
        for (i, stop) in self.stops.iter_mut().enumerate() {
            stop.set(filled(step_ratio(i, count), ratio), now);
            if reduce {
                stop.snap(stop.target());
            }
        }
    }
}

fn filled(stop: f32, ratio: f32) -> f32 {
    if stop < ratio { 1.0 } else { 0.0 }
}

fn paint_dots(
    bounds: Bounds<Pixels>,
    width: f32,
    bands: [Option<f64>; 2],
    skin: StepSliderSkin,
    window: &mut Window,
) {
    let height = f32::from(bounds.size.height);
    let band = width * SWEEP_BAND;
    let lefts: Vec<f32> = bands
        .iter()
        .flatten()
        .map(|elapsed| band_left(width, sweep_progress(*elapsed)))
        .collect();
    let dot = |c: [f32; 2], radius: f32, color: Color| {
        quad(
            Bounds::new(
                point(
                    bounds.origin.x + px(c[0] - radius),
                    bounds.origin.y + px(c[1] - radius),
                ),
                size(px(radius * 2.0), px(radius * 2.0)),
            ),
            Corners::all(px(radius)),
            color,
            px(0.0),
            transparent_black(),
            BorderStyle::default(),
        )
    };
    let (base_in, base_out) = metrics::DOT_RADII;
    let (sweep_in, sweep_out) = metrics::SWEEP_DOT_RADII;
    for c in dot_centers(width, height) {
        window.paint_quad(dot(c, (base_in + base_out) / 2.0, skin.dot));
        let mask = lefts
            .iter()
            .map(|left| band_mask(c[0] - left, band))
            .fold(0.0_f32, f32::max);
        if mask > 0.0 {
            window.paint_quad(dot(
                c,
                (sweep_in + sweep_out) / 2.0,
                skin.sweep_dot.fade(mask),
            ));
        }
    }
}

impl RenderOnce for StepSlider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let skin = StepSliderSkin::new(&theme);
        let now = Instant::now();
        let reduce = cx.reduce_motion();
        let count = self.steps.len();
        let index = nearest_step_index(&self.steps, self.value);
        let ratio = step_ratio(index, count);

        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| SliderState {
            focus: cx.focus_handle(),
            index: None,
            glide: Animated::new(ratio, GLIDE),
            stops: Vec::new(),
            labels: Vec::new(),
            ring: Animated::new(0.0, FOCUS),
            sweeps: Sweeps::default(),
            control: None,
            dragging: false,
        });
        let focused = state.read(cx).focus.is_focused(window);
        state.update(cx, |s, _| {
            s.sync(index, count, now, reduce);
            s.ring.set(if focused { 1.0 } else { 0.0 }, now);
        });
        let s = state.read(cx);
        let t = seconds(now);
        let bands = s.sweeps.running(t);
        let animating = s.glide.is_animating(now)
            || s.ring.is_animating(now)
            || s.control.is_none()
            || (!reduce && s.sweeps.is_animating(t))
            || s.stops.iter().chain(&s.labels).any(|a| a.is_animating(now));
        if animating {
            window.request_animation_frame();
        }

        let track = Track::new(s.control.map_or(0.0, |b| f32::from(b.size.width)));
        let shown = s.glide.value(now);
        let fill = track.fill_width(shown);
        let focus = s.focus.clone();
        let interactive = !self.disabled && count > 1;
        let child_id = |name: String| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());

        let select = {
            let steps = self.steps.clone();
            let on_change = self.on_change.clone();
            Rc::new(move |next: usize, window: &mut Window, cx: &mut App| {
                if next != index
                    && let (Some(value), Some(handler)) = (steps.get(next), &on_change)
                {
                    handler(*value, window, cx);
                }
            })
        };
        let commit = {
            let steps = self.steps.clone();
            let on_commit = self.on_commit.clone();
            Rc::new(move |next: usize, window: &mut Window, cx: &mut App| {
                if let (Some(value), Some(handler)) = (steps.get(next), &on_commit) {
                    handler(*value, window, cx);
                }
            })
        };

        let dots = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| paint_dots(bounds, fill, bands, skin, window),
        )
        .absolute()
        .inset_0();
        let indicator = div()
            .absolute()
            .top_0()
            .left_0()
            .h(px(metrics::THUMB))
            .w(px(fill))
            .rounded_full()
            .overflow_hidden()
            .bg(gradient(skin.fill.0, skin.fill.1))
            .shadow(shadows(&[skin.fill_highlight]))
            .when(!reduce, |d| d.child(dots));

        let stops = (0..count).filter_map(|i| {
            let at = step_ratio(i, count);
            if (at - ratio).abs() < STOP_UNDER_THUMB {
                return None;
            }
            let color = mix(skin.stop, skin.stop_filled, s.stops[i].value(now));
            Some(
                div()
                    .absolute()
                    .top(px((metrics::THUMB - metrics::STOP) / 2.0))
                    .left(px(track.stop_center(at) - metrics::STOP / 2.0))
                    .size(px(metrics::STOP))
                    .rounded_full()
                    .bg(color),
            )
        });

        let ring = s.ring.value(now);
        let thumb = div()
            .absolute()
            .top_0()
            .left(px(track.thumb_left(shown)))
            .size(px(metrics::THUMB))
            .rounded_full()
            .bg(gradient(skin.thumb.0, skin.thumb.1))
            .shadow(shadows(&[
                skin.thumb_shadows[0],
                skin.thumb_shadows[1],
                Shadow::drop(0.0, 0.0, metrics::FOCUS_RING, skin.focus_ring.fade(ring)),
            ]));

        let pointer_state = state.clone();
        let drag_state = state.clone();
        let (drag_select, drag_commit) = (select.clone(), commit.clone());
        let press_select = select.clone();
        let steps_len = count;
        let control = div()
            .id(child_id("control".into()))
            .relative()
            .h(px(metrics::THUMB))
            .w_full()
            .when(interactive, |d| d.cursor_pointer())
            .when(self.disabled, |d| d.opacity(0.5))
            .child(
                canvas(
                    move |bounds, _, cx| {
                        pointer_state.update(cx, |s, _| s.control = Some(bounds));
                    },
                    move |bounds, _, window, _| {
                        let state = drag_state.clone();
                        let select = drag_select.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, _, window, cx| {
                            if !state.read(cx).dragging {
                                return;
                            }
                            let track = Track::new(f32::from(bounds.size.width));
                            let x = f32::from(event.position.x - bounds.origin.x);
                            select(track.index_at(x, steps_len), window, cx);
                        });
                        let state = drag_state.clone();
                        let commit = drag_commit.clone();
                        window.on_mouse_event(move |event: &MouseUpEvent, _, window, cx| {
                            if !state.read(cx).dragging {
                                return;
                            }
                            state.update(cx, |s, _| s.dragging = false);
                            let track = Track::new(f32::from(bounds.size.width));
                            let x = f32::from(event.position.x - bounds.origin.x);
                            commit(track.index_at(x, steps_len), window, cx);
                        });
                    },
                )
                .absolute()
                .size_full(),
            )
            .child(indicator)
            .children(stops)
            .child(thumb)
            .when(interactive, |d| {
                let press_state = state.clone();
                let press_focus = focus.clone();
                d.on_mouse_down(MouseButton::Left, move |event, window, cx| {
                    window.focus(&press_focus, cx);
                    let Some(bounds) = press_state.read(cx).control else {
                        return;
                    };
                    press_state.update(cx, |s, _| s.dragging = true);
                    let track = Track::new(f32::from(bounds.size.width));
                    let x = f32::from(event.position.x - bounds.origin.x);
                    press_select(track.index_at(x, steps_len), window, cx);
                })
            });

        let (key_select, key_commit) = (select.clone(), commit.clone());
        let pill = div()
            .id(self.id.clone())
            .track_focus(&focus)
            .tab_stop(interactive)
            .h(px(metrics::TRACK_HEIGHT))
            .w_full()
            .p(px(metrics::TRACK_PADDING))
            .rounded_full()
            .bg(skin.track)
            .shadow(shadows(&[skin.track_shadow]))
            .on_key_down(move |event, window, cx| {
                let large = event.keystroke.modifiers.shift;
                let key = match event.keystroke.key.as_str() {
                    "right" | "up" if large => SliderKey::IncreaseLarge,
                    "left" | "down" if large => SliderKey::DecreaseLarge,
                    "right" | "up" => SliderKey::Increase,
                    "left" | "down" => SliderKey::Decrease,
                    "pageup" => SliderKey::IncreaseLarge,
                    "pagedown" => SliderKey::DecreaseLarge,
                    "home" => SliderKey::Home,
                    "end" => SliderKey::End,
                    _ => return,
                };
                let next = key.apply(index, steps_len);
                key_select(next, window, cx);
                key_commit(next, window, cx);
                cx.stop_propagation();
            })
            .child(control);

        let narrow = track.width + 2.0 * metrics::TRACK_PADDING < metrics::MAJOR_LABELS_BELOW;
        let labels = (!self.hide_labels).then(|| {
            let items = self
                .labels
                .into_iter()
                .enumerate()
                .filter_map(|(i, label)| {
                    let step = self.steps[i];
                    let major = self.major.as_ref().is_none_or(|m| m.contains(&step));
                    if narrow && !major {
                        return None;
                    }
                    let current = i == index;
                    let color = if current {
                        skin.label_active
                    } else {
                        mix(skin.label, skin.label_active, s.labels[i].value(now))
                    };
                    let center = track.stop_center(step_ratio(i, count)) - metrics::LABEL_INSET;
                    let (hover_state, select, commit) =
                        (state.clone(), select.clone(), commit.clone());
                    Some(
                        div()
                            .absolute()
                            .top_0()
                            .left(px(center - 40.0))
                            .w(px(80.0))
                            .flex()
                            .justify_center()
                            .child(
                                div()
                                    .id(child_id(format!("label-{i}")))
                                    .text_color(color)
                                    .when(current, |d| d.font_weight(FontWeight(500.0)))
                                    .when(interactive, |d| {
                                        d.cursor_pointer()
                                            .on_hover(move |hovered, _, cx| {
                                                hover_state.update(cx, |s, cx| {
                                                    if let Some(a) = s.labels.get_mut(i) {
                                                        a.set(
                                                            if *hovered { 1.0 } else { 0.0 },
                                                            Instant::now(),
                                                        );
                                                    }
                                                    cx.notify();
                                                });
                                            })
                                            .on_click(move |_, window, cx| {
                                                if i != index {
                                                    select(i, window, cx);
                                                    commit(i, window, cx);
                                                }
                                            })
                                    })
                                    .child(label),
                            ),
                    )
                });
            div()
                .relative()
                .h(px(metrics::LABEL_HEIGHT))
                .mx(px(metrics::LABEL_INSET))
                .text_size(px(metrics::LABEL_FONT_SIZE))
                .line_height(px(metrics::LABEL_LINE_HEIGHT))
                .children(items)
        });

        div()
            .flex()
            .flex_col()
            .gap(px(metrics::LABEL_GAP))
            .w_full()
            .font_family(fonts::SANS)
            .child(pill)
            .children(labels)
    }
}
