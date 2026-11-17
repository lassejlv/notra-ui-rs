//! Claude Code for Rune: the terminal block's parts and the playable demo.

use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;

use ::rune::tok::TextStyle;
use ::rune::{El, Key, KeyEvent, Span, TextEdit, Tx, col, el, rich, row, text, text_input};

use super::terminal::{TerminalWindow, code_font, frame_glyph, pulse, rich_text};
use super::{ToRune, tween};
use crate::components::claude_code::{
    self as model, CONTENT_PADDING_X, CONTENT_PADDING_Y, ClaudeCodeEffort, ClaudeCodeMode,
    ClaudeCodePalette, FONT_SIZE, GAP, Header, LINE_HEIGHT, RESULT_GLYPH, SPINNER, SPINNER_FRAMES,
    Spinner, Todo, ToolCall, ToolStatus, Turn, TurnStatus, TurnSummary, session,
};
use crate::components::terminal::{Rich, Run, ch};
use crate::theme::{Color, Theme};

/// Reaches a part's state inside the app's state.
pub type Lens<S, T> = fn(&mut S) -> &mut T;

fn palette(theme: &Theme) -> ClaudeCodePalette {
    ClaudeCodePalette::new(theme.appearance)
}

fn font() -> TextStyle {
    code_font(FONT_SIZE, 400.0, LINE_HEIGHT)
}

fn cells(n: f32) -> f32 {
    ch(n, FONT_SIZE)
}

fn glyph(s: &str, color: Color) -> El {
    text(s).font(font()).color(color.rune())
}

/// A `grid-cols-[Nch_minmax(0,1fr)]` row: a fixed gutter, then the content.
fn gutter_row(gutter: f32, lead: El, content: El) -> El {
    row()
        .w_full()
        .child(el().w(cells(gutter)).child(lead))
        .child(content.grow(1.0).shrink().min_w(0.0))
}

/// Which collapsibles are open, owned by the app.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClaudeCodeState {
    open: HashSet<String>,
}

impl ClaudeCodeState {
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

/// A Base UI collapsible trigger: a focusable button that toggles `id`.
fn trigger<S: 'static>(
    id: &str,
    open: bool,
    p: &ClaudeCodePalette,
    lens: impl Fn(&mut S) -> &mut ClaudeCodeState + Copy + 'static,
) -> El {
    let key = id.to_owned();
    row()
        .key(format!("{id}-trigger"))
        .role("button")
        .expanded(open)
        .focusable(true)
        .radius(model::FOCUS_RADIUS)
        .focus_ring(model::FOCUS_RING, 0.0, p.muted.fade(0.6).rune())
        .cursor(::rune::Cursor::Pointer)
        .hover_group()
        .on_press(move |s: &mut S| lens(s).toggle(&key))
}

/// The terminal window: chrome, a scrolling transcript and an optional pinned prompt.
pub struct ClaudeCodeTerminal {
    key: String,
    title: Option<String>,
    height: Option<f32>,
    children: Vec<El>,
    footer: Option<El>,
    follow: bool,
}

impl ClaudeCodeTerminal {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: None,
            height: None,
            children: Vec::new(),
            footer: None,
            follow: false,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn child(mut self, child: El) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = El>) -> Self {
        self.children.extend(children);
        self
    }

    /// Pinned under the transcript, like Claude Code's prompt.
    pub fn footer(mut self, footer: El) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Keeps the transcript at its end while output arrives.
    pub fn follow(mut self, follow: bool) -> Self {
        self.follow = follow;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        let body = col()
            .w_full()
            .gap(GAP)
            .px(CONTENT_PADDING_X)
            .py(CONTENT_PADDING_Y)
            .children(self.children);
        let mut window = TerminalWindow::new(self.key, p.chrome())
            .body(body)
            .follow(self.follow);
        if let Some(title) = self.title {
            window = window.title(title);
        }
        if let Some(height) = self.height {
            window = window.height(height);
        }
        if let Some(footer) = self.footer {
            window = window.footer(
                col()
                    .w_full()
                    .px(CONTENT_PADDING_X)
                    .pb(model::FOOTER_PADDING_BOTTOM)
                    .child(footer),
            );
        }
        window.build()
    }
}

