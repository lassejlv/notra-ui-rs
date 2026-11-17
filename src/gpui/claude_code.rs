//! Claude Code for GPUI: the terminal block's parts and the playable demo.

use std::time::{Duration, Instant};

use ::gpui::{
    AnyElement, App, AppContext as _, Context, ElementId, Entity, FocusHandle, FontWeight,
    HighlightStyle, InteractiveElement, IntoElement, ParentElement, Render, RenderOnce,
    ScrollHandle, SharedString, StatefulInteractiveElement, Styled, StyledText, Subscription,
    Window, div, prelude::*, px,
};

use super::terminal::{
    FrameGlyph, PromptEvent, PromptInput, TerminalWindow, bind_prompt, pulse_opacity, rich_text,
};
use super::{ActiveTheme, shadows};
use crate::components::claude_code::{
    self as model, CONTENT_PADDING_X, CONTENT_PADDING_Y, ClaudeCodeEffort, ClaudeCodeMode,
    ClaudeCodePalette, FONT_SIZE, GAP, Header, LINE_HEIGHT, RESULT_GLYPH, SPINNER, SPINNER_FRAMES,
    Spinner, Todo, ToolCall, ToolStatus, Turn, TurnStatus, TurnSummary, session,
};
use crate::components::terminal::{Rich, Run, ch};
use crate::motion::Animated;
use crate::theme::Shadow;

fn palette(cx: &App) -> ClaudeCodePalette {
    ClaudeCodePalette::new(cx.notra().appearance)
}

fn cells(n: f32) -> ::gpui::Pixels {
    px(ch(n, FONT_SIZE))
}

/// A `grid-cols-[Nch_minmax(0,1fr)]` row: a fixed gutter, then the content.
fn gutter_row(gutter: f32, lead: impl IntoElement, content: impl IntoElement) -> ::gpui::Div {
    div()
        .flex()
        .min_w_0()
        .child(div().w(cells(gutter)).flex_none().child(lead))
        .child(div().flex_1().min_w_0().child(content))
}

/// The terminal window: chrome, a scrolling transcript and an optional pinned prompt.
#[derive(IntoElement)]
pub struct ClaudeCodeTerminal {
    id: ElementId,
    title: Option<SharedString>,
    height: Option<f32>,
    children: Vec<AnyElement>,
    footer: Option<AnyElement>,
    scroll: Option<ScrollHandle>,
}

impl ClaudeCodeTerminal {
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

    /// Pinned under the transcript, like Claude Code's prompt.
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    pub fn track_scroll(mut self, handle: &ScrollHandle) -> Self {
        self.scroll = Some(handle.clone());
        self
    }
}

impl ParentElement for ClaudeCodeTerminal {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ClaudeCodeTerminal {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let mut window = TerminalWindow::new(self.id, p.chrome())
            .text(FONT_SIZE, LINE_HEIGHT)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(GAP))
                    .px(px(CONTENT_PADDING_X))
                    .py(px(CONTENT_PADDING_Y))
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
                    .px(px(CONTENT_PADDING_X))
                    .pb(px(model::FOOTER_PADDING_BOTTOM))
                    .text_color(p.fg)
                    .child(footer),
            );
        }
        window
    }
}

/// The pixel mascot, `size` wide, in the accent colour.
#[derive(IntoElement)]
pub struct ClaudeCodeLogo {
    size: f32,
}

impl ClaudeCodeLogo {
    pub fn new(size: f32) -> Self {
        Self { size }
    }
}

impl Default for ClaudeCodeLogo {
    fn default() -> Self {
        Self::new(model::LOGO_WIDTH)
    }
}

impl RenderOnce for ClaudeCodeLogo {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let accent = palette(cx).accent;
        let (columns, rows) = model::MASCOT_GRID;
        let cell = self.size / columns;
        div()
            .relative()
            .flex_none()
            .w(px(self.size))
            .h(px(cell * rows))
            .children(model::MASCOT.iter().map(|&(x, y, w, h)| {
                div()
                    .absolute()
                    .left(px(x * cell))
                    .top(px(y * cell))
                    .w(px(w * cell))
                    .h(px(h * cell))
                    .bg(accent)
            }))
    }
}

