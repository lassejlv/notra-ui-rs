//! Pieces the terminal blocks (Claude Code, Codex, OpenCode) share: styled runs of
//! monospace text, the macOS window chrome, frame cyclers for glyph spinners and
//! the pacing rules of their scripted playback.

use std::ops::Range;
use std::time::Duration;

use crate::motion::{Loop, ease, keyframes};
use crate::theme::{Color, Shadow};

/// The advance of one monospace cell (`1ch`) in em: Geist Mono and the site's
/// `ui-monospace` stack both draw every glyph 0.6 em wide.
pub const CH_EM: f32 = 0.6;

/// `n` monospace cells at `font_size`, CSS `calc(n * 1ch)`.
pub fn ch(n: f32, font_size: f32) -> f32 {
    n * CH_EM * font_size
}

/// A run of text in one style.
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub text: String,
    pub color: Color,
    pub bold: bool,
    pub italic: bool,
    pub strike: bool,
    pub underline: bool,
}

impl Run {
    pub fn new(text: impl Into<String>, color: Color) -> Self {
        Self {
            text: text.into(),
            color,
            bold: false,
            italic: false,
            strike: false,
            underline: false,
        }
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn strike(mut self) -> Self {
        self.strike = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
}

/// A paragraph of styled runs, laid out and wrapped as one text.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rich(pub Vec<Run>);

impl Rich {
    pub fn new() -> Self {
        Self::default()
    }

    /// One run of plain text.
    pub fn plain(text: impl Into<String>, color: Color) -> Self {
        Self(vec![Run::new(text, color)])
    }

    pub fn push(mut self, run: Run) -> Self {
        self.0.push(run);
        self
    }

    pub fn extend(mut self, runs: impl IntoIterator<Item = Run>) -> Self {
        self.0.extend(runs);
        self
    }

    /// The whole string.
    pub fn text(&self) -> String {
        self.0.iter().map(|r| r.text.as_str()).collect()
    }

    /// Each run with its byte range in [`Rich::text`].
    pub fn ranges(&self) -> impl Iterator<Item = (Range<usize>, &Run)> {
        let mut at = 0;
        self.0.iter().map(move |run| {
            let range = at..at + run.text.len();
            at = range.end;
            (range, run)
        })
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|r| r.text.is_empty())
    }
}

/// How a terminal's inline grammar styles its marks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineStyle {
    pub text: Color,
    /// ``code``: its colour.
    pub code: Color,
    /// `**strong**`: its colour, or `None` to leave `**` as text.
    pub strong: Option<Color>,
}

/// Splits `text` on ``code`` and `**strong**` marks as Claude Code's and Codex's
/// inline renderers do; a mark needs at least one character inside it.
pub fn inline(text: &str, style: InlineStyle) -> Vec<Run> {
    let mut runs = Vec::new();
    let mut plain = String::new();
    let mut rest = text;
    while !rest.is_empty() {
        let code = rest
            .strip_prefix('`')
            .and_then(|r| r.find('`'))
            .filter(|&len| len > 0)
            .map(|len| (1, len, 1, style.code, false));
        let strong = style.strong.and_then(|color| {
            let inner = rest.strip_prefix("**")?;
            let len = inner.find('*').filter(|&len| len > 0)?;
            inner[len..]
                .starts_with("**")
                .then_some((2, len, 2, color, true))
        });
        match code.or(strong) {
            Some((open, len, close, color, bold)) => {
                if !plain.is_empty() {
                    runs.push(Run::new(std::mem::take(&mut plain), style.text));
                }
                let run = Run::new(&rest[open..open + len], color);
                runs.push(if bold { run.bold() } else { run });
                rest = &rest[open + len + close..];
            }
            None => {
                let mut chars = rest.chars();
                plain.extend(chars.next());
                rest = chars.as_str();
            }
        }
    }
    if !plain.is_empty() {
        runs.push(Run::new(plain, style.text));
    }
    runs
}

/// Splits a reply into the tokens the playback hooks stream one at a time, as
/// `text.split(/(\s+)/)` does: words and the whitespace between them.
pub fn stream_tokens(text: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    if text.starts_with(char::is_whitespace) {
        tokens.push("");
    }
    let mut start = 0;
    let mut space = None;
    for (i, c) in text.char_indices() {
        let is_space = c.is_whitespace();
        if space.is_some_and(|s| s != is_space) {
            tokens.push(&text[start..i]);
            start = i;
        }
        space = Some(is_space);
    }
    if start < text.len() {
        tokens.push(&text[start..]);
    }
    tokens
}

/// The paces of a scripted turn under the user's motion preference: with reduced
/// motion every wait is capped and a reply lands in one piece.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pace {
    pub reduced: bool,
    /// The cap on every wait under reduced motion (`reducedMaxMs`).
    pub reduced_max: Duration,
}

