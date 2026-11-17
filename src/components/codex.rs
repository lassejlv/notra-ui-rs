//! Codex: the Codex CLI session block (header, messages, reasoning, Ran and
//! Explored cells, the working line, markdown and the composer) and its scripted demo.

use std::time::Duration;

use super::terminal::{ChromePalette, InlineStyle, Pace, Rich, Run, inline, stream_tokens};
use crate::motion::{Loop, ease, keyframes};
use crate::theme::{Appearance, Color};

/// Body text: `text-[0.8125rem] leading-[1.3]`.
pub const FONT_SIZE: f32 = 13.0;
pub const LINE_HEIGHT: f32 = FONT_SIZE * 1.3;
/// `1.3em`: the gap between cells and the transcript's vertical padding.
pub const GAP: f32 = LINE_HEIGHT;
/// `0.65em` (table cell padding, the footer's bottom padding).
pub const HALF_GAP: f32 = LINE_HEIGHT / 2.0;
/// `mt-[0.5em]` between the composer band and its status lines.
pub const STATUS_GAP: f32 = FONT_SIZE * 0.5;
/// The demo terminal's height (`h-170`).
pub const DEMO_HEIGHT: f32 = 680.0;
/// Output lines an exec shows before `+ N lines`.
pub const PREVIEW_LINES: usize = 3;
/// The focus ring of collapsible triggers (`ring-1 ring-blue/60 rounded-xs`).
pub const FOCUS_RING: f32 = 1.0;
pub const FOCUS_RADIUS: f32 = 2.0;

/// The working label's shimmer: a 300 %-wide gradient swept across in 2 s.
pub const SHIMMER: Loop = Loop::new(Duration::from_millis(2000), ease::LINEAR);
/// The working bullet's blink: opacity 1 → 0.35 → 1 over 1.2 s.
pub const BLINK: Loop = Loop::new(Duration::from_millis(1200), ease::LINEAR);
/// The gradient stops of the shimmer: dim until 35 %, the text colour at 50 %, dim from 65 %.
pub const SHIMMER_STOPS: (f32, f32, f32) = (0.35, 0.5, 0.65);

/// The colours of Codex's terminal, from the item's `--codex-*` tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CodexPalette {
    pub background: Color,
    pub title_bar: Color,
    pub dot: Color,
    pub title: Color,
    pub shadow: Color,
    pub ring: Color,
    pub fg: Color,
    pub dim: Color,
    pub muted: Color,
    pub rule: Color,
    pub band: Color,
    pub composer: Color,
    pub placeholder: Color,
    pub cursor: Color,
    pub blue: Color,
    pub command: Color,
    pub arg: Color,
    pub operator: Color,
    pub flag: Color,
    pub green: Color,
    pub success: Color,
    pub yellow: Color,
    pub amber: Color,
    pub red: Color,
}

impl CodexPalette {
    pub const fn light() -> Self {
        Self {
            background: Color::rgb(0xffffff),
            title_bar: Color::rgb(0xececec),
            dot: Color::rgb(0xd4d4d4),
            title: Color::rgba(0x00000073),
            shadow: Color::rgba(0x00000026),
            ring: Color::rgba(0x0000001a),
            fg: Color::rgb(0x1a1a1a),
            dim: Color::rgb(0x5c5c5c),
            muted: Color::rgb(0x767676),
            rule: Color::rgb(0xe0e0e0),
            band: Color::rgb(0xf0f0f0),
            composer: Color::rgb(0xf3f3f3),
            placeholder: Color::rgb(0x8a8a8a),
            cursor: Color::rgb(0xd6006a),
            blue: Color::rgb(0x0a6fd6),
            command: Color::rgb(0x1f5fbf),
            arg: Color::rgb(0x3d4f8f),
            operator: Color::rgb(0x0f7f70),
            flag: Color::rgb(0xc0364d),
            green: Color::rgb(0x1f8a3a),
            success: Color::rgb(0x00852a),
            yellow: Color::rgb(0x8f6200),
            amber: Color::rgb(0x8a6420),
            red: Color::rgb(0xd12f2f),
        }
    }

