//! Codex for GPUI: the Codex CLI session block's parts and the playable demo.

use std::time::{Duration, Instant};

use ::gpui::{
    AnyElement, App, AppContext as _, Context, ElementId, Entity, FontWeight, HighlightStyle,
    InteractiveElement, IntoElement, ParentElement, Render, RenderOnce, ScrollHandle, SharedString,
    Styled, StyledText, Subscription, Window, div, prelude::*, px,
};

use super::ActiveTheme;
use super::terminal::{
    Disclosure, PromptEvent, PromptInput, TerminalWindow, bind_prompt, disclosure,
    disclosure_trigger, pulse_opacity, rich_text,
};
use crate::components::codex::{
    self as model, CodexPalette, Exec, ExecStatus, ExploredItem, FONT_SIZE, GAP, HALF_GAP,
    LINE_HEIGHT, STATUS_GAP, Turn, TurnStatus, session,
};
use crate::components::terminal::{Rich, Run, ch};
use crate::motion::{Tween, duration, ease};
use crate::theme::Shadow;

const HOVER: Tween = Tween::new(duration::FAST, ease::IN_OUT);

fn palette(cx: &App) -> CodexPalette {
    CodexPalette::new(cx.notra().appearance)
}

fn cells(n: f32) -> ::gpui::Pixels {
    px(ch(n, FONT_SIZE))
}

/// `grid-cols-[2ch_minmax(0,1fr)] px-[1ch]`: the gutter and the content column.
fn codex_row(lead: impl IntoElement, content: impl IntoElement) -> ::gpui::Div {
    div()
        .flex()
        .w_full()
        .min_w_0()
        .px(cells(1.0))
        .child(div().w(cells(2.0)).flex_none().child(lead))
        .child(div().flex_1().min_w_0().child(content))
}

/// A `grid-cols-[2ch_minmax(0,1fr)]` line inside the content column.
fn tree_line(lead: &'static str, content: impl IntoElement) -> ::gpui::Div {
    div()
        .flex()
        .min_w_0()
        .child(div().w(cells(2.0)).flex_none().child(lead))
        .child(div().flex_1().min_w_0().child(content))
}

fn trigger(
    id: ElementId,
    state: &Entity<Disclosure>,
    p: &CodexPalette,
    cx: &App,
) -> ::gpui::Stateful<::gpui::Div> {
    disclosure_trigger(
        id,
        state,
        Shadow::drop(0.0, 0.0, model::FOCUS_RING, p.blue.fade(0.6)),
        model::FOCUS_RADIUS,
        cx,
    )
    .text_color(p.muted)
    .hover(|s| s.text_color(p.fg))
}

/// The terminal window: chrome, the scrolling session and an optional footer.
#[derive(IntoElement)]
pub struct CodexTerminal {
    id: ElementId,
    title: Option<SharedString>,
    height: Option<f32>,
    children: Vec<AnyElement>,
    footer: Option<AnyElement>,
    scroll: Option<ScrollHandle>,
}

impl CodexTerminal {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            height: None,
            children: Vec::new(),
            footer: None,
            scroll: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Pinned under the session: the working line and the composer.
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    pub fn track_scroll(mut self, handle: &ScrollHandle) -> Self {
        self.scroll = Some(handle.clone());
        self
    }
}

impl ParentElement for CodexTerminal {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for CodexTerminal {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let mut window = TerminalWindow::new(self.id, p.chrome())
            .text(FONT_SIZE, LINE_HEIGHT)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(GAP))
                    .py(px(GAP))
                    .text_color(p.fg)
                    .children(self.children),
            );
        if let Some(title) = self.title {
            window = window.title(title);
        }
        if let Some(height) = self.height {
            window = window.height(height);
        }
        if let Some(scroll) = &self.scroll {
            window = window.track_scroll(scroll);
        }
        if let Some(footer) = self.footer {
            window = window.footer(
                div()
                    .flex_none()
                    .pb(px(HALF_GAP))
                    .text_color(p.fg)
                    .child(footer),
            );
        }
        window
    }
}

/// `>_ OpenAI Codex (v…)`, the cwd and a greeting.
#[derive(IntoElement)]
pub struct CodexHeader {
    version: SharedString,
    cwd: SharedString,
    greeting: Option<SharedString>,
}

impl CodexHeader {
    pub fn new() -> Self {
        Self {
            version: session::VERSION.into(),
            cwd: session::CWD.into(),
            greeting: Some(session::GREETING.into()),
        }
    }

