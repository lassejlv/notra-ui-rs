//! Shared Rune pieces of the terminal blocks: styled monospace text, the window
//! chrome and glyph spinners.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use ::rune::tok::{Family, TextStyle};
use ::rune::{El, ScrollSpec, Span, col, el, rich, row, text};
use ::rune_motion::{Beat, CubicBezier, Cycle, Easing, LoopSpec, Sequence};

use super::ToRune;
use crate::components::terminal::{ChromePalette, FrameCycle, Rich, chrome};

/// Geist Mono through Rune's monospace family (`AppSpec::families.1`), which keeps
/// code free of ligatures (`--` stays two hyphens, as in a terminal).
pub fn code_font(size: f32, weight: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size, weight)
        .family(Family::Mono)
        .lh(line_height)
}

/// [`Rich`] runs as one wrapping Rune text in `style`.
pub fn rich_text(rich_runs: &Rich, style: TextStyle) -> El {
    let mut spans = Vec::new();
    let mut strikes = Vec::new();
    let mut base = None;
    for (range, run) in rich_runs.ranges().filter(|(r, _)| !r.is_empty()) {
        base.get_or_insert(run.color);
        let mut span = Span::new(range.start, range.end).color(run.color.rune());
        if run.bold {
            span = span.weight(700.0);
        }
        if run.italic {
            span = span.italic();
        }
        if run.underline {
            span = span.underline(run.color.rune());
        }
        if run.strike {
            strikes.push((range.start, range.end, Some(run.color.rune())));
        }
        spans.push(span);
    }
    let mut e = rich(rich_runs.text(), spans).font(style).strikes(strikes);
    if let Some(color) = base {
        e = e.color(color.rune());
    }
    e
}

/// Which collapsibles are open, by id; the app owns it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Disclosures {
    open: HashSet<String>,
}

impl Disclosures {
    /// Every collapsible in `open` starts open.
    pub fn with_open(open: impl IntoIterator<Item = String>) -> Self {
        Self {
            open: open.into_iter().collect(),
        }
    }

    pub fn is_open(&self, id: &str) -> bool {
        self.open.contains(id)
    }

    pub fn toggle(&mut self, id: &str) {
        if !self.open.remove(id) {
            self.open.insert(id.to_owned());
        }
    }
}

/// A Base UI collapsible trigger: a focusable button toggling `id` in the
/// [`Disclosures`] that `lens` reaches, with a `ring`-coloured focus ring.
pub fn disclosure_trigger<S: 'static>(
    id: &str,
    open: bool,
    ring: crate::theme::Color,
    ring_width: f32,
    radius: f32,
    lens: impl Fn(&mut S) -> &mut Disclosures + Copy + 'static,
) -> El {
    let key = id.to_owned();
    row()
        .key(format!("{id}-trigger"))
        .role("button")
        .expanded(open)
        .focusable(true)
        .radius(radius)
        .focus_ring(ring_width, 0.0, ring.rune())
        .cursor(::rune::Cursor::Pointer)
        .hover_group()
        .on_press(move |s: &mut S| lens(s).toggle(&key))
}

/// One line of the terminals' monospace text.
pub fn line(text_content: impl Into<String>, style: TextStyle, color: crate::theme::Color) -> El {
    text(text_content).font(style).color(color.rune())
}

/// The window Claude Code and Codex draw: title bar, scrolling transcript and a
/// footer pinned under it. The body's padding and gaps belong to the caller.
pub struct TerminalWindow {
    key: String,
    palette: ChromePalette,
    title: Option<String>,
    height: Option<f32>,
    body: Option<El>,
    footer: Option<El>,
    follow: bool,
}