    pub const fn dark() -> Self {
        Self {
            background: Color::rgb(0x0f0f0f),
            title_bar: Color::rgb(0x1c1c1c),
            dot: Color::rgb(0x3a3a3a),
            title: Color::rgba(0xffffff66),
            shadow: Color::rgba(0x00000033),
            ring: Color::rgba(0xffffff00),
            fg: Color::rgb(0xf7f7f7),
            dim: Color::rgb(0x909090),
            muted: Color::rgb(0x777777),
            rule: Color::rgb(0x343434),
            band: Color::rgb(0x2d2d2d),
            composer: Color::rgb(0x252525),
            placeholder: Color::rgb(0x868686),
            cursor: Color::rgb(0xfc0076),
            blue: Color::rgb(0x3c9ffc),
            command: Color::rgb(0x70abfa),
            arg: Color::rgb(0xc3cff4),
            operator: Color::rgb(0x6cdfcd),
            flag: Color::rgb(0xf3909f),
            green: Color::rgb(0x85df8d),
            success: Color::rgb(0x00a12e),
            yellow: Color::rgb(0xfcdb9b),
            amber: Color::rgb(0xbe9952),
            red: Color::rgb(0xff6b6b),
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
            title_bar: self.title_bar,
            traffic: self.dot,
            title: self.title,
            shadow: self.shadow,
            ring: self.ring,
        }
    }

    /// ``code`` in green; Codex has no bold mark in its replies.
    pub const fn inline(&self) -> InlineStyle {
        InlineStyle {
            text: self.fg,
            code: self.green,
            strong: None,
        }
    }
}

/// A reply paragraph with ``code`` spans.
pub fn rich(text: &str, palette: &CodexPalette) -> Rich {
    Rich(inline(text, palette.inline()))
}

/// The status of a command cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ExecStatus {
    #[default]
    Ran,
    /// Running: the bullet pulses.
    Running,
    Failed,
}

impl ExecStatus {
    pub fn label(self) -> &'static str {
        match self {
            ExecStatus::Running => "Running",
            ExecStatus::Ran | ExecStatus::Failed => "Ran",
        }
    }

    pub fn bullet(self, p: &CodexPalette) -> Color {
        match self {
            ExecStatus::Ran => p.success,
            ExecStatus::Running => p.muted,
            ExecStatus::Failed => p.red,
        }
    }
}

/// What a shell token is, for colouring a command line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShellToken {
    Command,
    Flag,
    String,
    Operator,
    Argument,
    Space,
}

impl ShellToken {
    pub fn color(self, p: &CodexPalette) -> Color {
        match self {
            ShellToken::Command => p.command,
            ShellToken::Flag => p.flag,
            ShellToken::String => p.green,
            ShellToken::Operator => p.operator,
            ShellToken::Argument => p.arg,
            ShellToken::Space => p.fg,
        }
    }
}

/// Splits a command line as Codex colours it: the first word after an operator is a
/// command, quoted strings and `&& || | ; < >` stand alone, dashes start flags.
pub fn tokenize_shell(command: &str) -> Vec<(ShellToken, &str)> {
    let mut tokens = Vec::new();
    let mut expects_command = true;
    let mut rest = command;
    while let Some(c) = rest.chars().next() {
        let (kind, len) = if c.is_whitespace() {
            let len = rest
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(rest.len());
            (ShellToken::Space, len)
        } else if c == '\'' || c == '"' {
            let len = rest[1..].find(c).map_or(rest.len(), |end| end + 2);
            expects_command = false;
            (ShellToken::String, len)
        } else if rest.starts_with("&&") || rest.starts_with("||") {
            expects_command = true;
            (ShellToken::Operator, 2)
        } else if "|;<>".contains(c) {
            expects_command = true;
            (ShellToken::Operator, 1)
        } else {
            let len = if c == '&' {
                1
            } else {
                rest.find(|c: char| c.is_whitespace() || "'\"|;&<>".contains(c))
                    .unwrap_or(rest.len())
            };
            let word = &rest[..len];
            let kind = if expects_command {
                expects_command = false;
                ShellToken::Command
            } else if word.starts_with('-') {
                ShellToken::Flag
            } else {
                ShellToken::Argument
            };
            (kind, len)
        };
        tokens.push((kind, &rest[..len]));
        rest = &rest[len..];
    }
    tokens
}