    pub fn version(mut self, version: impl Into<SharedString>) -> Self {
        self.version = version.into();
        self
    }

    pub fn cwd(mut self, cwd: impl Into<SharedString>) -> Self {
        self.cwd = cwd.into();
        self
    }

    pub fn greeting(mut self, greeting: Option<SharedString>) -> Self {
        self.greeting = greeting;
        self
    }
}

impl Default for CodexHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for CodexHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let title = Rich::new()
            .push(Run::new(">_ ", p.blue))
            .push(Run::new("OpenAI Codex", p.fg).bold())
            .push(Run::new(format!(" (v{})", self.version), p.muted));
        codex_row(
            "",
            div()
                .flex()
                .flex_col()
                .child(rich_text(&title))
                .child(
                    div()
                        .pl(cells(3.0))
                        .truncate()
                        .text_color(p.muted)
                        .child(self.cwd),
                )
                .when_some(self.greeting, |d, g| {
                    d.child(div().mt(px(GAP)).text_color(p.blue).child(g))
                }),
        )
    }
}

/// A prompt on its band (`›`) or a reply (`•`) whose paragraphs sit 1.3em apart.
#[derive(IntoElement)]
pub struct CodexMessage {
    user: Option<SharedString>,
    children: Vec<AnyElement>,
}

impl CodexMessage {
    pub fn user(text: impl Into<SharedString>) -> Self {
        Self {
            user: Some(text.into()),
            children: Vec::new(),
        }
    }

    pub fn assistant() -> Self {
        Self {
            user: None,
            children: Vec::new(),
        }
    }
}

impl ParentElement for CodexMessage {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for CodexMessage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        match self.user {
            Some(text) => codex_row(div().text_color(p.dim).child("›"), text)
                .py(px(GAP))
                .bg(p.band),
            None => codex_row(
                div().text_color(p.muted).child("•"),
                div().flex().flex_col().gap(px(GAP)).children(self.children),
            ),
        }
    }
}

/// A reply paragraph: ``code`` in green.
pub fn codex_paragraph(text: &str, cx: &App) -> StyledText {
    rich_text(&model::rich(text, &palette(cx)))
}

/// The model's reasoning summary: bold italic and muted.
#[derive(IntoElement)]
pub struct CodexReasoning {
    text: SharedString,
}

impl CodexReasoning {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for CodexReasoning {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        codex_row(
            "•",
            div()
                .font_weight(FontWeight::BOLD)
                .italic()
                .child(self.text),
        )
        .text_color(p.muted)
    }
}

/// `• Ran <command>` with its output, the rest behind `+ N lines`.
#[derive(IntoElement)]
pub struct CodexExec {
    id: ElementId,
    exec: Exec,
}

impl CodexExec {
    pub fn new(id: impl Into<ElementId>, exec: Exec) -> Self {
        Self {
            id: id.into(),
            exec,
        }
    }
}

impl RenderOnce for CodexExec {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let exec = self.exec;
        let bullet_opacity = if exec.status == ExecStatus::Running {
            pulse_opacity((self.id.clone(), "pulse"), window, cx)
        } else {
            1.0
        };
        let headline = Rich::new()
            .push(Run::new(exec.status.label(), p.fg).bold())
            .push(Run::new(" ", p.fg))
            .extend(model::shell_line(&exec.command, &p));
        let (preview, rest) = exec.split_output();
        let mut output = None;
        if !preview.is_empty() {
            let mut lines = div()
                .flex()
                .flex_col()
                .min_w_0()
                .text_color(p.muted)
                .child(tree_line("└", preview.join("\n")));
            if !rest.is_empty() {
                let state = disclosure(&self.id, false, HOVER, window, cx);
                let open = state.read(cx).open;
                let label = if open {
                    "− collapse".to_owned()
                } else {
                    format!("+ {} lines (click to expand)", rest.len())
                };
                lines = lines
                    .when(open, |d| d.child(tree_line("", rest.join("\n"))))
                    .child(tree_line(
                        "",
                        div().flex().child(
                            trigger(self.id, &state, &p, cx)
                                .when(open, |d| d.text_color(p.fg))
                                .child(label),
                        ),
                    ));
            }
            output = Some(lines);
        }
        codex_row(
            div()
                .text_color(exec.status.bullet(&p))
                .opacity(bullet_opacity)
                .child("•"),
            div()
                .flex()
                .flex_col()
                .min_w_0()
                .child(div().truncate().child(rich_text(&headline)))
                .children(output),
        )
    }
}