impl TerminalWindow {
    /// `key` names the window; its transcript scrolls under `{key}-body`.
    pub fn new(key: impl Into<String>, palette: ChromePalette) -> Self {
        Self {
            key: key.into(),
            palette,
            title: None,
            height: None,
            body: None,
            footer: None,
            follow: false,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// A fixed height; the transcript scrolls inside it.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn body(mut self, body: El) -> Self {
        self.body = Some(body);
        self
    }

    pub fn footer(mut self, footer: El) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Keeps the transcript scrolled to its end while new output arrives.
    pub fn follow(mut self, follow: bool) -> Self {
        self.follow = follow;
        self
    }

    pub fn build(self) -> El {
        let p = self.palette;
        let lights = (0..3).fold(
            row()
                .pin(Some(chrome::BAR_PADDING_X), Some(0.0), None, Some(0.0))
                .center_y()
                .gap(chrome::LIGHT_GAP),
            |r, i| {
                r.child(
                    el().key(format!("{}-light-{i}", self.key))
                        .size(chrome::LIGHT_SIZE, chrome::LIGHT_SIZE)
                        .radius(chrome::LIGHT_SIZE / 2.0)
                        .fill(p.traffic.rune()),
                )
            },
        );
        let mut bar = row()
            .key(format!("{}-bar", self.key))
            .h(chrome::BAR_HEIGHT)
            .w_full()
            .px(chrome::BAR_PADDING_X)
            .center_y()
            .justify_center()
            .fill(p.title_bar.rune())
            .child(lights);
        if let Some(title) = self.title {
            bar = bar.child(
                text(title)
                    .key(format!("{}-title", self.key))
                    .font(code_font(chrome::TITLE_SIZE, 400.0, chrome::TITLE_LINE))
                    .color(p.title.rune())
                    .single_line()
                    .truncate(),
            );
        }
        let mut body = col()
            .key(format!("{}-body", self.key))
            .w_full()
            .children(self.body);
        if self.height.is_some() {
            body = body.grow(1.0).min_h(0.0).shrink().scroll(ScrollSpec {
                stick_bottom: self.follow,
            });
        }
        let mut window = col()
            .key(self.key)
            .w_full()
            .radius(chrome::RADIUS)
            .fill(p.background.rune())
            .clip_rounded(chrome::RADIUS)
            .child(bar)
            .child(body)
            .children(self.footer);
        for s in chrome::shadows(&p) {
            window = window.box_shadow(s.rune());
        }
        match self.height {
            Some(h) => window.h(h),
            None => window,
        }
    }
}

const PULSE_BEATS: &[Beat] = &[
    Beat::new(0.0).opacity(1.0).easing(PULSE_EASE),
    Beat::new(1.0).opacity(0.5).easing(PULSE_EASE),
    Beat::new(2.0).opacity(1.0).easing(PULSE_EASE),
];
const PULSE_EASE: Easing = Easing::CubicBezier(CubicBezier::new(0.4, 0.0, 0.6, 1.0));

/// Tailwind's `animate-pulse` as a Rune loop: opacity 1 → 0.5 → 1 every 2 s.
pub fn pulse() -> LoopSpec {
    LoopSpec::sequence(Sequence::new(PULSE_BEATS))
}

/// A glyph spinner: every frame drawn in place, each shown during its slot of the
/// loop by a stepped wall-clock opacity cycle, so it runs without view updates.
pub fn frame_glyph(
    key: &str,
    frames: &'static [&'static str],
    cycle: FrameCycle,
    style: TextStyle,
    color: crate::theme::Color,
    paused: bool,
) -> El {
    let period = cycle.period().as_secs_f32();
    frames
        .iter()
        .enumerate()
        .fold(el().key(key), |stack, (i, glyph)| {
            let mut layer = text(*glyph)
                .key(format!("{key}-frame-{i}"))
                .font(style)
                .color(color.rune());
            if paused {
                layer = layer.opacity(if i == 0 { 1.0 } else { 0.0 });
            } else {
                layer = layer.cycle(Cycle {
                    period,
                    phase: 0.0,
                    stops: frame_stops(cycle, i),
                    still: if i == 0 { 1.0 } else { 0.0 },
                    fps: 1.0 / cycle.interval.as_secs_f32(),
                });
            }
            if i > 0 {
                layer = layer.pin(Some(0.0), Some(0.0), None, None);
            }
            stack.child(layer)
        })
}

/// Opacity stops that show frame `index` during its slot and hide it elsewhere.
fn frame_stops(cycle: FrameCycle, index: usize) -> &'static [(f32, f32)] {
    type Stops = &'static [(f32, f32)];
    static CACHE: OnceLock<Mutex<HashMap<(usize, usize), Stops>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let mut cache = cache.lock().unwrap_or_else(|e| e.into_inner());
    cache.entry((cycle.count, index)).or_insert_with(|| {
        let (start, end) = cycle.window(index);
        let edge = 1e-4;
        let stops = if start <= 0.0 {
            vec![(0.0, 1.0), (end - edge, 1.0), (end, 0.0), (1.0, 0.0)]
        } else if end >= 1.0 {
            vec![(0.0, 0.0), (start - edge, 0.0), (start, 1.0), (1.0, 1.0)]
        } else {
            vec![
                (0.0, 0.0),
                (start - edge, 0.0),
                (start, 1.0),
                (end - edge, 1.0),
                (end, 0.0),
                (1.0, 0.0),
            ]
        };
        Box::leak(stops.into_boxed_slice())
    })
}