/// A command line coloured by token.
pub fn shell_line(command: &str, p: &CodexPalette) -> Vec<Run> {
    tokenize_shell(command)
        .into_iter()
        .map(|(kind, text)| Run::new(text, kind.color(p)))
        .collect()
}

/// A `Ran` cell: the command and its output.
#[derive(Clone, Debug, PartialEq)]
pub struct Exec {
    pub id: String,
    pub command: String,
    pub output: Option<String>,
    pub status: ExecStatus,
}

impl Exec {
    pub fn new(id: &str, command: &str, output: Option<&str>, status: ExecStatus) -> Self {
        Self {
            id: id.into(),
            command: command.into(),
            output: output.map(Into::into),
            status,
        }
    }

    /// The first output lines, and the lines behind `+ N lines`.
    pub fn split_output(&self) -> (Vec<&str>, Vec<&str>) {
        let lines: Vec<&str> = self
            .output
            .as_deref()
            .map_or(Vec::new(), |o| o.split('\n').collect());
        let cut = lines.len().min(PREVIEW_LINES);
        (lines[..cut].to_vec(), lines[cut..].to_vec())
    }
}

/// What an explored line did.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExploredVerb {
    Read,
    Search,
    List,
}

impl ExploredVerb {
    pub fn label(self) -> &'static str {
        match self {
            ExploredVerb::Read => "Read",
            ExploredVerb::Search => "Search",
            ExploredVerb::List => "List",
        }
    }
}

/// One line of an `Explored` cell.
#[derive(Clone, Debug, PartialEq)]
pub struct ExploredItem {
    pub id: &'static str,
    pub verb: ExploredVerb,
    pub target: &'static str,
    pub scope: Option<&'static str>,
}

impl ExploredItem {
    pub fn line(&self, p: &CodexPalette) -> Rich {
        let mut r = Rich::new()
            .push(Run::new(self.verb.label(), p.blue))
            .push(Run::new(format!(" {}", self.target), p.fg));
        if let Some(scope) = self.scope {
            r = r
                .push(Run::new(" in ", p.muted))
                .push(Run::new(scope, p.fg));
        }
        r
    }
}

/// The colour of the shimmering label's character `index` of `count`, `elapsed` into
/// the sweep: the gradient `dim 35% · fg 50% · dim 65%` at the character's centre.
pub fn shimmer_color(
    index: usize,
    count: usize,
    elapsed: Duration,
    dim: Color,
    fg: Color,
) -> Color {
    let width = count.max(1) as f32;
    let position = 1.0 - SHIMMER.phase(elapsed);
    let x = index as f32 + 0.5;
    let u = (x + 2.0 * width * position) / (3.0 * width);
    let (start, peak, end) = SHIMMER_STOPS;
    let t = if u <= start || u >= end {
        0.0
    } else if u <= peak {
        (u - start) / (peak - start)
    } else {
        (end - u) / (end - peak)
    };
    lerp(dim, fg, t)
}

fn lerp(a: Color, b: Color, t: f32) -> Color {
    let m = |x: f32, y: f32| x + (y - x) * t;
    Color {
        r: m(a.r, b.r),
        g: m(a.g, b.g),
        b: m(a.b, b.b),
        a: m(a.a, b.a),
    }
}

/// The working bullet's opacity `elapsed` into its blink.
pub fn blink_opacity(elapsed: Duration) -> f32 {
    keyframes(
        &[(0.0, 1.0), (0.5, 0.35), (1.0, 1.0)],
        BLINK.easing,
        BLINK.phase(elapsed),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TurnStatus {
    Working,
    Streaming,
    Done,
    Interrupted,
}

/// One exchange of the demo after its opening session.
#[derive(Clone, Debug, PartialEq)]
pub struct Turn {
    pub id: String,
    pub prompt: String,
    pub execs: Vec<Exec>,
    pub reply: String,
    pub status: TurnStatus,
}

impl Turn {
    /// The reply's paragraphs (blank lines split them).
    pub fn paragraphs(&self) -> Vec<&str> {
        self.reply.split("\n\n").collect()
    }
}

/// A scripted answer: commands run first (with their final status), then the reply.
#[derive(Clone, Debug, PartialEq)]
pub struct Reply {
    pub execs: &'static [(&'static str, &'static str, &'static str, ExecStatus)],
    pub text: &'static str,
}