/// `• Explored` (or `Exploring`) and what was read, searched and listed.
#[derive(IntoElement)]
pub struct CodexExplored {
    id: ElementId,
    items: Vec<ExploredItem>,
    details: Vec<ExploredItem>,
    active: bool,
}

impl CodexExplored {
    pub fn new(id: impl Into<ElementId>, items: Vec<ExploredItem>) -> Self {
        Self {
            id: id.into(),
            items,
            details: Vec::new(),
            active: false,
        }
    }

    /// Lines behind `+ Show details`.
    pub fn details(mut self, details: Vec<ExploredItem>) -> Self {
        self.details = details;
        self
    }

    /// Still exploring: the bullet pulses.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

impl RenderOnce for CodexExplored {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let opacity = if self.active {
            pulse_opacity((self.id.clone(), "pulse"), window, cx)
        } else {
            1.0
        };
        let line = |item: &ExploredItem, first: bool| {
            div()
                .flex()
                .min_w_0()
                .child(
                    div()
                        .w(cells(2.0))
                        .flex_none()
                        .text_color(p.muted)
                        .child(if first { "└" } else { "" }),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .child(rich_text(&item.line(&p))),
                )
        };
        let mut body = div()
            .flex()
            .flex_col()
            .min_w_0()
            .child(div().font_weight(FontWeight::BOLD).child(if self.active {
                "Exploring"
            } else {
                "Explored"
            }))
            .children(
                self.items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| line(item, i == 0)),
            );
        if !self.details.is_empty() {
            let state = disclosure(&self.id, false, HOVER, window, cx);
            let open = state.read(cx).open;
            body = body
                .when(open, |d| {
                    d.children(self.details.iter().map(|item| line(item, false)))
                })
                .child(
                    div().flex().pl(cells(2.0)).child(
                        trigger(self.id, &state, &p, cx)
                            .when(open, |d| d.text_color(p.fg))
                            .child(if open {
                                "− Hide details"
                            } else {
                                "+ Show details"
                            }),
                    ),
                );
        }
        codex_row(div().text_color(p.muted).opacity(opacity).child("•"), body)
    }
}

/// `• Working (13s • esc to interrupt)` with the shimmering label and a blinking bullet.
#[derive(IntoElement)]
pub struct CodexWorking {
    id: ElementId,
    label: SharedString,
    elapsed: SharedString,
}

impl CodexWorking {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: "Working".into(),
            elapsed: "0s".into(),
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn elapsed(mut self, elapsed: impl Into<SharedString>) -> Self {
        self.elapsed = elapsed.into();
        self
    }
}

impl RenderOnce for CodexWorking {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let started = *window
            .use_keyed_state(self.id.clone(), cx, |_, _| Instant::now())
            .read(cx);
        let still = cx.reduce_motion();
        let elapsed = if still {
            Duration::ZERO
        } else {
            started.elapsed()
        };
        if !still {
            window.request_animation_frame();
        }
        let count = self.label.chars().count();
        let mut highlights = Vec::new();
        for (i, (at, c)) in self.label.char_indices().enumerate() {
            let color = if still {
                p.dim
            } else {
                model::shimmer_color(i, count, elapsed, p.dim, p.fg)
            };
            highlights.push((
                at..at + c.len_utf8(),
                HighlightStyle {
                    color: Some(color.into()),
                    ..Default::default()
                },
            ));
        }
        let rest = Rich::new()
            .push(Run::new(format!(" ({} • ", self.elapsed), p.dim))
            .push(Run::new("esc", p.fg).bold())
            .push(Run::new(" to interrupt)", p.dim));
        codex_row(
            div()
                .text_color(p.fg)
                .opacity(if still {
                    1.0
                } else {
                    model::blink_opacity(elapsed)
                })
                .child("•"),
            div()
                .flex()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .child(StyledText::new(self.label.clone()).with_highlights(highlights))
                .child(rich_text(&rest)),
        )
        .text_color(p.dim)
    }
}

/// `■ Conversation interrupted…` in red.
#[derive(IntoElement)]
pub struct CodexInterrupted {
    text: SharedString,
}

impl CodexInterrupted {
    pub fn new() -> Self {
        Self {
            text: session::INTERRUPTED.into(),
        }
    }

    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = text.into();
        self
    }
}