/// The pixel mascot in the accent colour.
pub struct ClaudeCodeLogo {
    size: f32,
}

impl ClaudeCodeLogo {
    pub fn new() -> Self {
        Self {
            size: model::LOGO_WIDTH,
        }
    }

    /// Its width; the height keeps the 22 × 14 grid.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let accent = palette(theme).accent.rune();
        let (columns, rows) = model::MASCOT_GRID;
        let cell = self.size / columns;
        model::MASCOT.iter().fold(
            el().size(self.size, cell * rows)
                .role("img")
                .label("Claude Code"),
            |logo, &(x, y, w, h)| {
                logo.child(
                    el().pin(Some(x * cell), Some(y * cell), None, None)
                        .size(w * cell, h * cell)
                        .fill(accent),
                )
            },
        )
    }
}

impl Default for ClaudeCodeLogo {
    fn default() -> Self {
        Self::new()
    }
}

/// The launch header: mascot, version, plan and cwd, then notices and tips.
pub struct ClaudeCodeHeader {
    header: Header,
}

impl ClaudeCodeHeader {
    pub fn new(header: Header) -> Self {
        Self { header }
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        let h = self.header;
        let title = Rich::new()
            .push(Run::new("Claude Code", p.strong).bold())
            .push(Run::new(" ", p.fg))
            .push(Run::new(h.version, p.muted));
        let line = |s: &str| glyph(s, p.muted).single_line().truncate();
        let plan = h.plan();
        let mut lines = col()
            .shrink()
            .min_w(0.0)
            .child(rich_text(&title, font()).single_line().truncate());
        if !plan.is_empty() {
            lines = lines.child(line(&plan));
        }
        if !h.cwd.is_empty() {
            lines = lines.child(line(h.cwd));
        }
        let mut e = col().w_full().gap(GAP).child(
            row()
                .center_y()
                .gap(cells(2.0))
                .child(ClaudeCodeLogo::new().build(theme))
                .child(lines),
        );
        if !h.whats_new.is_empty() || !h.tips.is_empty() {
            let notices = h
                .whats_new
                .iter()
                .map(|l| rich_text(&model::rich(l, p.fg, &p), font()))
                .chain(h.tips.iter().map(|l| {
                    row()
                        .gap(cells(2.0))
                        .child(glyph(RESULT_GLYPH, p.muted))
                        .child(
                            rich_text(&model::rich(l, p.muted, &p), font())
                                .shrink()
                                .min_w(0.0),
                        )
                }));
            e = e.child(col().w_full().pl(cells(2.0)).children(notices));
        }
        e
    }
}

/// A prompt (`❯` on a band) or a reply (`●`), in the inline grammar.
pub struct ClaudeCodeMessage {
    text: String,
    user: bool,
}

impl ClaudeCodeMessage {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            user: true,
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            user: false,
        }
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        let (mark, mark_color, color) = if self.user {
            ("❯", p.user_prompt, p.strong)
        } else {
            ("●", p.strong, p.fg)
        };
        let e = gutter_row(
            2.0,
            glyph(mark, mark_color),
            rich_text(&model::rich(&self.text, color, &p), font()),
        );
        if self.user {
            e.fill(p.user_bg.rune())
        } else {
            e
        }
    }
}

/// `∴ Thinking… (click to expand)` over the reasoning.
pub struct ClaudeCodeThinking {
    id: String,
    label: String,
    body: Option<String>,
}