/// The launch header: mascot, version, plan and cwd, then notices and tips.
#[derive(IntoElement)]
pub struct ClaudeCodeHeader {
    header: Header,
}

impl ClaudeCodeHeader {
    pub fn new(header: Header) -> Self {
        Self { header }
    }
}

impl RenderOnce for ClaudeCodeHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let h = self.header;
        let title = Rich::new()
            .push(Run::new("Claude Code", p.strong).bold())
            .push(Run::new(" ", p.fg))
            .push(Run::new(h.version, p.muted));
        let plan = h.plan();
        let line = |text: String| div().truncate().text_color(p.muted).child(text);
        let notices = !h.whats_new.is_empty() || !h.tips.is_empty();
        div()
            .flex()
            .flex_col()
            .gap(px(GAP))
            .min_w_0()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(cells(2.0))
                    .min_w_0()
                    .child(ClaudeCodeLogo::default())
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w_0()
                            .child(div().truncate().child(rich_text(&title)))
                            .when(!plan.is_empty(), |d| d.child(line(plan)))
                            .when(!h.cwd.is_empty(), |d| d.child(line(h.cwd.into()))),
                    ),
            )
            .when(notices, |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .min_w_0()
                        .pl(cells(2.0))
                        .children(
                            h.whats_new
                                .iter()
                                .map(|l| rich_text(&model::rich(l, p.fg, &p))),
                        )
                        .children(h.tips.iter().map(|l| {
                            div()
                                .flex()
                                .gap(cells(2.0))
                                .text_color(p.muted)
                                .child(div().flex_none().child(RESULT_GLYPH))
                                .child(
                                    div()
                                        .min_w_0()
                                        .child(rich_text(&model::rich(l, p.muted, &p))),
                                )
                        })),
                )
            })
    }
}

/// A prompt (`❯` on a band) or a reply (`●`), in the inline grammar.
#[derive(IntoElement)]
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
}

impl RenderOnce for ClaudeCodeMessage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let (glyph, glyph_color, text_color) = if self.user {
            ("❯", p.user_prompt, p.strong)
        } else {
            ("●", p.strong, p.fg)
        };
        gutter_row(
            2.0,
            div().text_color(glyph_color).child(glyph),
            rich_text(&model::rich(&self.text, text_color, &p)),
        )
        .when(self.user, |d| d.bg(p.user_bg))
    }
}

/// Open state and pointer of a collapsible trigger.
struct Disclosure {
    open: bool,
    focus: FocusHandle,
    hover: Animated,
}

fn disclosure(
    id: &ElementId,
    default_open: bool,
    window: &mut Window,
    cx: &mut App,
) -> Entity<Disclosure> {
    window.use_keyed_state(id.clone(), cx, |_, cx| Disclosure {
        open: default_open,
        focus: cx.focus_handle(),
        hover: Animated::new(0.0, model::HINT_FADE),
    })
}

/// A Base UI collapsible trigger: focusable, toggles on click, Enter and Space.
fn trigger(
    id: ElementId,
    state: &Entity<Disclosure>,
    p: &ClaudeCodePalette,
    cx: &App,
) -> ::gpui::Stateful<::gpui::Div> {
    let focus = state.read(cx).focus.clone();
    let toggle = state.clone();
    let hover = state.clone();
    div()
        .id(id)
        .track_focus(&focus)
        .tab_stop(true)
        .rounded(px(model::FOCUS_RADIUS))
        .cursor_pointer()
        .focus_visible(|s| {
            s.shadow(shadows(&[Shadow::drop(
                0.0,
                0.0,
                model::FOCUS_RING,
                p.muted.fade(0.6),
            )]))
        })
        .on_hover(move |hovered, _, cx| {
            hover.update(cx, |s, cx| {
                s.hover
                    .set(if *hovered { 1.0 } else { 0.0 }, Instant::now());
                cx.notify();
            });
        })
        .on_click(move |_, _, cx| {
            toggle.update(cx, |s, cx| {
                s.open = !s.open;
                cx.notify();
            });
        })
}

/// `∴ Thinking… (click to expand)` over the reasoning.
#[derive(IntoElement)]
pub struct ClaudeCodeThinking {
    id: ElementId,
    label: SharedString,
    body: Option<SharedString>,
    default_open: bool,
}

