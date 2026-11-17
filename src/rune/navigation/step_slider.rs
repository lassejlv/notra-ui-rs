use std::f32::consts::TAU;

use ::rune::{Drag, El, Key, Phase, Shape, Tx, canvas, el, row, text};
use ::rune_motion::{Spring, SpringPair, Transition};

use crate::components::navigation::step_slider::{
    GLIDE, LABEL_COLOR, STOP_UNDER_THUMB, SWEEP, SWEEP_BAND, SliderKey, StepSliderSkin, Sweeps,
    Track, band_left, dot_centers, metrics, nearest_step_index, step_ratio,
};
use crate::rune::{ToRune, sans, tween};
use crate::theme::{Color, Theme};

/// What the step slider remembers between frames: its value, the control's
/// laid-out width and the sweep bands.
#[derive(Clone, Debug, PartialEq)]
pub struct StepSliderState {
    pub value: f64,
    /// The last value a drag, key or label click settled on.
    pub committed: f64,
    width: f32,
    sweeps: Sweeps,
}

impl StepSliderState {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            committed: value,
            width: 0.0,
            sweeps: Sweeps::default(),
        }
    }

    fn set(&mut self, value: f64, tx: &mut Tx) {
        if value != self.value {
            self.value = value;
            self.sweeps.step(tx.clock());
        }
    }
}

type Lens<S> = fn(&mut S) -> &mut StepSliderState;

/// A slider that snaps to fixed values, with a dotted fill swept on every step:
/// `StepSlider::new("prompts", &STEPS, &s.prompts).build(&theme, |s: &mut State| &mut s.prompts)`.
pub struct StepSlider<'a> {
    key: String,
    steps: Vec<f64>,
    state: &'a StepSliderState,
    labels: Vec<String>,
    major: Option<Vec<f64>>,
    hide_labels: bool,
    disabled: bool,
    label: String,
    value_text: Option<Box<dyn Fn(f64) -> String>>,
}

impl<'a> StepSlider<'a> {
    /// `steps` in ascending order; the state's value snaps to the nearest one.
    pub fn new(key: impl Into<String>, steps: &[f64], state: &'a StepSliderState) -> Self {
        Self {
            key: key.into(),
            steps: steps.to_vec(),
            state,
            labels: steps.iter().map(|s| format!("{s}")).collect(),
            major: None,
            hide_labels: false,
            disabled: false,
            label: String::new(),
            value_text: None,
        }
    }

    /// The label under each stop (`formatLabel`).
    pub fn format(mut self, format: impl Fn(f64) -> String) -> Self {
        self.labels = self.steps.iter().map(|s| format(*s)).collect();
        self
    }

    /// Labels that stay when the slider is narrower than 512 points.
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

    /// The thumb's accessible name.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Screen reader text for a value (`getValueText`).
    pub fn value_text(mut self, f: impl Fn(f64) -> String + 'static) -> Self {
        self.value_text = Some(Box::new(f));
        self
    }

    pub fn build<S: 'static>(self, theme: &Theme, lens: Lens<S>) -> El {
        let skin = StepSliderSkin::new(theme);
        let count = self.steps.len();
        let index = nearest_step_index(&self.steps, self.state.value);
        let ratio = step_ratio(index, count);
        let track = Track::new(self.state.width);
        let fill = track.fill_width(ratio);
        let glide = Spring::new(GLIDE.duration.as_secs_f32(), 0.0);
        let key = self.key;
        let interactive = !self.disabled && count > 1;
        let steps = self.steps.clone();

        let grid_width =
            (track.width / metrics::DOT_TILE).ceil() * metrics::DOT_TILE + metrics::DOT_TILE * 2.0;
        let dots = |radius: f32, color: Color| {
            dot_centers(grid_width, metrics::THUMB).fold(Shape::new(), |shape, [x, y]| {
                let points: Vec<[f32; 2]> = (0..8)
                    .map(|k| {
                        let a = k as f32 * TAU / 8.0;
                        [x + radius * a.cos(), y + radius * a.sin()]
                    })
                    .collect();
                shape.fill(points, color.rune())
            })
        };
        let (base_in, base_out) = metrics::DOT_RADII;
        let (sweep_in, sweep_out) = metrics::SWEEP_DOT_RADII;
        let grid_left = |width: f32| width / 2.0 - grid_width / 2.0;

