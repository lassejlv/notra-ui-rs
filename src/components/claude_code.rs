//! Claude Code: the terminal transcript block (header with the pixel mascot,
//! messages, tool calls, todos, thinking, spinner, prompt) and its scripted demo.

use std::time::Duration;

use super::terminal::{
    ChromePalette, FrameCycle, InlineStyle, Pace, Rich, Run, inline, stream_tokens,
};
use crate::theme::{Appearance, Color};

/// Body text: `text-[0.8125rem] leading-5`.
pub const FONT_SIZE: f32 = 13.0;
pub const LINE_HEIGHT: f32 = 20.0;
/// `gap-5` between transcript blocks, and between the prompt and its status line.
pub const GAP: f32 = 20.0;
/// The transcript's padding (`px-4 py-4` from the `sm` breakpoint up).
pub const CONTENT_PADDING_X: f32 = 16.0;
pub const CONTENT_PADDING_Y: f32 = 16.0;
/// The footer's bottom padding (`pb-3`).
pub const FOOTER_PADDING_BOTTOM: f32 = 12.0;
/// The prompt field's vertical padding (`py-1.5`) inside its 1 px rules.
pub const PROMPT_PADDING_Y: f32 = 6.0;
/// The demo terminal's height (`h-170`).
pub const DEMO_HEIGHT: f32 = 680.0;
/// The mascot's default width; its height keeps the 22 × 14 grid.
pub const LOGO_WIDTH: f32 = 76.0;
/// The focus ring of collapsible triggers (`ring-2 ring-muted/60 rounded-sm`).
pub const FOCUS_RING: f32 = 2.0;
pub const FOCUS_RADIUS: f32 = 4.0;
/// `transition-opacity` of the summary's "(click to expand)" hint.
pub const HINT_FADE: crate::motion::Tween =
    crate::motion::Tween::new(crate::motion::duration::FAST, crate::motion::ease::IN_OUT);

/// The corner glyph that leads result lines.
pub const RESULT_GLYPH: &str = "⎿";
/// The spinner's frames, forwards then back (`✻ ✽ ✶ ✳ ✢` bounce), one per 120 ms.
pub const SPINNER_FRAMES: [&str; 10] = ["·", "✢", "✳", "✶", "✻", "✽", "✻", "✶", "✳", "✢"];
pub const SPINNER: FrameCycle = FrameCycle::new(SPINNER_FRAMES.len(), 120);

/// The colours of Claude Code's terminal, from the item's `--claude-code-*` tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClaudeCodePalette {
    pub background: Color,
    pub chrome: Color,
    pub traffic: Color,
    pub title: Color,
    pub shadow: Color,
    pub ring: Color,
    pub fg: Color,
    pub strong: Color,
    pub accent: Color,
    pub muted: Color,
    pub rule: Color,
    pub link: Color,
    pub pr: Color,
    pub user_bg: Color,
    pub user_prompt: Color,
    pub success: Color,
    pub error: Color,
    pub mode_auto: Color,
    pub mode_accept_edits: Color,
    pub mode_plan: Color,
    pub mode_bypass: Color,
}

impl ClaudeCodePalette {
    pub const fn light() -> Self {
        Self {
            background: Color::rgb(0xffffff),
            chrome: Color::rgb(0xececec),
            traffic: Color::rgb(0xd4d4d4),
            title: Color::rgba(0x00000073),
            shadow: Color::rgba(0x00000026),
            ring: Color::rgba(0x0000001a),
            fg: Color::rgb(0x1f1f1f),
            strong: Color::rgb(0x000000),
            accent: Color::rgb(0xd77757),
            muted: Color::rgb(0x666666),
            rule: Color::rgb(0x999999),
            link: Color::rgb(0x5769f7),
            pr: Color::rgb(0x966c1e),
            user_bg: Color::rgb(0xf0f0f0),
            user_prompt: Color::rgb(0x999999),
            success: Color::rgb(0x2c7a39),
            error: Color::rgb(0xab2b3f),
            mode_auto: Color::rgb(0x966c1e),
            mode_accept_edits: Color::rgb(0x8700ff),
            mode_plan: Color::rgb(0x006666),
            mode_bypass: Color::rgb(0xd1245a),
        }
    }