impl Default for CodexInterrupted {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for CodexInterrupted {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        codex_row("■", self.text).text_color(palette(cx).red)
    }
}

/// A bulleted list inside a reply.
#[derive(IntoElement)]
pub struct CodexList {
    items: Vec<Rich>,
}

impl CodexList {
    pub fn new(items: Vec<Rich>) -> Self {
        Self { items }
    }
}

impl RenderOnce for CodexList {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().flex().flex_col().children(
            self.items
                .iter()
                .map(|item| tree_line("•", rich_text(item))),
        )
    }
}

/// A table inside a reply: yellow headers over a 2 px rule, 1 px rules between rows.
#[derive(IntoElement)]
pub struct CodexTable {
    headers: Vec<SharedString>,
    rows: Vec<Vec<SharedString>>,
    code_columns: Vec<usize>,
}

impl CodexTable {
    pub fn new(headers: Vec<SharedString>, rows: Vec<Vec<SharedString>>) -> Self {
        Self {
            headers,
            rows,
            code_columns: Vec::new(),
        }
    }

    /// Columns drawn as code (green).
    pub fn code_columns(mut self, columns: Vec<usize>) -> Self {
        self.code_columns = columns;
        self
    }
}

impl RenderOnce for CodexTable {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let columns = self.headers.len();
        let width = |c: usize| {
            let longest = std::iter::once(&self.headers[c])
                .chain(self.rows.iter().filter_map(|r| r.get(c)))
                .map(|s| s.chars().count())
                .max()
                .unwrap_or(0);
            cells(longest as f32 + 2.0)
        };
        let widths: Vec<_> = (0..columns).map(width).collect();
        let last = self.rows.len().saturating_sub(1);
        let column = |c: usize| {
            let header = div()
                .px(cells(1.0))
                .pb(px(HALF_GAP))
                .border_b_2()
                .border_color(p.rule)
                .text_color(p.yellow)
                .font_weight(FontWeight::BOLD)
                .child(self.headers[c].clone());
            let cells_in = self.rows.iter().enumerate().map(|(r, row)| {
                div()
                    .px(cells(1.0))
                    .py(px(HALF_GAP))
                    .when(r < last, |d| d.border_b_1().border_color(p.rule))
                    .when(self.code_columns.contains(&c), |d| d.text_color(p.green))
                    .child(row.get(c).cloned().unwrap_or_default())
            });
            div()
                .flex()
                .flex_col()
                .flex_none()
                .w(widths[c])
                .whitespace_nowrap()
                .child(header)
                .children(cells_in)
        };
        div()
            .flex()
            .ml(-cells(2.0))
            .pl(cells(2.0))
            .gap(cells(2.0))
            .overflow_hidden()
            .children((0..columns).map(column))
    }
}

/// The composer: the prompt band and the model, cwd and shortcut lines under it.
#[derive(IntoElement)]
pub struct CodexComposer {
    id: ElementId,
    input: Entity<PromptInput>,
    placeholder: SharedString,
    model: SharedString,
    effort: Option<SharedString>,
    cwd: SharedString,
    task: Option<SharedString>,
    warnings: Option<u32>,
    context: Option<SharedString>,
}