impl ClaudeCodeThinking {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: "Thinking…".into(),
            body: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn build<S: 'static>(
        self,
        theme: &Theme,
        state: &ClaudeCodeState,
        lens: impl Fn(&mut S) -> &mut ClaudeCodeState + Copy + 'static,
    ) -> El {
        let p = palette(theme);
        let open = state.is_open(&self.id);
        let label = Rich::new()
            .push(Run::new(self.label, p.muted).italic())
            .push(Run::new(
                if open { "" } else { " (click to expand)" },
                p.muted,
            ));
        let head = trigger(&self.id, open, &p, lens)
            .child(el().w(cells(2.0)).child(glyph("∴", p.muted)))
            .child(rich_text(&label, font()).shrink().min_w(0.0));
        let mut e = col().w_full().child(row().child(head));
        if let Some(body) = self.body.filter(|_| open) {
            e = e.child(
                rich_text(&Rich(vec![Run::new(body, p.muted).italic()]), font()).pl(cells(2.0)),
            );
        }
        e
    }
}

/// `● Update Todos` and the checklist under it.
pub struct ClaudeCodeTodoList {
    todos: Vec<Todo>,
    heading: Option<String>,
}

impl ClaudeCodeTodoList {
    pub fn new(todos: Vec<Todo>) -> Self {
        Self {
            todos,
            heading: Some("Update Todos".into()),
        }
    }

    /// The tool line above the list; `None` leaves it out.
    pub fn heading(mut self, heading: Option<String>) -> Self {
        self.heading = heading;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        let has_heading = self.heading.is_some();
        let mut e = col().w_full();
        if let Some(heading) = self.heading {
            e = e.child(gutter_row(
                2.0,
                glyph("●", p.success),
                rich_text(&Rich(vec![Run::new(heading, p.strong).bold()]), font()),
            ));
        }
        let items = self.todos.into_iter().enumerate().map(|(i, todo)| {
            let corner = if i == 0 && has_heading {
                RESULT_GLYPH
            } else {
                ""
            };
            row()
                .w_full()
                .label(format!("{}: {}", todo.status.label(), todo.label))
                .child(el().w(cells(3.0)).child(glyph(corner, p.muted)))
                .child(
                    row()
                        .w(cells(2.0))
                        .justify_center()
                        .child(glyph(todo.status.glyph(), todo.status.glyph_color(&p))),
                )
                .child(
                    rich_text(&Rich(vec![todo.status.text(&todo.label, &p)]), font())
                        .pl(cells(1.0))
                        .grow(1.0)
                        .shrink()
                        .min_w(0.0),
                )
        });
        e.child(col().w_full().pl(cells(2.0)).role("list").children(items))
    }
}

/// A tool call: `● Tool(arg)` and `⎿ result`, its detail behind a click.
pub struct ClaudeCodeToolCall {
    call: ToolCall,
}

impl ClaudeCodeToolCall {
    pub fn new(call: ToolCall) -> Self {
        Self { call }
    }

    pub fn build<S: 'static>(
        self,
        theme: &Theme,
        state: &ClaudeCodeState,
        lens: impl Fn(&mut S) -> &mut ClaudeCodeState + Copy + 'static,
    ) -> El {
        let p = palette(theme);
        let call = self.call;
        let mut bullet = glyph("●", call.status.bullet(&p)).key(format!("{}-bullet", call.id));
        if call.status == ToolStatus::Pending {
            bullet = bullet.looping(pulse());
        }
        let headline = gutter_row(2.0, bullet, rich_text(&call.headline(&p), font()))
            .label(format!("{}: {}", call.status.label(), call.tool));
        let result = |hint: bool| {
            row()
                .w_full()
                .pl(cells(2.0))
                .child(el().w(cells(3.0)).child(glyph(RESULT_GLYPH, p.muted)))
                .child(
                    text(format!(
                        "{}{}",
                        call.result,
                        if hint { " (click to expand)" } else { "" }
                    ))
                    .font(font())
                    .color(p.muted.rune())
                    .group_color(p.fg.rune())
                    .wrap()
                    .grow(1.0)
                    .shrink()
                    .min_w(0.0),
                )
        };
        let mut e = col().w_full().key(call.id.clone()).child(headline);
        match &call.detail {
            Some(detail) => {
                let open = state.is_open(&call.id);
                e = e.child(
                    trigger(&call.id, open, &p, lens)
                        .w_full()
                        .child(result(!open)),
                );
                if open {
                    e = e.child(glyph(detail, p.muted).wrap().pl(cells(5.0)));
                }
            }
            None => e = e.child(result(false)),
        }
        e
    }
}