    pub const fn dark() -> Self {
        Self {
            background: Color::rgb(0x0f0f0f),
            chrome: Color::rgb(0x1c1c1c),
            traffic: Color::rgb(0x3a3a3a),
            title: Color::rgba(0xffffff66),
            shadow: Color::rgba(0x0000004d),
            ring: Color::rgba(0xffffff1a),
            fg: Color::rgb(0xf7f7f7),
            strong: Color::rgb(0xffffff),
            accent: Color::rgb(0xe06443),
            muted: Color::rgb(0x8c8c8c),
            rule: Color::rgb(0x7b7b7b),
            link: Color::rgb(0xa4b0fc),
            pr: Color::rgb(0xd9b559),
            user_bg: Color::rgb(0x2e2e2e),
            user_prompt: Color::rgb(0x5c5c5c),
            success: Color::rgb(0x4eba65),
            error: Color::rgb(0xff6b80),
            mode_auto: Color::rgb(0xe5c07b),
            mode_accept_edits: Color::rgb(0xaf87ff),
            mode_plan: Color::rgb(0x48968c),
            mode_bypass: Color::rgb(0xfd526f),
        }
    }

    pub const fn new(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Light => Self::light(),
            Appearance::Dark => Self::dark(),
        }
    }

    pub const fn chrome(&self) -> ChromePalette {
        ChromePalette {
            background: self.background,
            title_bar: self.chrome,
            traffic: self.traffic,
            title: self.title,
            shadow: self.shadow,
            ring: self.ring,
        }
    }

    /// ``code`` in the link colour, `**strong**` bold in the strong colour.
    pub const fn inline(&self, text: Color) -> InlineStyle {
        InlineStyle {
            text,
            code: self.link,
            strong: Some(self.strong),
        }
    }

    /// The empty prompt's block cursor: white painted with `mix-blend-difference`
    /// over the background, and over the placeholder's first character.
    pub fn cursor(&self) -> (Color, Color) {
        (difference(self.background), difference(self.muted))
    }
}

fn difference(c: Color) -> Color {
    Color {
        r: 1.0 - c.r,
        g: 1.0 - c.g,
        b: 1.0 - c.b,
        a: 1.0,
    }
}

/// Styled runs of `text` in the terminal's inline grammar.
pub fn rich(text: &str, color: Color, palette: &ClaudeCodePalette) -> Rich {
    Rich(inline(text, palette.inline(color)))
}

/// The permission mode on the prompt's status line; Shift+Tab cycles it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ClaudeCodeMode {
    #[default]
    Manual,
    AcceptEdits,
    Plan,
    Auto,
    Bypass,
}

impl ClaudeCodeMode {
    pub const ORDER: [ClaudeCodeMode; 5] = [
        ClaudeCodeMode::Manual,
        ClaudeCodeMode::AcceptEdits,
        ClaudeCodeMode::Plan,
        ClaudeCodeMode::Auto,
        ClaudeCodeMode::Bypass,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ClaudeCodeMode::Manual => "? for shortcuts",
            ClaudeCodeMode::AcceptEdits => "⏵⏵ accept edits on",
            ClaudeCodeMode::Plan => "⏸ plan mode on",
            ClaudeCodeMode::Auto => "⏵⏵ auto mode on",
            ClaudeCodeMode::Bypass => "⏵⏵ bypass permissions on",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ClaudeCodeMode::Manual => "manual",
            ClaudeCodeMode::AcceptEdits => "accept-edits",
            ClaudeCodeMode::Plan => "plan",
            ClaudeCodeMode::Auto => "auto",
            ClaudeCodeMode::Bypass => "bypass",
        }
    }

    /// Whether the status line shows the `shift+tab` hint.
    pub fn cycles(self) -> bool {
        self != ClaudeCodeMode::Manual
    }

    pub fn color(self, p: &ClaudeCodePalette) -> Color {
        match self {
            ClaudeCodeMode::Manual => p.muted,
            ClaudeCodeMode::AcceptEdits => p.mode_accept_edits,
            ClaudeCodeMode::Plan => p.mode_plan,
            ClaudeCodeMode::Auto => p.mode_auto,
            ClaudeCodeMode::Bypass => p.mode_bypass,
        }
    }

    pub fn next(self) -> Self {
        let i = Self::ORDER.iter().position(|&m| m == self).unwrap_or(0);
        Self::ORDER[(i + 1) % Self::ORDER.len()]
    }
}

/// The reasoning effort on the right of the status line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClaudeCodeEffort {
    Low,
    Medium,
    High,
    Xhigh,
    Max,
    Ultracode,
}

impl ClaudeCodeEffort {
    pub const ORDER: [ClaudeCodeEffort; 6] = [
        ClaudeCodeEffort::Low,
        ClaudeCodeEffort::Medium,
        ClaudeCodeEffort::High,
        ClaudeCodeEffort::Xhigh,
        ClaudeCodeEffort::Max,
        ClaudeCodeEffort::Ultracode,
    ];