impl CodexComposer {
    /// Reads and types into `input`; subscribe to its [`PromptEvent`]s to send.
    pub fn new(id: impl Into<ElementId>, input: Entity<PromptInput>) -> Self {
        Self {
            id: id.into(),
            input,
            placeholder: session::PLACEHOLDER.into(),
            model: session::MODEL.into(),
            effort: None,
            cwd: session::CWD.into(),
            task: None,
            warnings: None,
            context: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn model(mut self, model: impl Into<SharedString>) -> Self {
        self.model = model.into();
        self
    }

    pub fn effort(mut self, effort: impl Into<SharedString>) -> Self {
        self.effort = Some(effort.into());
        self
    }

    pub fn cwd(mut self, cwd: impl Into<SharedString>) -> Self {
        self.cwd = cwd.into();
        self
    }

    pub fn task(mut self, task: impl Into<SharedString>) -> Self {
        self.task = Some(task.into());
        self
    }

    pub fn warnings(mut self, warnings: u32) -> Self {
        self.warnings = Some(warnings);
        self
    }

    /// Shown on the right when there are no warnings.
    pub fn context(mut self, context: impl Into<SharedString>) -> Self {
        self.context = Some(context.into());
        self
    }
}

impl RenderOnce for CodexComposer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let (text, focused) = {
            let input = self.input.read(cx);
            (input.text().to_owned(), input.is_focused(window))
        };
        let field = if text.is_empty() {
            let mut chars = self.placeholder.chars();
            let first = chars.next().map_or(" ".to_owned(), String::from);
            let rest: String = chars.collect();
            div()
                .flex()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .child(div().bg(p.cursor).text_color(p.fg).child(first))
                .child(div().text_color(p.placeholder).child(rest))
        } else {
            div()
                .flex()
                .min_w_0()
                .text_color(p.fg)
                .child(div().truncate().child(text))
                .when(focused, |d| {
                    d.child(div().w(px(1.0)).h(px(LINE_HEIGHT)).bg(p.cursor))
                })
        };
        let band = bind_prompt(
            div()
                .id(self.id)
                .flex()
                .w_full()
                .px(cells(1.0))
                .py(px(GAP))
                .bg(p.composer)
                .cursor_text()
                .child(
                    div()
                        .w(cells(2.0))
                        .flex_none()
                        .text_color(p.fg)
                        .font_weight(FontWeight::BOLD)
                        .child("›"),
                )
                .child(div().flex_1().min_w_0().child(field)),
            &self.input,
            cx,
        );
        let dot = || Run::new(" · ", p.dim);
        let key = |k: &str| Run::new(k, p.fg).bold();
        let model_label = match &self.effort {
            Some(effort) => format!("{} {effort}", self.model),
            None => self.model.to_string(),
        };
        let mut first = Rich::new()
            .push(Run::new(model_label, p.yellow))
            .push(dot())
            .push(Run::new(self.cwd.to_string(), p.green));
        if let Some(task) = &self.task {
            first = first.push(dot()).push(Run::new(task.to_string(), p.yellow));
        }
        let shortcuts = Rich::new()
            .push(key("←"))
            .push(Run::new(" for agents", p.dim))
            .push(dot())
            .push(key("?"))
            .push(Run::new(" for shortcuts", p.dim));
        let right = match (self.warnings.filter(|&w| w > 0), &self.context) {
            (Some(w), _) => Some(rich_text(
                &Rich::new()
                    .push(Run::new(
                        format!("⚠ {w} {}", if w == 1 { "warning" } else { "warnings" }),
                        p.amber,
                    ))
                    .push(dot())
                    .push(key("f2"))
                    .push(Run::new(" to view", p.dim)),
            )),
            (None, Some(context)) => Some(rich_text(&Rich::plain(context.to_string(), p.dim))),
            (None, None) => None,
        };
        div()
            .flex()
            .flex_col()
            .min_w_0()
            .text_color(p.dim)
            .child(band)
            .child(
                codex_row(
                    "",
                    div()
                        .flex()
                        .flex_col()
                        .child(div().truncate().child(rich_text(&first)))
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .justify_between()
                                .gap_x(cells(2.0))
                                .child(rich_text(&shortcuts))
                                .children(right),
                        ),
                )
                .mt(px(STATUS_GAP)),
            )
    }
}

/// One exchange of the demo: prompt, commands, reply, interruption.
#[derive(IntoElement)]
pub struct CodexTurn {
    turn: Turn,
}

impl CodexTurn {
    pub fn new(turn: Turn) -> Self {
        Self { turn }
    }
}

impl RenderOnce for CodexTurn {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let turn = self.turn;
        let paragraphs: Vec<StyledText> = if turn.reply.is_empty() {
            Vec::new()
        } else {
            turn.paragraphs()
                .iter()
                .map(|t| codex_paragraph(t, cx))
                .collect()
        };
        div()
            .flex()
            .flex_col()
            .gap(px(GAP))
            .child(CodexMessage::user(turn.prompt.clone()))
            .children(
                turn.execs
                    .into_iter()
                    .map(|exec| CodexExec::new(ElementId::Name(exec.id.clone().into()), exec)),
            )
            .when(!paragraphs.is_empty(), |d| {
                d.child(CodexMessage::assistant().children(paragraphs))
            })
            .when(turn.status == TurnStatus::Interrupted, |d| {
                d.child(CodexInterrupted::new())
            })
    }
}