/// `Ran 3 shell commands`, folding the calls under it.
pub struct ClaudeCodeToolSummary {
    id: String,
    calls: Vec<ToolCall>,
    verb: String,
    noun: String,
}

impl ClaudeCodeToolSummary {
    pub fn new(id: impl Into<String>, calls: Vec<ToolCall>) -> Self {
        Self {
            id: id.into(),
            calls,
            verb: "Ran".into(),
            noun: "shell command".into(),
        }
    }

    pub fn build<S: 'static>(
        self,
        theme: &Theme,
        state: &ClaudeCodeState,
        lens: impl Fn(&mut S) -> &mut ClaudeCodeState + Copy + 'static,
    ) -> El {
        let p = palette(theme);
        let count = self.calls.len();
        let noun = if count == 1 {
            self.noun
        } else {
            format!("{}s", self.noun)
        };
        let open = state.is_open(&self.id);
        let content = format!("{} {count} {noun}", self.verb);
        let bold_at = self.verb.len() + 1;
        let summary = rich(
            content,
            vec![Span::new(bold_at, bold_at + count.to_string().len()).weight(700.0)],
        )
        .font(font())
        .color(p.muted.rune())
        .group_color(p.fg.rune());
        let mut head = trigger(&self.id, open, &p, lens).child(summary);
        if !open {
            head = head.child(
                text(" (click to expand)")
                    .font(font())
                    .color(p.fg.rune())
                    .group_opacity(0.0, 1.0)
                    .hover_motion(tween(model::HINT_FADE), tween(model::HINT_FADE)),
            );
        }
        let mut e = col()
            .w_full()
            .gap(GAP)
            .pl(cells(2.0))
            .child(row().child(head));
        if open {
            e = e.child(
                col().w_full().gap(GAP).ml(-cells(2.0)).children(
                    self.calls
                        .into_iter()
                        .map(|call| ClaudeCodeToolCall::new(call).build(theme, state, lens)),
                ),
            );
        }
        e
    }
}

/// The spinner line while a turn runs.
pub struct ClaudeCodeSpinner {
    key: String,
    spinner: Spinner,
    paused: bool,
}

impl ClaudeCodeSpinner {
    pub fn new(key: impl Into<String>, spinner: Spinner) -> Self {
        Self {
            key: key.into(),
            spinner,
            paused: false,
        }
    }

    /// Freezes the glyph, for screenshots.
    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = paused;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        let s = self.spinner;
        let mut line = Rich::new().push(Run::new(format!("{}…", s.verb), p.accent));
        if let Some(meta) = s.meta() {
            line = line.push(Run::new(format!(" ({meta})"), p.muted));
        }
        let mut e = col().w_full().role("status").child(gutter_row(
            2.0,
            frame_glyph(
                &self.key,
                &SPINNER_FRAMES,
                SPINNER,
                font(),
                p.accent,
                self.paused,
            ),
            rich_text(&line, font()),
        ));
        if let Some(tip) = s.tip {
            e = e.child(
                gutter_row(
                    3.0,
                    glyph(RESULT_GLYPH, p.muted),
                    rich_text(&model::rich(&tip, p.muted, &p), font()),
                )
                .pl(cells(2.0)),
            );
        }
        e
    }
}

/// `✻ Worked for 19s · done 9:42 AM`.
pub struct ClaudeCodeTurnSummary {
    summary: TurnSummary,
}

impl ClaudeCodeTurnSummary {
    pub fn new(summary: TurnSummary) -> Self {
        Self { summary }
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        gutter_row(
            2.0,
            glyph("✻", p.muted),
            glyph(&self.summary.text(), p.muted).wrap(),
        )
    }
}