        let travel = band_left(fill, 1.0) - band_left(fill, 0.0);
        let sweep_spring =
            SpringPair::new(Spring::new(SWEEP.duration.as_secs_f32(), 0.0), Spring::NONE);
        let band = fill * SWEEP_BAND;
        let end = band_left(fill, 1.0);
        let bands = self
            .state
            .sweeps
            .serials()
            .into_iter()
            .enumerate()
            .filter(|(_, n)| *n > 0)
            .map(|(b, n)| {
                el().key(format!("{key}/sweep-{b}-{n}"))
                    .absolute(end, 0.0)
                    .size(band, metrics::THUMB)
                    .clip_fade_linear(band / 2.0, 0.0, band / 2.0, 0.0)
                    .transition(Transition::offset(-travel, 0.0).springs(sweep_spring))
                    .child(
                        canvas(dots((sweep_in + sweep_out) / 2.0, skin.sweep_dot))
                            .key(format!("{key}/sweep-{b}-{n}/dots"))
                            .absolute(grid_left(fill) - end, 0.0)
                            .size(grid_width, metrics::THUMB)
                            .transition(Transition::offset(travel, 0.0).springs(sweep_spring)),
                    )
            });

        let indicator = el()
            .key(format!("{key}/indicator"))
            .absolute(0.0, 0.0)
            .size(fill, metrics::THUMB)
            .motion(glide)
            .radius(metrics::THUMB / 2.0)
            .gradient(skin.fill.0.rune(), skin.fill.1.rune())
            .box_shadow(skin.fill_highlight.rune())
            .clip_rounded(metrics::THUMB / 2.0)
            .child(
                canvas(dots((base_in + base_out) / 2.0, skin.dot))
                    .key(format!("{key}/dots"))
                    .absolute(grid_left(fill), 0.0)
                    .size(grid_width, metrics::THUMB)
                    .motion(glide),
            )
            .children(bands);

        let stop_color = tween(crate::components::navigation::step_slider::STOP_COLOR);
        let stops = (0..count).filter_map(|i| {
            let at = step_ratio(i, count);
            if (at - ratio).abs() < STOP_UNDER_THUMB {
                return None;
            }
            let color = if at < ratio {
                skin.stop_filled
            } else {
                skin.stop
            };
            Some(
                el().key(format!("{key}/stop-{i}"))
                    .absolute(
                        track.stop_center(at) - metrics::STOP / 2.0,
                        (metrics::THUMB - metrics::STOP) / 2.0,
                    )
                    .size(metrics::STOP, metrics::STOP)
                    .radius(metrics::STOP / 2.0)
                    .fill(color.rune())
                    .opacity_motion(stop_color),
            )
        });

        let value_text = self.value_text.as_ref().map_or_else(
            || format!("{}", self.steps[index]),
            |f| f(self.steps[index]),
        );
        let key_steps = steps.clone();
        let mut thumb = el()
            .key(format!("{key}/thumb"))
            .absolute(track.thumb_left(ratio), 0.0)
            .size(metrics::THUMB, metrics::THUMB)
            .motion(glide)
            .radius(metrics::THUMB / 2.0)
            .gradient(skin.thumb.0.rune(), skin.thumb.1.rune())
            .box_shadow(skin.thumb_shadows[0].rune())
            .box_shadow(skin.thumb_shadows[1].rune())
            .role("slider")
            .label(self.label)
            .range_value(index as f64, 0.0, count.saturating_sub(1) as f64)
            .range_step(1.0)
            .a11y_value(value_text)
            .focusable(interactive)
            .focus_ring(metrics::FOCUS_RING, 0.0, skin.focus_ring.rune());
        if interactive {
            thumb = thumb.on_key(move |s: &mut S, ev, tx: &mut Tx| {
                let large = ev.mods.shift;
                let action = match ev.key {
                    Key::Right | Key::Up if large => SliderKey::IncreaseLarge,
                    Key::Left | Key::Down if large => SliderKey::DecreaseLarge,
                    Key::Right | Key::Up => SliderKey::Increase,
                    Key::Left | Key::Down => SliderKey::Decrease,
                    Key::PageUp => SliderKey::IncreaseLarge,
                    Key::PageDown => SliderKey::DecreaseLarge,
                    Key::Home => SliderKey::Home,
                    Key::End => SliderKey::End,
                    _ => return false,
                };
                let state = lens(s);
                let at = nearest_step_index(&key_steps, state.value);
                let next = key_steps[action.apply(at, key_steps.len())];
                state.set(next, tx);
                state.committed = next;
                true
            });
        }