impl ClaudeCodeThinking {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: "Thinking…".into(),
            body: None,
            default_open: false,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }
}

impl RenderOnce for ClaudeCodeThinking {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let state = disclosure(&self.id, self.default_open, window, cx);
        let open = state.read(cx).open;
        let label = Rich::new()
            .push(Run::new(self.label.to_string(), p.muted).italic())
            .push(Run::new(
                if open { "" } else { " (click to expand)" },
                p.muted,
            ));
        div()
            .flex()
            .flex_col()
            .min_w_0()
            .text_color(p.muted)
            .child(
                trigger(self.id, &state, &p, cx)
                    .flex()
                    .child(div().w(cells(2.0)).flex_none().child("∴"))
                    .child(rich_text(&label)),
            )
            .when_some(self.body.filter(|_| open), |d, body| {
                d.child(div().pl(cells(2.0)).italic().child(body))
            })
    }
}

/// `● Update Todos` and the checklist under it.
#[derive(IntoElement)]
pub struct ClaudeCodeTodoList {
    todos: Vec<Todo>,
    heading: Option<SharedString>,
}

impl ClaudeCodeTodoList {
    pub fn new(todos: Vec<Todo>) -> Self {
        Self {
            todos,
            heading: Some("Update Todos".into()),
        }
    }

    /// The tool line above the list; `None` leaves it out.
    pub fn heading(mut self, heading: Option<SharedString>) -> Self {
        self.heading = heading;
        self
    }
}

impl RenderOnce for ClaudeCodeTodoList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let has_heading = self.heading.is_some();
        div()
            .flex()
            .flex_col()
            .min_w_0()
            .when_some(self.heading, |d, heading| {
                d.child(gutter_row(
                    2.0,
                    div().text_color(p.success).child("●"),
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_color(p.strong)
                        .child(heading),
                ))
            })
            .child(div().flex().flex_col().pl(cells(2.0)).children(
                self.todos.into_iter().enumerate().map(|(i, todo)| {
                    let corner = if i == 0 && has_heading {
                        RESULT_GLYPH
                    } else {
                        ""
                    };
                    div()
                        .flex()
                        .min_w_0()
                        .child(
                            div()
                                .w(cells(3.0))
                                .flex_none()
                                .text_color(p.muted)
                                .child(corner),
                        )
                        .child(
                            div()
                                .w(cells(2.0))
                                .flex_none()
                                .flex()
                                .justify_center()
                                .text_color(todo.status.glyph_color(&p))
                                .child(todo.status.glyph()),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .pl(cells(1.0))
                                .child(rich_text(&Rich(vec![todo.status.text(&todo.label, &p)]))),
                        )
                }),
            ))
    }
}

/// A tool call: `● Tool(arg)` and `⎿ result`, its detail behind a click.
#[derive(IntoElement)]
pub struct ClaudeCodeToolCall {
    id: ElementId,
    call: ToolCall,
}

impl ClaudeCodeToolCall {
    pub fn new(id: impl Into<ElementId>, call: ToolCall) -> Self {
        Self {
            id: id.into(),
            call,
        }
    }
}

impl RenderOnce for ClaudeCodeToolCall {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let call = self.call;
        let bullet_opacity = if call.status == ToolStatus::Pending {
            pulse_opacity((self.id.clone(), "pulse"), window, cx)
        } else {
            1.0
        };
        let headline = gutter_row(
            2.0,
            div()
                .text_color(call.status.bullet(&p))
                .opacity(bullet_opacity)
                .child("●"),
            rich_text(&call.headline(&p)),
        );
        let result_row = |hint: bool| {
            div()
                .flex()
                .w_full()
                .min_w_0()
                .pl(cells(2.0))
                .child(div().w(cells(3.0)).flex_none().child(RESULT_GLYPH))
                .child(div().flex_1().min_w_0().child(format!(
                    "{}{}",
                    call.result,
                    if hint { " (click to expand)" } else { "" }
                )))
        };
        let mut root = div().flex().flex_col().min_w_0().child(headline);
        match call.detail.clone() {
            Some(detail) => {
                let state = disclosure(&self.id, false, window, cx);
                let open = state.read(cx).open;
                root = root
                    .child(
                        trigger(self.id, &state, &p, cx)
                            .text_color(p.muted)
                            .hover(|s| s.text_color(p.fg))
                            .child(result_row(!open)),
                    )
                    .when(open, |d| {
                        d.child(div().pl(cells(5.0)).text_color(p.muted).child(detail))
                    });
            }
            None => root = root.child(result_row(false).text_color(p.muted)),
        }
        root
    }
}