/// The playback's paces (`CODEX_CHAT_TIMING`).
pub mod timing {
    pub const THINK_MS: u64 = 1100;
    pub const EXEC_MS: u64 = 1000;
    pub const WORD_MS: u64 = 26;
    pub const REDUCED_MAX_MS: u64 = 60;
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

struct Timeline {
    think: Duration,
    exec: Duration,
    word: Duration,
    tokens: Vec<&'static str>,
    execs: usize,
}

impl Timeline {
    fn new(reply: &Reply, pace: Pace) -> Self {
        Self {
            think: pace.wait(timing::THINK_MS),
            exec: pace.wait(timing::EXEC_MS),
            word: pace.wait(timing::WORD_MS),
            tokens: if pace.reduced {
                vec![reply.text]
            } else {
                stream_tokens(reply.text)
            },
            execs: reply.execs.len(),
        }
    }

    fn stream_start(&self) -> Duration {
        self.think + self.exec * self.execs as u32
    }

    fn done(&self) -> Duration {
        self.stream_start() + self.word * self.tokens.len() as u32
    }

    fn steps(&self) -> impl Iterator<Item = Duration> + '_ {
        let execs = (0..=self.execs).map(|i| self.think + self.exec * i as u32);
        let words = (0..=self.tokens.len()).map(|k| self.stream_start() + self.word * k as u32);
        execs.chain(words)
    }

    fn streamed(&self, elapsed: Duration) -> usize {
        if elapsed < self.stream_start() {
            return 0;
        }
        if self.word.is_zero() {
            return self.tokens.len();
        }
        let steps =
            ((elapsed - self.stream_start()).as_nanos() / self.word.as_nanos()) as usize + 1;
        steps.min(self.tokens.len())
    }
}

/// What the demo shows at one moment.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatView {
    pub turns: Vec<Turn>,
    /// Whole seconds since the running turn started, while one runs.
    pub working: Option<u64>,
}

/// The demo's chat (`useCodexChat`) as a pure function of time.
pub struct Chat {
    turns: Vec<Turn>,
    replies: &'static [Reply],
    active: Option<Active>,
    sent: usize,
}

impl Chat {
    pub fn new(replies: &'static [Reply]) -> Self {
        Self {
            turns: Vec::new(),
            replies,
            active: None,
            sent: 0,
        }
    }

    pub fn demo() -> Self {
        Self::new(session::REPLIES)
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
            id: format!("codex-turn-{}", self.sent),
            prompt: prompt.into(),
            reply,
            started: now,
            pace,
        });
    }

    /// Escape: keeps what has printed, drops the running command, marks the turn.
    pub fn stop(&mut self, now: Duration) {
        if let Some(mut turn) = self.active_turn(now) {
            turn.execs.retain(|e| e.status != ExecStatus::Running);
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
        let timeline = Timeline::new(&self.replies[active.reply], active.pace);
        if now.saturating_sub(active.started) < timeline.done() {
            return false;
        }
        if let Some(mut turn) = self.active_turn(now) {
            turn.status = TurnStatus::Done;
            self.turns.push(turn);
        }
        self.active = None;
        true
    }

    /// The next moment the view changes: a command or token landing, or the counter ticking.
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
        let working = self
            .active
            .as_ref()
            .map(|a| now.saturating_sub(a.started).as_secs());
        turns.extend(self.active_turn(now));
        ChatView { turns, working }
    }

    fn active_turn(&self, now: Duration) -> Option<Turn> {
        let active = self.active.as_ref()?;
        let reply = &self.replies[active.reply];
        let timeline = Timeline::new(reply, active.pace);
        let elapsed = now.saturating_sub(active.started);
        let execs = reply
            .execs
            .iter()
            .enumerate()
            .take_while(|(i, _)| elapsed >= timeline.think + timeline.exec * *i as u32)
            .map(|(i, &(id, command, output, status))| {
                let id = format!("{}-{id}", active.id);
                if elapsed >= timeline.think + timeline.exec * (i as u32 + 1) {
                    Exec::new(&id, command, Some(output), status)
                } else {
                    Exec::new(&id, command, None, ExecStatus::Running)
                }
            })
            .collect();
        Some(Turn {
            id: active.id.clone(),
            prompt: active.prompt.clone(),
            execs,
            reply: timeline.tokens[..timeline.streamed(elapsed)].concat(),
            status: if elapsed >= timeline.stream_start() {
                TurnStatus::Streaming
            } else {
                TurnStatus::Working
            },
        })
    }
}