/// `⎿ Interrupted · What should Claude do instead?`
pub struct ClaudeCodeInterrupted {
    hint: Option<String>,
}

impl ClaudeCodeInterrupted {
    pub fn new() -> Self {
        Self {
            hint: Some("What should Claude do instead?".into()),
        }
    }

    pub fn hint(mut self, hint: Option<String>) -> Self {
        self.hint = hint;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        let mut line = Rich::new().push(Run::new("Interrupted", p.error));
        if let Some(hint) = self.hint {
            line = line.push(Run::new(format!(" · {hint}"), p.muted));
        }
        gutter_row(3.0, glyph(RESULT_GLYPH, p.muted), rich_text(&line, font()))
            .pl(cells(2.0))
            .role("status")
    }
}

impl Default for ClaudeCodeInterrupted {
    fn default() -> Self {
        Self::new()
    }
}

type KeyHandler<S> = Box<dyn Fn(&mut S, &KeyEvent, &mut Tx) -> bool>;
type SubmitHandler<S> = Box<dyn Fn(&mut S, &mut Tx)>;

/// The prompt: a ruled input line and the mode and effort status line.
pub struct ClaudeCodePrompt<S> {
    key: String,
    draft: TextEdit,
    edit: Rc<dyn Fn(&mut S) -> &mut TextEdit>,
    placeholder: String,
    mode: ClaudeCodeMode,
    effort: Option<ClaudeCodeEffort>,
    pull_request: Option<u32>,
    submit: Option<SubmitHandler<S>>,
    keys: Option<KeyHandler<S>>,
}