impl Pace {
    pub fn wait(&self, ms: u64) -> Duration {
        let wait = Duration::from_millis(ms);
        if self.reduced {
            wait.min(self.reduced_max)
        } else {
            wait
        }
    }
}

/// A glyph spinner: `count` frames, one every `interval`, looping.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameCycle {
    pub count: usize,
    pub interval: Duration,
}

impl FrameCycle {
    pub const fn new(count: usize, interval_ms: u64) -> Self {
        Self {
            count,
            interval: Duration::from_millis(interval_ms),
        }
    }

    /// The frame shown `elapsed` after the spinner started.
    pub fn frame(&self, elapsed: Duration) -> usize {
        if self.count == 0 || self.interval.is_zero() {
            return 0;
        }
        (elapsed.as_millis() / self.interval.as_millis()) as usize % self.count
    }

    /// One loop.
    pub fn period(&self) -> Duration {
        self.interval * self.count as u32
    }

    /// When frame `index` shows within a loop, as fractions of [`FrameCycle::period`].
    pub fn window(&self, index: usize) -> (f32, f32) {
        let n = self.count.max(1) as f32;
        (index as f32 / n, (index + 1) as f32 / n)
    }
}

/// Tailwind's `animate-pulse`: opacity 1 → 0.5 → 1 every 2 s, on the pulse curve.
pub const PULSE: Loop = Loop::new(Duration::from_secs(2), ease::PULSE);

/// The opacity of an `animate-pulse` element `elapsed` after it mounted.
pub fn pulse_opacity(elapsed: Duration) -> f32 {
    keyframes(
        &[(0.0, 1.0), (0.5, 0.5), (1.0, 1.0)],
        PULSE.easing,
        PULSE.phase(elapsed),
    )
}

/// The colours of a terminal window's chrome.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChromePalette {
    pub background: Color,
    pub title_bar: Color,
    pub traffic: Color,
    pub title: Color,
    pub shadow: Color,
    pub ring: Color,
}

/// The macOS-style window Claude Code and Codex draw: a 32 px title bar with three
/// grey traffic lights and a centred title, a 1 px ring and a soft drop shadow.
pub mod chrome {
    use super::*;

    pub const RADIUS: f32 = 12.0;
    pub const BAR_HEIGHT: f32 = 32.0;
    pub const BAR_PADDING_X: f32 = 12.0;
    pub const LIGHT_SIZE: f32 = 12.0;
    pub const LIGHT_GAP: f32 = 8.0;
    pub const TITLE_SIZE: f32 = 12.0;
    pub const TITLE_LINE: f32 = 16.0;

    /// `ring-1` plus `shadow-[0_1.5rem_3.5rem_-1rem_var(--shadow)]`.
    pub fn shadows(palette: &ChromePalette) -> [Shadow; 2] {
        [
            Shadow::drop(0.0, 0.0, 1.0, palette.ring),
            Shadow::drop(24.0, 56.0, -16.0, palette.shadow),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STYLE: InlineStyle = InlineStyle {
        text: Color::BLACK,
        code: Color::WHITE,
        strong: Some(Color::TRANSPARENT),
    };

    #[test]
    fn inline_marks_code_and_strong() {
        let runs = inline("Since **v1.8.0** on `main`.", STYLE);
        let texts: Vec<_> = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(texts, ["Since ", "v1.8.0", " on ", "main", "."]);
        assert!(runs[1].bold);
        assert_eq!(runs[3].color, Color::WHITE);
    }

    #[test]
    fn inline_leaves_unclosed_marks() {
        let runs = inline("a `b and ** c", STYLE);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "a `b and ** c");
    }

    #[test]
    fn stream_tokens_match_js_split() {
        assert_eq!(stream_tokens("a  b\n\nc"), ["a", "  ", "b", "\n\n", "c"]);
        assert_eq!(stream_tokens(" a"), ["", " ", "a"]);
    }

    #[test]
    fn frame_cycle_wraps() {
        let cycle = FrameCycle::new(10, 80);
        assert_eq!(cycle.frame(Duration::from_millis(79)), 0);
        assert_eq!(cycle.frame(Duration::from_millis(820)), 0);
        assert_eq!(cycle.frame(Duration::from_millis(165)), 2);
    }

    #[test]
    fn rich_ranges_cover_the_text() {
        let rich = Rich::plain("ab", Color::BLACK).push(Run::new("cd", Color::WHITE));
        let ranges: Vec<_> = rich.ranges().map(|(r, _)| r).collect();
        assert_eq!(ranges, [0..2, 2..4]);
        assert_eq!(rich.text(), "abcd");
    }
}