/// The demo session (`CODEX_DEMO_SESSION`, `CODEX_REPLIES`) and component defaults.
pub mod session {
    use super::*;

    pub const TITLE: &str = "codex — ~/acme/web";
    pub const VERSION: &str = "0.159.2";
    pub const CWD: &str = "~/acme/web";
    pub const GREETING: &str = "Anything interesting on the docket?";
    pub const MODEL: &str = "gpt-6.1-codex";
    pub const EFFORT: &str = "low";
    pub const PLACEHOLDER: &str = "Ask Codex to do anything";
    pub const TASK: &str = "Draft changelog";
    pub const WARNINGS: u32 = 2;
    pub const INTERRUPTED: &str =
        "Conversation interrupted - tell the model what to do differently.";

    pub const USER_MESSAGE: &str = "what shipped this week? draft the changelog";
    pub const INTRO: &str = "I'll check this week's merged commits and the current changelog, then draft the new entry.";
    pub const REASONING: &str = "Grouping commits into features and fixes";
    pub const SUMMARY: &str = "This week shipped 3 features and 4 fixes. Here is the draft:";
    pub const TABLE_INTRO: &str = "The changes touch these parts of the repo:";
    pub const FOLLOW_UP: &str = "Want me to publish it to acme.com/changelog?";

    pub const EXEC_COMMAND: &str =
        "git log --oneline --since='1 week' | rg -i 'feat|fix' && cat CHANGELOG.md";
    pub const EXEC_OUTPUT: &str = "a41c9e2 feat(geo): track citations per model\n7d02b18 fix(scheduler): retry failed publish jobs\nc9e5f11 feat(studio): brand voice presets\n4be0a7d fix(api): paginate /v1/posts past 100 items\n91f3c20 fix(geo): dedupe prompts across projects\ne03d6b4 feat(api): add /v1/citations endpoint\n2c7a9f8 fix(ui): focus ring on the composer\n## 2026-09-23\n- Scheduler v2 with 40% faster builds";

    pub const EXPLORED: &[ExploredItem] = &[
        ExploredItem {
            id: "search-changelog",
            verb: ExploredVerb::Search,
            target: "changelog|release",
            scope: Some("CHANGELOG.md"),
        },
        ExploredItem {
            id: "read-package",
            verb: ExploredVerb::Read,
            target: "package.json",
            scope: None,
        },
    ];

    pub const EXPLORED_DETAILS: &[ExploredItem] = &[
        ExploredItem {
            id: "list-apps",
            verb: ExploredVerb::List,
            target: "apps",
            scope: None,
        },
        ExploredItem {
            id: "read-readme",
            verb: ExploredVerb::Read,
            target: "apps/dashboard/README.md",
            scope: None,
        },
    ];

    /// `(label, text)` of each highlight.
    pub const HIGHLIGHTS: &[(&str, &str)] = &[
        (
            "GEO citations:",
            "See which sources each AI model cites for your tracked prompts.",
        ),
        (
            "Brand voice presets:",
            "Save a voice once and reuse it for every changelog and post.",
        ),
        ("Scheduler:", "Failed publish jobs now retry on their own."),
    ];

    pub const TABLE_HEADERS: &[&str] = &["Folder", "Change"];
    pub const TABLE_ROWS: &[&[&str]] = &[
        &["apps/dashboard", "Citations view on the GEO page"],
        &["packages/geo-core", "Citation tracking per model"],
        &["apps/api", "New /v1/citations endpoint"],
        &["packages/ui", "Brand voice picker"],
    ];

    /// The exec states the exec example shows.
    pub fn exec_states() -> Vec<Exec> {
        vec![
            Exec::new(
                "ran",
                "notra list_events --range week --json | jq '.[].title'",
                Some(
                    "\"Scheduler v2\"\n\"GEO citations\"\n\"Brand voice presets\"\n\"Paginate /v1/posts\"\n\"Dedupe prompts\"",
                ),
                ExecStatus::Ran,
            ),
            Exec::new(
                "running",
                "notra create_post --type changelog --voice 'acme'",
                None,
                ExecStatus::Running,
            ),
            Exec::new(
                "failed",
                "notra publish_post changelog",
                Some("error: missing NOTRA_API_KEY"),
                ExecStatus::Failed,
            ),
        ]
    }