/// `Ran 3 shell commands`, folding the calls under it.
#[derive(IntoElement)]
pub struct ClaudeCodeToolSummary {
    id: ElementId,
    calls: Vec<ToolCall>,
    verb: SharedString,
    noun: SharedString,
}

impl ClaudeCodeToolSummary {
    pub fn new(id: impl Into<ElementId>, calls: Vec<ToolCall>) -> Self {
        Self {
            id: id.into(),
            calls,
            verb: "Ran".into(),
            noun: "shell command".into(),
        }
    }
}

impl RenderOnce for ClaudeCodeToolSummary {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let count = self.calls.len();
        let noun = if count == 1 {
            self.noun.to_string()
        } else {
            format!("{}s", self.noun)
        };
        let state = disclosure(&self.id, false, window, cx);
        let (open, focused, hint) = {
            let s = state.read(cx);
            let now = Instant::now();
            if s.hover.is_animating(now) {
                window.request_animation_frame();
            }
            (
                s.open,
                s.focus.contains_focused(window, cx),
                s.hover.value(now),
            )
        };
        let text = format!("{} {count} {noun}", self.verb);
        let bold = self.verb.len() + 1..self.verb.len() + 1 + count.to_string().len();
        let summary = StyledText::new(text).with_highlights([(
            bold,
            HighlightStyle {
                font_weight: Some(FontWeight::BOLD),
                ..Default::default()
            },
        )]);
        let id = self.id.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(GAP))
            .pl(cells(2.0))
            .text_color(p.muted)
            .child(
                div().flex().child(
                    trigger(self.id, &state, &p, cx)
                        .flex()
                        .hover(|s| s.text_color(p.fg))
                        .child(summary)
                        .when(!open, |d| {
                            d.child(
                                div()
                                    .opacity(if focused { 1.0 } else { hint })
                                    .child(" (click to expand)"),
                            )
                        }),
                ),
            )
            .when(open, |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(GAP))
                        .ml(-cells(2.0))
                        .children(self.calls.into_iter().map(|call| {
                            ClaudeCodeToolCall::new(
                                (id.clone(), SharedString::from(call.id.clone())),
                                call,
                            )
                        })),
                )
            })
    }
}

/// The spinner line while a turn runs.
#[derive(IntoElement)]
pub struct ClaudeCodeSpinner {
    id: ElementId,
    spinner: Spinner,
    paused: bool,
}

impl ClaudeCodeSpinner {
    pub fn new(id: impl Into<ElementId>, spinner: Spinner) -> Self {
        Self {
            id: id.into(),
            spinner,
            paused: false,
        }
    }

    /// Freezes the glyph, for screenshots.
    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = paused;
        self
    }
}

impl RenderOnce for ClaudeCodeSpinner {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let s = self.spinner;
        let mut line = Rich::new().push(Run::new(format!("{}…", s.verb), p.accent));
        if let Some(meta) = s.meta() {
            line = line.push(Run::new(format!(" ({meta})"), p.muted));
        }
        div()
            .flex()
            .flex_col()
            .min_w_0()
            .text_color(p.muted)
            .child(gutter_row(
                2.0,
                div()
                    .text_color(p.accent)
                    .child(FrameGlyph::new(self.id, &SPINNER_FRAMES, SPINNER).paused(self.paused)),
                rich_text(&line),
            ))
            .when_some(s.tip, |d, tip| {
                d.child(
                    gutter_row(
                        3.0,
                        RESULT_GLYPH,
                        rich_text(&model::rich(&tip, p.muted, &p)),
                    )
                    .pl(cells(2.0)),
                )
            })
    }
}

/// `✻ Worked for 19s · done 9:42 AM`.
#[derive(IntoElement)]
pub struct ClaudeCodeTurnSummary {
    summary: TurnSummary,
}

impl ClaudeCodeTurnSummary {
    pub fn new(summary: TurnSummary) -> Self {
        Self { summary }
    }
}