        let drag_steps = steps.clone();
        let mut control = el()
            .key(format!("{key}/control"))
            .h(metrics::THUMB)
            .w_full()
            .on_layout(move |s: &mut S, rect| lens(s).width = rect[2])
            .child(indicator)
            .children(stops)
            .child(thumb);
        if interactive {
            control = control.cursor(::rune::Cursor::Pointer).on_drag(
                move |s: &mut S, drag: &mut Drag, tx: &mut Tx| {
                    let at = Track::new(drag.w).index_at(drag.x, drag_steps.len());
                    let state = lens(s);
                    state.set(drag_steps[at], tx);
                    if drag.phase == Phase::Up {
                        state.committed = drag_steps[at];
                    }
                },
            );
        } else if self.disabled {
            control = control.opacity(0.5);
        }

        let pill = row()
            .key(key.clone())
            .h(metrics::TRACK_HEIGHT)
            .w_full()
            .padding(metrics::TRACK_PADDING)
            .radius(metrics::TRACK_HEIGHT / 2.0)
            .fill(skin.track.rune())
            .box_shadow(skin.track_shadow.rune())
            .child(control);

        let narrow = track.width + 2.0 * metrics::TRACK_PADDING < metrics::MAJOR_LABELS_BELOW;
        let label_motion = tween(LABEL_COLOR);
        let labels = (!self.hide_labels).then(|| {
            let items = self
                .labels
                .into_iter()
                .enumerate()
                .filter_map(|(i, label)| {
                    let step = steps[i];
                    if narrow && !self.major.as_ref().is_none_or(|m| m.contains(&step)) {
                        return None;
                    }
                    let current = i == index;
                    let center = track.stop_center(step_ratio(i, count));
                    let mut t = text(label)
                        .font(
                            sans(
                                metrics::LABEL_FONT_SIZE,
                                if current { 500.0 } else { 400.0 },
                                metrics::LABEL_LINE_HEIGHT,
                            )
                            .tracking(metrics::LABEL_TRACKING)
                            .features("'tnum' 1"),
                        )
                        .color(
                            if current {
                                skin.label_active
                            } else {
                                skin.label
                            }
                            .rune(),
                        )
                        .single_line();
                    if !current {
                        t = t.group_color(skin.label_active.rune());
                    }
                    let mut cell = row()
                        .key(format!("{key}/label-{i}"))
                        .absolute(center - 40.0, 0.0)
                        .w(80.0)
                        .justify_center()
                        .child(
                            row()
                                .key(format!("{key}/label-{i}/hit"))
                                .hover_group()
                                .hover_motion(label_motion, label_motion)
                                .child(t),
                        );
                    if interactive && !current {
                        cell = cell.cursor(::rune::Cursor::Pointer).on_press_tx(
                            move |s: &mut S, tx: &mut Tx| {
                                let state = lens(s);
                                state.set(step, tx);
                                state.committed = step;
                            },
                        );
                    }
                    Some(cell)
                });
            el().key(format!("{key}/labels"))
                .h(metrics::LABEL_HEIGHT)
                .mx(metrics::LABEL_INSET)
                .children(items)
        });

        ::rune::col()
            .key(format!("{key}/root"))
            .gap(metrics::LABEL_GAP)
            .w_full()
            .child(pill)
            .children(labels)
    }
}