    pub const REPLIES: &[Reply] = &[
        Reply {
            execs: &[(
                "publish",
                "notra publish_post changelog --site acme.com",
                "published changelog-2026-09-30\nhttps://acme.com/changelog/2026-09-30",
                ExecStatus::Ran,
            )],
            text: "Published. The entry is live at `acme.com/changelog/2026-09-30` and the RSS feed picked it up.\n\nWant a short post for LinkedIn as well?",
        },
        Reply {
            execs: &[(
                "grep-retry",
                "rg -n 'retry' packages/api/src/jobs",
                "publish.ts:41:  const retry = createRetry({ attempts: 3 });\npublish.ts:58:  await retry(() => sendToSite(post));\npublish.test.ts:12:it('retries a failed publish', async () => {",
                ExecStatus::Ran,
            )],
            text: "The retry lives in `packages/api/src/jobs/publish.ts`. Failed publish jobs retry three times with backoff, and `publish.test.ts` covers it.",
        },
        Reply {
            execs: &[(
                "tests",
                "bun run test --filter api",
                "212 pass\n0 fail",
                ExecStatus::Ran,
            )],
            text: "All 212 tests pass. Nothing is blocking the release.",
        },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn shell_tokens_follow_codex() {
        let kinds: Vec<_> = tokenize_shell("git log --since='1 week' | rg -i x && cat a")
            .into_iter()
            .filter(|(k, _)| *k != ShellToken::Space)
            .collect();
        assert_eq!(
            kinds,
            [
                (ShellToken::Command, "git"),
                (ShellToken::Argument, "log"),
                (ShellToken::Flag, "--since="),
                (ShellToken::String, "'1 week'"),
                (ShellToken::Operator, "|"),
                (ShellToken::Command, "rg"),
                (ShellToken::Flag, "-i"),
                (ShellToken::Argument, "x"),
                (ShellToken::Operator, "&&"),
                (ShellToken::Command, "cat"),
                (ShellToken::Argument, "a"),
            ]
        );
    }

    #[test]
    fn exec_previews_three_lines() {
        let exec = Exec::new("x", "c", Some(session::EXEC_OUTPUT), ExecStatus::Ran);
        let (preview, rest) = exec.split_output();
        assert_eq!(preview.len(), 3);
        assert_eq!(rest.len(), 6);
    }

    #[test]
    fn shimmer_sweeps_left_to_right() {
        let (dim, fg) = (Color::BLACK, Color::WHITE);
        let at = |i, ms| shimmer_color(i, 7, MS(ms), dim, fg).r;
        assert_eq!(at(0, 0), 0.0);
        assert!(at(0, 700) > 0.5);
        assert!(at(6, 1300) > 0.5);
        assert!(at(6, 1300) > at(0, 1300));
    }

    #[test]
    fn playback_runs_then_streams() {
        let mut chat = Chat::demo();
        chat.send("publish it", Duration::ZERO, pace(false));
        let turn = chat.view(MS(1_200)).turns.pop();
        assert!(turn.is_some_and(|t| t.execs.len() == 1
            && t.execs[0].status == ExecStatus::Running
            && t.execs[0].output.is_none()));
        let turn = chat.view(MS(2_200)).turns.pop();
        assert!(turn.is_some_and(
            |t| t.execs[0].status == ExecStatus::Ran && t.reply.starts_with("Published")
        ));
        assert_eq!(chat.view(MS(2_200)).working, Some(2));
        assert!(chat.settle(MS(60_000)));
        assert_eq!(chat.view(MS(60_000)).working, None);
    }

    #[test]
    fn stop_drops_the_running_command() {
        let mut chat = Chat::demo();
        chat.send("go", Duration::ZERO, pace(false));
        chat.stop(MS(1_500));
        let turn = chat.view(MS(1_500)).turns.pop();
        assert!(turn.is_some_and(|t| t.status == TurnStatus::Interrupted && t.execs.is_empty()));
    }
}