    pub fn glyph(self) -> &'static str {
        match self {
            ClaudeCodeEffort::Low => "○",
            ClaudeCodeEffort::Medium => "◑",
            ClaudeCodeEffort::High => "◕",
            ClaudeCodeEffort::Xhigh => "◉",
            ClaudeCodeEffort::Max => "●",
            ClaudeCodeEffort::Ultracode => "✦",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ClaudeCodeEffort::Low => "low",
            ClaudeCodeEffort::Medium => "medium",
            ClaudeCodeEffort::High => "high",
            ClaudeCodeEffort::Xhigh => "xhigh",
            ClaudeCodeEffort::Max => "max",
            ClaudeCodeEffort::Ultracode => "ultracode",
        }
    }

    pub fn color(self, p: &ClaudeCodePalette) -> Color {
        match self {
            ClaudeCodeEffort::Ultracode => p.accent,
            _ => p.muted,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TodoStatus {
    Done,
    Active,
    Todo,
}

impl TodoStatus {
    pub fn glyph(self) -> &'static str {
        match self {
            TodoStatus::Done => "☒",
            TodoStatus::Active | TodoStatus::Todo => "☐",
        }
    }

    /// The screen-reader prefix.
    pub fn label(self) -> &'static str {
        match self {
            TodoStatus::Done => "Done",
            TodoStatus::Active => "In progress",
            TodoStatus::Todo => "To do",
        }
    }

    pub fn glyph_color(self, p: &ClaudeCodePalette) -> Color {
        match self {
            TodoStatus::Done => p.muted,
            _ => p.fg,
        }
    }

    /// The item's label in its status style.
    pub fn text(self, label: &str, p: &ClaudeCodePalette) -> Run {
        match self {
            TodoStatus::Active => Run::new(label, p.strong).bold(),
            TodoStatus::Done => Run::new(label, p.muted).strike(),
            TodoStatus::Todo => Run::new(label, p.fg),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ToolStatus {
    #[default]
    Success,
    Error,
    /// Running: the bullet pulses.
    Pending,
}

impl ToolStatus {
    pub fn bullet(self, p: &ClaudeCodePalette) -> Color {
        match self {
            ToolStatus::Success => p.success,
            ToolStatus::Error => p.error,
            ToolStatus::Pending => p.muted,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ToolStatus::Success => "Done",
            ToolStatus::Error => "Failed",
            ToolStatus::Pending => "Running",
        }
    }
}

/// A tool call: `● Tool(arg)` over `⎿ result`, with optional detail behind a click.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub tool: String,
    pub arg: Option<String>,
    pub result: String,
    pub detail: Option<String>,
    pub status: ToolStatus,
}

impl ToolCall {
    pub fn new(id: &str, tool: &str, arg: Option<&str>, result: &str) -> Self {
        Self {
            id: id.into(),
            tool: tool.into(),
            arg: arg.map(Into::into),
            result: result.into(),
            detail: None,
            status: ToolStatus::Success,
        }
    }

    pub fn status(mut self, status: ToolStatus) -> Self {
        self.status = status;
        self
    }

    pub fn detail(mut self, detail: &str) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// The headline: the tool in bold with its argument, or the tool alone as plain text.
    pub fn headline(&self, p: &ClaudeCodePalette) -> Rich {
        match &self.arg {
            Some(arg) => Rich::new()
                .push(Run::new(self.tool.as_str(), p.strong).bold())
                .push(Run::new(format!("({arg})"), p.fg)),
            None => Rich::plain(self.tool.as_str(), p.fg),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Todo {
    pub label: String,
    pub status: TodoStatus,
}

impl Todo {
    pub fn new(label: &str, status: TodoStatus) -> Self {
        Self {
            label: label.into(),
            status,
        }
    }
}

/// `✻ Cogitated for 7s · done 9:41 AM`.
#[derive(Clone, Debug, PartialEq)]
pub struct TurnSummary {
    pub verb: String,
    pub duration: String,
    pub done_at: Option<String>,
}

impl TurnSummary {
    pub fn new(verb: &str, duration: &str, done_at: &str) -> Self {
        Self {
            verb: verb.into(),
            duration: duration.into(),
            done_at: Some(done_at.into()),
        }
    }

    pub fn text(&self) -> String {
        match &self.done_at {
            Some(at) => format!("{} for {} · done {at}", self.verb, self.duration),
            None => format!("{} for {}", self.verb, self.duration),
        }
    }
}

/// The spinner line: `✻ Sketching… (14s · ↓ 688 tokens · thinking)` and a tip.
#[derive(Clone, Debug, PartialEq)]
pub struct Spinner {
    pub verb: String,
    pub elapsed: Option<String>,
    pub tokens: Option<u32>,
    pub details: Vec<String>,
    pub tip: Option<String>,
}

impl Spinner {
    pub fn new(verb: &str) -> Self {
        Self {
            verb: verb.into(),
            elapsed: None,
            tokens: None,
            details: Vec::new(),
            tip: None,
        }
    }

    /// The parenthesised fields after the verb.
    pub fn meta(&self) -> Option<String> {
        let fields: Vec<String> = self
            .elapsed
            .iter()
            .cloned()
            .chain(
                self.tokens
                    .map(|t| format!("↓ {} tokens", group_thousands(t))),
            )
            .chain(self.details.iter().cloned())
            .collect();
        (!fields.is_empty()).then(|| fields.join(" · "))
    }
}

fn group_thousands(n: u32) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The header printed at launch.
#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    pub version: &'static str,
    pub model: &'static str,
    pub org: &'static str,
    pub cwd: &'static str,
    /// Notice lines; support ``code`` and `**bold**`.
    pub whats_new: &'static [&'static str],
    /// Dim `⎿` lines under the notices.
    pub tips: &'static [&'static str],
}

impl Header {
    /// `model · org`.
    pub fn plan(&self) -> String {
        [self.model, self.org]
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

/// The mascot on a 22 × 14 grid as filled rectangles `(x, y, w, h)`: the body with
/// the eyes left open.
pub const MASCOT_GRID: (f32, f32) = (22.0, 14.0);
pub const MASCOT: [(f32, f32, f32, f32); 13] = [
    (2.0, 0.0, 18.0, 2.0),
    (2.0, 2.0, 4.0, 3.0),
    (8.0, 2.0, 6.0, 3.0),
    (16.0, 2.0, 4.0, 3.0),
    (2.0, 5.0, 18.0, 5.0),
    (0.0, 4.0, 6.0, 1.0),
    (8.0, 4.0, 6.0, 1.0),
    (16.0, 4.0, 6.0, 1.0),
    (0.0, 5.0, 22.0, 2.0),
    (3.0, 10.0, 2.0, 4.0),
    (7.0, 10.0, 2.0, 4.0),
    (13.0, 10.0, 2.0, 4.0),
    (17.0, 10.0, 2.0, 4.0),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TurnStatus {
    Working,
    Streaming,
    Done,
    Interrupted,
}

/// One exchange of the transcript.
#[derive(Clone, Debug, PartialEq)]
pub struct Turn {
    pub id: String,
    pub prompt: String,
    pub todos: Vec<Todo>,
    pub commands: Vec<ToolCall>,
    pub answer: String,
    pub summary: Option<TurnSummary>,
    pub status: TurnStatus,
}

impl Turn {
    /// Finished turns whose every command is `Bash` fold under `Ran N shell commands`.
    pub fn folds_shell(&self) -> bool {
        self.status == TurnStatus::Done
            && !self.commands.is_empty()
            && self.commands.iter().all(|c| c.tool == "Bash")
    }
}

/// A scripted answer the demo plays back when a prompt is sent.
#[derive(Clone, Debug, PartialEq)]
pub struct Reply {
    pub commands: &'static [(&'static str, &'static str, &'static str, &'static str)],
    pub answer: &'static str,
    pub spinner_verb: &'static str,
    pub summary_verb: &'static str,
}

/// The playback's paces (`CLAUDE_CODE_CHAT_TIMING`).
pub mod timing {
    pub const THINK_MS: u64 = 1200;
    pub const TOOL_MS: u64 = 900;
    pub const WORD_MS: u64 = 28;
    pub const REDUCED_MAX_MS: u64 = 60;
    /// Characters per streamed token in the spinner's count.
    pub const CHARS_PER_TOKEN: usize = 4;
}

/// The paces under the user's motion preference.
pub fn pace(reduced: bool) -> Pace {
    Pace {
        reduced,
        reduced_max: Duration::from_millis(timing::REDUCED_MAX_MS),
    }
}

struct Active {
    id: String,
    prompt: String,
    reply: usize,
    started: Duration,
    pace: Pace,
}

/// The moments a scripted reply changes the transcript, relative to its start.
struct Timeline {
    think: Duration,
    tool: Duration,
    word: Duration,
    tokens: Vec<&'static str>,
    commands: usize,
}

impl Timeline {
    fn new(reply: &Reply, pace: Pace) -> Self {
        Self {
            think: pace.wait(timing::THINK_MS),
            tool: pace.wait(timing::TOOL_MS),
            word: pace.wait(timing::WORD_MS),
            tokens: if pace.reduced {
                vec![reply.answer]
            } else {
                stream_tokens(reply.answer)
            },
            commands: reply.commands.len(),
        }
    }

    fn stream_start(&self) -> Duration {
        self.think + self.tool * self.commands as u32
    }

    fn done(&self) -> Duration {
        self.stream_start() + self.word * self.tokens.len() as u32
    }

    /// Every moment something changes, in order.
    fn steps(&self) -> impl Iterator<Item = Duration> + '_ {
        let commands = (0..=self.commands).map(|i| self.think + self.tool * i as u32);
        let words = (0..=self.tokens.len()).map(|k| self.stream_start() + self.word * k as u32);
        commands.chain(words)
    }

    /// How many tokens have streamed in by `elapsed`.
    fn streamed(&self, elapsed: Duration) -> usize {
        if elapsed < self.stream_start() {
            return 0;
        }
        let since = elapsed - self.stream_start();
        let steps = if self.word.is_zero() {
            self.tokens.len()
        } else {
            (since.as_nanos() / self.word.as_nanos()) as usize + 1
        };
        steps.min(self.tokens.len())
    }
}

/// What the demo shows at one moment.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatView {
    pub turns: Vec<Turn>,
    /// The spinner while a turn runs.
    pub spinner: Option<Spinner>,
}

/// The demo's chat (`useClaudeCodeChat`) as a pure function of time: settled turns
/// plus the running one, computed from how long ago it was sent.
pub struct Chat {
    turns: Vec<Turn>,
    replies: &'static [Reply],
    active: Option<Active>,
    sent: usize,
}

impl Chat {
    pub fn new(history: Vec<Turn>, replies: &'static [Reply]) -> Self {
        Self {
            turns: history,
            replies,
            active: None,
            sent: 0,
        }
    }

    /// The demo's starting session.
    pub fn demo() -> Self {
        Self::new(session::history(), session::REPLIES)
    }

    pub fn is_busy(&self) -> bool {
        self.active.is_some()
    }

    /// Sends `prompt` at `now` and starts the next scripted reply. Ignored while busy.
    pub fn send(&mut self, prompt: &str, now: Duration, pace: Pace) {
        if self.active.is_some() || self.replies.is_empty() {
            return;
        }
        let reply = self.sent % self.replies.len();
        self.sent += 1;
        self.active = Some(Active {
            id: format!("turn-{}", self.sent),
            prompt: prompt.into(),
            reply,
            started: now,
            pace,
        });
    }

    /// Escape: keeps what has printed, drops running tool calls and marks the turn.
    pub fn stop(&mut self, now: Duration) {
        if let Some(mut turn) = self.active_turn(now) {
            turn.commands.retain(|c| c.status != ToolStatus::Pending);
            turn.status = TurnStatus::Interrupted;
            self.turns.push(turn);
            self.active = None;
        }
    }

    /// Folds a finished reply into the transcript; true when it did.
    pub fn settle(&mut self, now: Duration) -> bool {
        let Some(active) = &self.active else {
            return false;
        };
        let reply = &self.replies[active.reply];
        let timeline = Timeline::new(reply, active.pace);
        let elapsed = now.saturating_sub(active.started);
        if elapsed < timeline.done() {
            return false;
        }
        let seconds = (timeline.done().as_secs_f32().round() as u64).max(1);
        if let Some(mut turn) = self.active_turn(now) {
            turn.status = TurnStatus::Done;
            turn.summary = Some(TurnSummary {
                verb: reply.summary_verb.into(),
                duration: format!("{seconds}s"),
                done_at: Some(session::clock(active.started + timeline.done())),
            });
            self.turns.push(turn);
        }
        self.active = None;
        true
    }

    /// The next moment the view changes, on the same clock as `now`: a tool call or
    /// token landing, or the elapsed counter ticking.
    pub fn next_change(&self, now: Duration) -> Option<Duration> {
        let active = self.active.as_ref()?;
        let timeline = Timeline::new(&self.replies[active.reply], active.pace);
        let elapsed = now.saturating_sub(active.started);
        let tick = Duration::from_secs(elapsed.as_secs() + 1);
        let step = timeline.steps().find(|&t| t > elapsed).unwrap_or(tick);
        Some(active.started + step.min(tick))
    }

    /// The transcript at `now`.
    pub fn view(&self, now: Duration) -> ChatView {
        let mut turns = self.turns.clone();
        let spinner = self.active.as_ref().and_then(|active| {
            let turn = self.active_turn(now)?;
            let reply = &self.replies[active.reply];
            let timeline = Timeline::new(reply, active.pace);
            let elapsed = now.saturating_sub(active.started);
            let streamed: usize = timeline.tokens[..timeline.streamed(elapsed)]
                .iter()
                .map(|t| t.len())
                .sum();
            let tokens = (streamed as f32 / timing::CHARS_PER_TOKEN as f32).round() as u32;
            let mut details = Vec::new();
            if turn.status == TurnStatus::Working {
                details.push("thinking".to_owned());
            }
            details.push("esc to interrupt".to_owned());
            turns.push(turn);
            Some(Spinner {
                verb: reply.spinner_verb.into(),
                elapsed: Some(format!("{}s", elapsed.as_secs())),
                tokens: (tokens > 0).then_some(tokens),
                details,
                tip: Some(session::SPINNER_TIP.into()),
            })
        });
        ChatView { turns, spinner }
    }

    fn active_turn(&self, now: Duration) -> Option<Turn> {
        let active = self.active.as_ref()?;
        let reply = &self.replies[active.reply];
        let timeline = Timeline::new(reply, active.pace);
        let elapsed = now.saturating_sub(active.started);
        let commands = reply
            .commands
            .iter()
            .enumerate()
            .take_while(|(i, _)| elapsed >= timeline.think + timeline.tool * *i as u32)
            .map(|(i, &(id, tool, arg, result))| {
                let done = elapsed >= timeline.think + timeline.tool * (i as u32 + 1);
                ToolCall::new(&format!("{}-{id}", active.id), tool, Some(arg), result).status(
                    if done {
                        ToolStatus::Success
                    } else {
                        ToolStatus::Pending
                    },
                )
            })
            .collect();
        let streaming = elapsed >= timeline.stream_start();
        Some(Turn {
            id: active.id.clone(),
            prompt: active.prompt.clone(),
            todos: Vec::new(),
            commands,
            answer: timeline.tokens[..timeline.streamed(elapsed)].concat(),
            summary: None,
            status: if streaming {
                TurnStatus::Streaming
            } else {
                TurnStatus::Working
            },
        })
    }
}

/// The demo session (`CLAUDE_CODE_SESSION` and `CLAUDE_CODE_REPLIES`).
pub mod session {
    use std::time::Duration;

    use super::*;

    pub const TITLE: &str = "claude — ~/acme/web";
    pub const PLACEHOLDER: &str = "Try \"write a LinkedIn post about the release\"";
    pub const PULL_REQUEST: u32 = 1317;
    pub const SPINNER_TIP: &str = "Tip: Run `/resume` to pick up an earlier conversation";

    pub const HEADER: Header = Header {
        version: "v2.1.285",
        model: "Opus 5.5 (1M context)",
        org: "Claude Max",
        cwd: "~/acme/web",
        whats_new: &["Get to finished work sooner with Opus 5.5. Switch anytime with `/model`."],
        tips: &[
            "SessionStart:startup hook succeeded",
            "notra MCP server connected · 24 tools",
        ],
    };

    pub const FIRST_ANSWER: &str = "Since **v1.8.0** there are 14 merged PRs on `main`, split across three areas.\n\n`apps/dashboard` got the new **GEO scan** view with per-engine citations, `packages/api` added `POST /v1/posts/:id/schedule`, and `apps/web` moved the changelog to `/changelog/[slug]`. Two PRs are fixes from review.\n\nWant me to draft the changelog for v1.9 from these?";

    pub const SECOND_ANSWER: &str = "Drafted **v1.9** in your brand voice and queued it in Notra for Thursday 9:00 AM. The post groups the 14 PRs into features and fixes and links each one.\n\nOpen it with `notra posts open changelog-v1-9` to review before it goes out.";

    /// The two settled turns the demo opens with.
    pub fn history() -> Vec<Turn> {
        vec![
            Turn {
                id: "what-changed".into(),
                prompt: "what changed since the last release?".into(),
                todos: Vec::new(),
                commands: vec![ToolCall::new(
                    "git-log",
                    "Bash",
                    Some("git log --oneline v1.8.0..HEAD"),
                    "14 commits",
                )],
                answer: FIRST_ANSWER.into(),
                summary: Some(TurnSummary::new("Cogitated", "7s", "9:41 AM")),
                status: TurnStatus::Done,
            },
            Turn {
                id: "draft-changelog".into(),
                prompt: "draft the changelog for v1.9 and queue it in notra".into(),
                todos: vec![
                    Todo::new("Group the 14 merged PRs by area", TodoStatus::Done),
                    Todo::new("Draft the changelog in brand voice", TodoStatus::Done),
                    Todo::new("Queue the post in notra", TodoStatus::Done),
                ],
                commands: vec![
                    ToolCall::new(
                        "brand-voice",
                        "notra - get_brand_voice (MCP)",
                        Some("type: changelog, voice: acme"),
                        "Loaded brand voice and the last 3 changelogs",
                    ),
                    ToolCall::new(
                        "queue-post",
                        "notra - create_post (MCP)",
                        Some("title: v1.9, schedule: thu 09:00"),
                        "Queued changelog-v1-9",
                    ),
                ],
                answer: SECOND_ANSWER.into(),
                summary: Some(TurnSummary::new("Brewed", "38s", "9:43 AM")),
                status: TurnStatus::Done,
            },
        ]
    }

    pub const REPLIES: &[Reply] = &[
        Reply {
            commands: &[
                (
                    "get-post",
                    "notra - get_post (MCP)",
                    "id: changelog-v1-9",
                    "Read changelog-v1-9 (412 words)",
                ),
                (
                    "create-linkedin",
                    "notra - create_post (MCP)",
                    "type: linkedin, source: changelog-v1-9",
                    "Created linkedin-v1-9 as a draft",
                ),
            ],
            answer: "Here's a LinkedIn post for the release:\n\n**Acme v1.9 is out.** Track which AI engines cite you with the new GEO scan view, schedule posts straight from the API, and find every release at `/changelog`.\n\nSaved it as a draft next to the changelog. Want a shorter version for X?",
            spinner_verb: "Crafting",
            summary_verb: "Crafted",
        },
        Reply {
            commands: &[
                (
                    "grep-retry",
                    "Search",
                    "pattern: \"retry\", path: packages/api",
                    "Found 4 files",
                ),
                (
                    "read-publish",
                    "Read",
                    "packages/api/src/jobs/publish.ts",
                    "Read 86 lines",
                ),
            ],
            answer: "The scheduler fix is in `packages/api/src/jobs/publish.ts`. Failed publish jobs now retry three times with backoff before they surface in the dashboard.\n\nThe tests in `publish.test.ts` cover both the retry and the final failure.",
            spinner_verb: "Spelunking",
            summary_verb: "Worked",
        },
        Reply {
            commands: &[("run-tests", "Bash", "bun run test", "212 passed")],
            answer: "All 212 tests pass and the build is clean. Nothing else is blocking **v1.9**.",
            spinner_verb: "Verifying",
            summary_verb: "Cogitated",
        },
    ];

    /// The session's wall clock: the demo opens at 9:43 AM, when its last turn ended,
    /// and `at` counts from then.
    pub fn clock(at: Duration) -> String {
        let minutes = 9 * 60 + 43 + at.as_secs() / 60;
        let (hour, minute) = ((minutes / 60) % 24, minutes % 60);
        let (twelve, half) = match hour {
            0 => (12, "AM"),
            1..=11 => (hour, "AM"),
            12 => (12, "PM"),
            _ => (hour - 12, "PM"),
        };
        format!("{twelve}:{minute:02} {half}")
    }
}

/// Content of the per-component examples on the docs pages.
pub mod examples {
    use super::*;

    pub const THINKING_BODY: &str = "The user wants the changes since v1.8.0. I'll read the git log first, then group the merged PRs by app so the answer maps to the monorepo layout.";

    pub fn tool_calls() -> Vec<ToolCall> {
        vec![
            ToolCall::new(
                "git-log",
                "Bash",
                Some("git log --oneline v1.8.0..HEAD"),
                "14 commits",
            ),
            ToolCall::new("read", "Read", Some("CHANGELOG.md"), "Read 212 lines")
                .detail("## v1.8.0\n- GEO scan view\n- Scheduled posts API\n- Changelog pages"),
            ToolCall::new(
                "draft",
                "Drafting the changelog in notra",
                None,
                "$ notra posts create --type changelog --from-prs v1.8.0..HEAD",
            )
            .status(ToolStatus::Pending),
            ToolCall::new(
                "publish",
                "notra",
                Some("publish_post"),
                "Error: post is still in review",
            )
            .status(ToolStatus::Error),
        ]
    }

    pub fn shell_calls() -> Vec<ToolCall> {
        vec![
            ToolCall::new(
                "status",
                "Bash",
                Some("git status --short"),
                "2 files changed",
            ),
            ToolCall::new("types", "Bash", Some("bun run check-types"), "0 errors"),
            ToolCall::new(
                "pr",
                "Bash",
                Some("gh pr view --json number"),
                "{\"number\": 1317}",
            ),
        ]
    }

    pub fn todos() -> Vec<Todo> {
        vec![
            Todo::new("Group the 14 merged PRs by area", TodoStatus::Done),
            Todo::new("Draft the changelog in brand voice", TodoStatus::Active),
            Todo::new("Queue the post in notra", TodoStatus::Todo),
        ]
    }

    pub fn spinner() -> Spinner {
        Spinner {
            elapsed: Some("14s".into()),
            tokens: Some(688),
            details: vec!["thinking".into()],
            tip: Some(session::SPINNER_TIP.into()),
            ..Spinner::new("Sketching")
        }
    }

    pub fn short_spinner() -> Spinner {
        Spinner {
            elapsed: Some("2s".into()),
            ..Spinner::new("Frosting")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn modes_cycle_back_to_manual() {
        let mut mode = ClaudeCodeMode::Bypass;
        mode = mode.next();
        assert_eq!(mode, ClaudeCodeMode::Manual);
        assert!(!mode.cycles());
    }

    #[test]
    fn spinner_meta_groups_tokens() {
        let s = Spinner {
            tokens: Some(1688),
            elapsed: Some("3s".into()),
            ..Spinner::new("Sketching")
        };
        assert_eq!(s.meta().as_deref(), Some("3s · ↓ 1,688 tokens"));
        assert_eq!(Spinner::new("x").meta(), None);
    }

    #[test]
    fn playback_runs_tools_then_streams_then_settles() {
        let mut chat = Chat::demo();
        let start = MS(5_000);
        chat.send("write a post", start, pace(false));
        let view = chat.view(start + MS(500));
        let turn = view.turns.last().cloned();
        assert_eq!(turn.as_ref().map(|t| t.status), Some(TurnStatus::Working));
        assert!(turn.is_some_and(|t| t.commands.is_empty()));
        assert!(view.spinner.is_some_and(|s| s.details[0] == "thinking"));

        let at = start + MS(1_300);
        let turn = chat.view(at).turns.pop();
        assert_eq!(
            turn.map(|t| t.commands.iter().map(|c| c.status).collect::<Vec<_>>()),
            Some(vec![ToolStatus::Pending])
        );

        let streaming = start + MS(1_200 + 1_800 + 28 * 3);
        let view = chat.view(streaming);
        assert!(
            view.turns.last().is_some_and(
                |t| t.status == TurnStatus::Streaming && t.answer.starts_with("Here's")
            )
        );
        assert!(view.spinner.is_some_and(|s| s.tokens.is_some()));

        assert!(!chat.settle(streaming));
        assert!(chat.settle(start + MS(60_000)));
        let done = chat.view(start + MS(60_000)).turns.pop();
        assert!(done.as_ref().is_some_and(
            |t| t.status == TurnStatus::Done && t.answer == session::REPLIES[0].answer
        ));
        assert!(
            done.and_then(|t| t.summary)
                .is_some_and(|s| s.verb == "Crafted")
        );
        assert!(!chat.is_busy());
    }

    #[test]
    fn stop_drops_running_tools() {
        let mut chat = Chat::demo();
        chat.send("go", Duration::ZERO, pace(false));
        chat.stop(MS(1_500));
        let turn = chat.view(MS(1_500)).turns.pop();
        assert!(turn.is_some_and(|t| t.status == TurnStatus::Interrupted && t.commands.is_empty()));
        assert!(!chat.is_busy());
    }

    #[test]
    fn reduced_motion_lands_the_answer_at_once() {
        let mut chat = Chat::demo();
        chat.send("go", Duration::ZERO, pace(true));
        let done = MS(60 + 60 * 2 + 60);
        assert!(chat.settle(done));
    }

    #[test]
    fn next_change_is_the_next_step_or_tick() {
        let mut chat = Chat::demo();
        assert_eq!(chat.next_change(Duration::ZERO), None);
        chat.send("go", Duration::ZERO, pace(false));
        assert_eq!(chat.next_change(MS(100)), Some(MS(1_000)));
        assert_eq!(chat.next_change(MS(1_100)), Some(MS(1_200)));
    }

    #[test]
    fn clock_counts_from_the_session() {
        assert_eq!(session::clock(Duration::ZERO), "9:43 AM");
        assert_eq!(session::clock(Duration::from_secs(60 * 20)), "10:03 AM");
    }

    #[test]
    fn shell_turns_fold() {
        let history = session::history();
        assert!(history[0].folds_shell());
        assert!(!history[1].folds_shell());
    }
}