/// The demo session's opening cells, before any playback.
pub fn codex_session(cx: &App) -> Vec<AnyElement> {
    let p = palette(cx);
    let highlights = session::HIGHLIGHTS
        .iter()
        .map(|(label, text)| {
            Rich::new()
                .push(Run::new(*label, p.fg).bold())
                .push(Run::new(format!(" {text}"), p.fg))
        })
        .collect();
    let rows = session::TABLE_ROWS
        .iter()
        .map(|row| row.iter().map(|c| SharedString::from(*c)).collect())
        .collect();
    vec![
        CodexHeader::new().into_any_element(),
        CodexMessage::user(session::USER_MESSAGE).into_any_element(),
        CodexMessage::assistant()
            .child(session::INTRO)
            .into_any_element(),
        CodexExec::new(
            "codex-session-exec",
            Exec::new(
                "exec",
                session::EXEC_COMMAND,
                Some(session::EXEC_OUTPUT),
                ExecStatus::Ran,
            ),
        )
        .into_any_element(),
        CodexExplored::new("codex-session-explored", session::EXPLORED.to_vec())
            .details(session::EXPLORED_DETAILS.to_vec())
            .into_any_element(),
        CodexMessage::assistant()
            .child(session::SUMMARY)
            .child(CodexList::new(highlights))
            .child(session::TABLE_INTRO)
            .child(
                CodexTable::new(
                    session::TABLE_HEADERS
                        .iter()
                        .map(|h| SharedString::from(*h))
                        .collect(),
                    rows,
                )
                .code_columns(vec![0]),
            )
            .child(session::FOLLOW_UP)
            .into_any_element(),
    ]
}

/// The playable demo: the session, a composer that sends scripted replies, the
/// working line while one plays and Escape to interrupt it.
pub struct CodexDemo {
    chat: model::Chat,
    input: Entity<PromptInput>,
    scroll: ScrollHandle,
    epoch: Instant,
    follow: bool,
    shown: Option<model::ChatView>,
    _events: Subscription,
}

impl CodexDemo {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(PromptInput::new);
        let events = cx.subscribe_in(&input, window, |this, input, event, _window, cx| {
            let now = this.epoch.elapsed();
            match event {
                PromptEvent::Submit(text) => {
                    if this.chat.is_busy() {
                        return;
                    }
                    this.chat.send(text, now, model::pace(cx.reduce_motion()));
                    this.follow = true;
                    input.update(cx, |input, cx| input.clear(cx));
                }
                PromptEvent::Escape => this.chat.stop(now),
                PromptEvent::CycleMode => {}
            }
            cx.notify();
        });
        Self {
            chat: model::Chat::demo(),
            input,
            scroll: ScrollHandle::new(),
            epoch: Instant::now(),
            follow: false,
            shown: None,
            _events: events,
        }
    }

    fn near_bottom(&self) -> Option<bool> {
        if self.scroll.bounds().size.height <= px(0.0) {
            return None;
        }
        let max = f32::from(self.scroll.max_offset().y);
        let offset = -f32::from(self.scroll.offset().y);
        Some(max - offset < 48.0)
    }
}

impl Render for CodexDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = self.epoch.elapsed();
        self.chat.settle(now);
        let view = self.chat.view(now);
        if let Some(due) = self.chat.next_change(now) {
            let wait = due.saturating_sub(now) + Duration::from_millis(1);
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(wait).await;
                this.update(cx, |_, cx| cx.notify()).ok();
            })
            .detach();
        }
        if !self.chat.is_busy()
            && let Some(near) = self.near_bottom()
        {
            self.follow = near;
        }
        if self.shown.as_ref() != Some(&view) {
            if self.follow && self.shown.is_some() {
                self.scroll.scroll_to_bottom();
            }
            self.shown = Some(view.clone());
        }
        let footer = div()
            .flex()
            .flex_col()
            .when_some(view.working, |d, secs| {
                d.child(
                    div()
                        .pb(px(HALF_GAP))
                        .child(CodexWorking::new("codex-demo-working").elapsed(format!("{secs}s"))),
                )
            })
            .child(
                CodexComposer::new("codex-demo-composer", self.input.clone())
                    .effort(session::EFFORT)
                    .task(session::TASK)
                    .warnings(session::WARNINGS),
            );
        CodexTerminal::new("codex-demo")
            .title(session::TITLE)
            .height(model::DEMO_HEIGHT)
            .track_scroll(&self.scroll)
            .footer(footer)
            .children(codex_session(cx))
            .children(view.turns.into_iter().map(CodexTurn::new))
    }
}