impl<S: 'static> ClaudeCodePrompt<S> {
    /// A field over `draft`, which `edit` reaches in the app state.
    pub fn new(
        key: impl Into<String>,
        draft: &TextEdit,
        edit: impl Fn(&mut S) -> &mut TextEdit + 'static,
    ) -> Self {
        Self {
            key: key.into(),
            draft: draft.clone(),
            edit: Rc::new(edit),
            placeholder: String::new(),
            mode: ClaudeCodeMode::Manual,
            effort: None,
            pull_request: None,
            submit: None,
            keys: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn mode(mut self, mode: ClaudeCodeMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn effort(mut self, effort: Option<ClaudeCodeEffort>) -> Self {
        self.effort = effort;
        self
    }

    pub fn pull_request(mut self, number: Option<u32>) -> Self {
        self.pull_request = number;
        self
    }

    /// Return with text in the field.
    pub fn on_submit(mut self, f: impl Fn(&mut S, &mut Tx) + 'static) -> Self {
        self.submit = Some(Box::new(f));
        self
    }

    /// Keys the field leaves alone (Escape, Shift+Tab); true consumes one.
    pub fn on_key(mut self, f: impl Fn(&mut S, &KeyEvent, &mut Tx) -> bool + 'static) -> Self {
        self.keys = Some(Box::new(f));
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        let empty = self.draft.text.is_empty();
        let edit = self.edit;
        let mut input = text_input(&self.draft, font())
            .key(format!("{}-input", self.key))
            .single_line()
            .label("Message Claude Code")
            .color(p.strong.rune())
            .placeholder(self.placeholder.clone(), p.muted.rune())
            .caret(
                if empty {
                    ::rune::Color::TRANSPARENT
                } else {
                    p.strong.rune()
                },
                p.link.fade(0.3).rune(),
            )
            .on_edit(move |s: &mut S| edit(s))
            .w_full();
        if let Some(submit) = self.submit {
            input = input.on_submit(move |s: &mut S, tx: &mut Tx| submit(s, tx));
        }
        let mut field = el().grow(1.0).shrink().min_w(0.0).child(input);
        if empty {
            let (block, ink) = p.cursor();
            let first = self
                .placeholder
                .chars()
                .next()
                .map_or(" ".into(), String::from);
            field = field.child(
                el().pin(Some(0.0), Some(0.0), None, Some(0.0))
                    .w(cells(1.0))
                    .fill(block.rune())
                    .child(glyph(&first, ink)),
            );
        }
        let mut line = row()
            .key(self.key.clone())
            .w_full()
            .py(model::PROMPT_PADDING_Y)
            .border_sides([1.0, 0.0, 1.0, 0.0], p.rule.rune())
            .child(el().w(cells(2.0)).child(glyph("❯", p.strong)))
            .child(field);
        if let Some(keys) = self.keys {
            line = line.on_key(move |s: &mut S, k: &KeyEvent, tx: &mut Tx| keys(s, k, tx));
        }
        let mut status = Rich::new().push(Run::new(self.mode.label(), self.mode.color(&p)));
        if self.mode.cycles() {
            status = status.push(Run::new(" (shift+tab to cycle)", p.muted));
        }
        if let Some(number) = self.pull_request {
            status = status
                .push(Run::new(" · PR ", p.muted))
                .push(Run::new(format!("#{number}"), p.pr).underline());
        }
        let mut meta = row()
            .w_full()
            .flex_wrap()
            .justify_between()
            .gap(cells(2.0))
            .pl(cells(2.0))
            .child(rich_text(&status, font()).shrink().min_w(0.0));
        if let Some(effort) = self.effort {
            meta = meta.child(glyph(
                &format!("{} {} · /effort", effort.glyph(), effort.name()),
                effort.color(&p),
            ));
        }
        col().w_full().gap(GAP).child(line).child(meta)
    }
}

/// One exchange: prompt, todos, tool calls (folded when all shell), reply, status.
pub struct ClaudeCodeTurn {
    turn: Turn,
}

impl ClaudeCodeTurn {
    pub fn new(turn: Turn) -> Self {
        Self { turn }
    }

    pub fn build<S: 'static>(
        self,
        theme: &Theme,
        state: &ClaudeCodeState,
        lens: impl Fn(&mut S) -> &mut ClaudeCodeState + Copy + 'static,
    ) -> El {
        let turn = self.turn;
        let folds = turn.folds_shell();
        let mut e = col()
            .key(turn.id.clone())
            .w_full()
            .gap(GAP)
            .child(ClaudeCodeMessage::user(turn.prompt.clone()).build(theme));
        if !turn.todos.is_empty() {
            e = e.child(ClaudeCodeTodoList::new(turn.todos.clone()).build(theme));
        }
        if folds {
            e = e.child(
                ClaudeCodeToolSummary::new(format!("{}-summary", turn.id), turn.commands)
                    .build(theme, state, lens),
            );
        } else {
            e = e.children(turn.commands.into_iter().map(|call| {
                ClaudeCodeToolCall::new(ToolCall {
                    id: format!("{}-{}", turn.id, call.id),
                    ..call
                })
                .build(theme, state, lens)
            }));
        }
        if !turn.answer.is_empty() {
            e = e.child(ClaudeCodeMessage::assistant(turn.answer).build(theme));
        }
        if turn.status == TurnStatus::Interrupted {
            e = e.child(ClaudeCodeInterrupted::new().build(theme));
        }
        if let Some(summary) = turn.summary {
            e = e.child(ClaudeCodeTurnSummary::new(summary).build(theme));
        }
        e
    }
}

/// The playable demo's state, owned by the app: the chat, the draft, the mode and
/// the open collapsibles. Drive it from the app's `poll` with [`ClaudeCodeDemoState::poll`].
pub struct ClaudeCodeDemoState {
    pub chat: model::Chat,
    pub draft: TextEdit,
    pub mode: ClaudeCodeMode,
    pub disclosure: ClaudeCodeState,
    /// Plays replies at the reduced-motion pace.
    pub reduced_motion: bool,
    now: Duration,
    follow: bool,
    shown: Option<model::ChatView>,
}

impl Default for ClaudeCodeDemoState {
    fn default() -> Self {
        Self {
            chat: model::Chat::demo(),
            draft: TextEdit::new(""),
            mode: ClaudeCodeMode::Bypass,
            disclosure: ClaudeCodeState::default(),
            reduced_motion: false,
            now: Duration::ZERO,
            follow: false,
            shown: None,
        }
    }
}

impl ClaudeCodeDemoState {
    /// Advances the playback to `clock` (seconds, as `poll` gets it), asks for a wake
    /// at the next step and reports whether what's shown changed.
    pub fn poll(&mut self, clock: f64, tx: &mut Tx) -> bool {
        self.now = Duration::from_secs_f64(clock.max(0.0));
        self.chat.settle(self.now);
        if let Some(due) = self.chat.next_change(self.now) {
            tx.wake_at(due.as_secs_f64());
        }
        let view = self.chat.view(self.now);
        let changed = self.shown.as_ref() != Some(&view);
        self.shown = Some(view);
        changed
    }

    /// Sends the draft, as Return does.
    pub fn send(&mut self, tx: &mut Tx) {
        let message = self.draft.text.trim().to_owned();
        if message.is_empty() || self.chat.is_busy() {
            return;
        }
        let now = Duration::from_secs_f64(tx.clock().max(0.0));
        self.send_at(&message, now);
        self.draft = TextEdit::new("");
        tx.scroll_to(format!("{DEMO_KEY}-body"), 0.0, f32::MAX);
        if let Some(due) = self.chat.next_change(now) {
            tx.wake_at(due.as_secs_f64());
        }
    }

    /// Sends `prompt` at `now` on the app clock and follows the transcript.
    pub fn send_at(&mut self, prompt: &str, now: Duration) {
        self.now = now;
        self.chat
            .send(prompt, now, model::pace(self.reduced_motion));
        self.follow = true;
    }

    /// Escape and Shift+Tab: interrupt the running turn, cycle the mode.
    pub fn key(&mut self, key: &KeyEvent, tx: &mut Tx) -> bool {
        match key.key {
            Key::Escape if self.chat.is_busy() => {
                self.chat.stop(Duration::from_secs_f64(tx.clock().max(0.0)));
                true
            }
            Key::Tab if key.mods.shift => {
                self.mode = self.mode.next();
                true
            }
            _ => false,
        }
    }
}

const DEMO_KEY: &str = "claude-code-demo";

/// The playable demo: the session transcript, a prompt that sends scripted replies,
/// Escape to interrupt and Shift+Tab to cycle the mode.
pub struct ClaudeCodeDemo;

impl ClaudeCodeDemo {
    pub fn build<S: 'static>(
        theme: &Theme,
        state: &ClaudeCodeDemoState,
        lens: Lens<S, ClaudeCodeDemoState>,
    ) -> El {
        let view = state.chat.view(state.now);
        let turns = view.turns.into_iter().map(|turn| {
            ClaudeCodeTurn::new(turn).build(theme, &state.disclosure, move |s: &mut S| {
                &mut lens(s).disclosure
            })
        });
        let prompt = ClaudeCodePrompt::new(
            format!("{DEMO_KEY}-prompt"),
            &state.draft,
            move |s: &mut S| &mut lens(s).draft,
        )
        .placeholder(session::PLACEHOLDER)
        .mode(state.mode)
        .pull_request(Some(session::PULL_REQUEST))
        .on_submit(move |s: &mut S, tx: &mut Tx| lens(s).send(tx))
        .on_key(move |s: &mut S, k: &KeyEvent, tx: &mut Tx| lens(s).key(k, tx));
        ClaudeCodeTerminal::new(DEMO_KEY)
            .title(session::TITLE)
            .height(model::DEMO_HEIGHT)
            .follow(state.follow)
            .footer(prompt.build(theme))
            .child(ClaudeCodeHeader::new(session::HEADER).build(theme))
            .children(turns)
            .children(
                view.spinner
                    .map(|s| ClaudeCodeSpinner::new(format!("{DEMO_KEY}-spinner"), s).build(theme)),
            )
            .build(theme)
    }
}