impl RenderOnce for ClaudeCodeTurnSummary {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        gutter_row(2.0, "✻", self.summary.text()).text_color(p.muted)
    }
}

/// `⎿ Interrupted · What should Claude do instead?`
#[derive(IntoElement)]
pub struct ClaudeCodeInterrupted {
    hint: Option<SharedString>,
}

impl ClaudeCodeInterrupted {
    pub fn new() -> Self {
        Self {
            hint: Some("What should Claude do instead?".into()),
        }
    }

    pub fn hint(mut self, hint: Option<SharedString>) -> Self {
        self.hint = hint;
        self
    }
}

impl Default for ClaudeCodeInterrupted {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ClaudeCodeInterrupted {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let mut line = Rich::new().push(Run::new("Interrupted", p.error));
        if let Some(hint) = self.hint {
            line = line.push(Run::new(format!(" · {hint}"), p.muted));
        }
        gutter_row(3.0, RESULT_GLYPH, rich_text(&line))
            .pl(cells(2.0))
            .text_color(p.muted)
    }
}

/// The prompt: a ruled input line and the mode and effort status line.
#[derive(IntoElement)]
pub struct ClaudeCodePrompt {
    id: ElementId,
    input: Entity<PromptInput>,
    placeholder: SharedString,
    mode: ClaudeCodeMode,
    effort: Option<ClaudeCodeEffort>,
    pull_request: Option<u32>,
}

impl ClaudeCodePrompt {
    /// Reads and types into `input`; subscribe to its [`PromptEvent`]s to send.
    pub fn new(id: impl Into<ElementId>, input: Entity<PromptInput>) -> Self {
        Self {
            id: id.into(),
            input,
            placeholder: SharedString::default(),
            mode: ClaudeCodeMode::Manual,
            effort: None,
            pull_request: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
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
}

impl RenderOnce for ClaudeCodePrompt {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = palette(cx);
        let (text, focused) = {
            let input = self.input.read(cx);
            (input.text().to_owned(), input.is_focused(window))
        };
        let field = if text.is_empty() {
            let (block, ink) = p.cursor();
            let mut chars = self.placeholder.chars();
            let first = chars.next().map(String::from).unwrap_or_else(|| " ".into());
            div()
                .relative()
                .min_w_0()
                .text_color(p.muted)
                .child(div().truncate().child(self.placeholder.clone()))
                .child(
                    div()
                        .absolute()
                        .left_0()
                        .top_0()
                        .bottom_0()
                        .w(cells(1.0))
                        .bg(block)
                        .text_color(ink)
                        .child(first),
                )
        } else {
            div()
                .flex()
                .min_w_0()
                .text_color(p.strong)
                .child(div().truncate().child(text))
                .when(focused, |d| {
                    d.child(
                        div()
                            .w(px(1.0))
                            .h(px(LINE_HEIGHT - 4.0))
                            .mt(px(2.0))
                            .bg(p.strong),
                    )
                })
        };
        let field = bind_prompt(
            div()
                .id(self.id)
                .flex()
                .min_w_0()
                .py(px(model::PROMPT_PADDING_Y))
                .border_y_1()
                .border_color(p.rule)
                .cursor_text()
                .child(
                    div()
                        .w(cells(2.0))
                        .flex_none()
                        .text_color(p.strong)
                        .child("❯"),
                )
                .child(div().flex_1().min_w_0().child(field)),
            &self.input,
            cx,
        );
        let mut status = Rich::new().push(Run::new(self.mode.label(), self.mode.color(&p)));
        if self.mode.cycles() {
            status = status.push(Run::new(" (shift+tab to cycle)", p.muted));
        }
        if let Some(number) = self.pull_request {
            status = status
                .push(Run::new(" · PR ", p.muted))
                .push(Run::new(format!("#{number}"), p.pr).underline());
        }
        div()
            .flex()
            .flex_col()
            .gap(px(GAP))
            .min_w_0()
            .child(field)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_between()
                    .gap_x(cells(2.0))
                    .pl(cells(2.0))
                    .min_w_0()
                    .child(div().min_w_0().child(rich_text(&status)))
                    .when_some(self.effort, |d, effort| {
                        d.child(
                            div()
                                .flex_none()
                                .text_color(effort.color(&p))
                                .child(format!("{} {} · /effort", effort.glyph(), effort.name())),
                        )
                    }),
            )
    }
}

/// One exchange: prompt, todos, tool calls (folded when all shell), reply, status.
#[derive(IntoElement)]
pub struct ClaudeCodeTurn {
    turn: Turn,
}

impl ClaudeCodeTurn {
    pub fn new(turn: Turn) -> Self {
        Self { turn }
    }
}

impl RenderOnce for ClaudeCodeTurn {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let turn = self.turn;
        let id = SharedString::from(turn.id.clone());
        let folds = turn.folds_shell();
        let calls = turn.commands.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(GAP))
            .child(ClaudeCodeMessage::user(turn.prompt.clone()))
            .when(!turn.todos.is_empty(), |d| {
                d.child(ClaudeCodeTodoList::new(turn.todos.clone()))
            })
            .map(|d| {
                if folds {
                    d.child(ClaudeCodeToolSummary::new(
                        ElementId::Name(format!("{id}-summary").into()),
                        calls,
                    ))
                } else {
                    d.children(calls.into_iter().map(|call| {
                        ClaudeCodeToolCall::new(
                            ElementId::Name(format!("{id}-{}", call.id).into()),
                            call,
                        )
                    }))
                }
            })
            .when(!turn.answer.is_empty(), |d| {
                d.child(ClaudeCodeMessage::assistant(turn.answer.clone()))
            })
            .when(turn.status == TurnStatus::Interrupted, |d| {
                d.child(ClaudeCodeInterrupted::new())
            })
            .when_some(turn.summary, |d, s| d.child(ClaudeCodeTurnSummary::new(s)))
    }
}

/// The playable demo: the session transcript, a prompt that sends scripted replies,
/// Escape to interrupt and Shift+Tab to cycle the mode.
pub struct ClaudeCodeDemo {
    chat: model::Chat,
    input: Entity<PromptInput>,
    mode: ClaudeCodeMode,
    scroll: ScrollHandle,
    epoch: Instant,
    follow: bool,
    shown: usize,
    _events: Subscription,
}

impl ClaudeCodeDemo {
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
                PromptEvent::CycleMode => this.mode = this.mode.next(),
            }
            cx.notify();
        });
        Self {
            chat: model::Chat::demo(),
            input,
            mode: ClaudeCodeMode::Bypass,
            scroll: ScrollHandle::new(),
            epoch: Instant::now(),
            follow: false,
            shown: 0,
            _events: events,
        }
    }

    /// Whether the transcript sits within 48 px of its end, as the site's follow check;
    /// `None` before the first layout.
    fn near_bottom(&self) -> Option<bool> {
        if self.scroll.bounds().size.height <= ::gpui::px(0.0) {
            return None;
        }
        let max = f32::from(self.scroll.max_offset().y);
        let offset = -f32::from(self.scroll.offset().y);
        Some(max - offset < 48.0)
    }
}

impl Render for ClaudeCodeDemo {
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
        let shown = view
            .turns
            .iter()
            .map(|t| t.answer.len() + t.commands.len() * 7 + 1)
            .sum::<usize>()
            + usize::from(view.spinner.is_some());
        if !self.chat.is_busy()
            && let Some(near) = self.near_bottom()
        {
            self.follow = near;
        }
        if shown != self.shown {
            self.shown = shown;
            if self.follow {
                self.scroll.scroll_to_bottom();
            }
        }
        ClaudeCodeTerminal::new("claude-code-demo")
            .title(session::TITLE)
            .height(model::DEMO_HEIGHT)
            .track_scroll(&self.scroll)
            .footer(
                ClaudeCodePrompt::new("claude-code-demo-prompt", self.input.clone())
                    .placeholder(session::PLACEHOLDER)
                    .mode(self.mode)
                    .pull_request(Some(session::PULL_REQUEST)),
            )
            .child(ClaudeCodeHeader::new(session::HEADER))
            .children(view.turns.into_iter().map(ClaudeCodeTurn::new))
            .children(
                view.spinner
                    .map(|s| ClaudeCodeSpinner::new("claude-code-demo-spinner", s)),
            )
    }
}
