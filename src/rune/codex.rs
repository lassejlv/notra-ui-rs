//! Codex for Rune: the Codex CLI session block's parts and the playable demo.

use std::rc::Rc;
use std::time::Duration;

use ::rune::tok::TextStyle;
use ::rune::{El, Key, KeyEvent, TextEdit, Tx, col, el, icon, row, text, text_input};
use ::rune_motion::{Beat, Easing, LoopSpec, Sequence};

use super::ToRune;
use super::terminal::{
    Disclosures, TerminalWindow, code_font, disclosure_trigger, pulse, rich_text,
};
use crate::components::codex::{
    self as model, CodexPalette, Exec, ExecStatus, ExploredItem, FONT_SIZE, GAP, HALF_GAP,
    LINE_HEIGHT, STATUS_GAP, Turn, TurnStatus, session,
};
use crate::components::terminal::{CH_EM, Rich, Run, ch};
use crate::icons::Icon;
use crate::theme::{Color, Theme};

/// Which Codex collapsibles are open, owned by the app.
pub type CodexState = Disclosures;

fn palette(theme: &Theme) -> CodexPalette {
    CodexPalette::new(theme.appearance)
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

/// `grid-cols-[2ch_minmax(0,1fr)] px-[1ch]`: the gutter and the content column.
fn codex_row(lead: El, content: El) -> El {
    row()
        .w_full()
        .px(cells(1.0))
        .child(el().w(cells(2.0)).child(lead))
        .child(content.grow(1.0).shrink().min_w(0.0))
}

fn tree_line(lead: El, content: El) -> El {
    row()
        .w_full()
        .child(el().w(cells(2.0)).child(lead))
        .child(content.grow(1.0).shrink().min_w(0.0))
}

fn trigger<S: 'static>(
    id: &str,
    open: bool,
    p: &CodexPalette,
    lens: impl Fn(&mut S) -> &mut CodexState + Copy + 'static,
) -> El {
    disclosure_trigger(
        id,
        open,
        p.blue.fade(0.6),
        model::FOCUS_RING,
        model::FOCUS_RADIUS,
        lens,
    )
}

fn trigger_label(label: impl Into<String>, open: bool, p: &CodexPalette) -> El {
    text(label)
        .font(font())
        .color(if open { p.fg } else { p.muted }.rune())
        .group_color(p.fg.rune())
}

/// The terminal window: chrome, the scrolling session and an optional footer.
pub struct CodexTerminal {
    key: String,
    title: Option<String>,
    height: Option<f32>,
    children: Vec<El>,
    footer: Option<El>,
    follow: bool,
}

impl CodexTerminal {
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

    /// Pinned under the session: the working line and the composer.
    pub fn footer(mut self, footer: El) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Keeps the session at its end while output arrives.
    pub fn follow(mut self, follow: bool) -> Self {
        self.follow = follow;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        let body = col().w_full().gap(GAP).py(GAP).children(self.children);
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
            window = window.footer(col().w_full().pb(HALF_GAP).child(footer));
        }
        window.build()
    }
}

/// `>_ OpenAI Codex (v…)`, the cwd and a greeting.
pub struct CodexHeader {
    version: String,
    cwd: String,
    greeting: Option<String>,
}

impl CodexHeader {
    pub fn new() -> Self {
        Self {
            version: session::VERSION.into(),
            cwd: session::CWD.into(),
            greeting: Some(session::GREETING.into()),
        }
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = cwd.into();
        self
    }

    pub fn greeting(mut self, greeting: Option<String>) -> Self {
        self.greeting = greeting;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        let title = Rich::new()
            .push(Run::new(">_ ", p.blue))
            .push(Run::new("OpenAI Codex", p.fg).bold())
            .push(Run::new(format!(" (v{})", self.version), p.muted));
        let mut lines = col().child(rich_text(&title, font())).child(
            glyph(&self.cwd, p.muted)
                .single_line()
                .truncate()
                .pl(cells(3.0)),
        );
        if let Some(greeting) = self.greeting {
            lines = lines.child(glyph(&greeting, p.blue).wrap().mt(GAP));
        }
        codex_row(el(), lines)
    }
}

impl Default for CodexHeader {
    fn default() -> Self {
        Self::new()
    }
}

/// A prompt on its band (`›`) or a reply (`•`) whose paragraphs sit 1.3em apart.
pub struct CodexMessage {
    user: Option<String>,
    children: Vec<El>,
}

impl CodexMessage {
    pub fn user(text: impl Into<String>) -> Self {
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

    pub fn child(mut self, child: El) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = El>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        match self.user {
            Some(t) => codex_row(glyph("›", p.dim), glyph(&t, p.fg).wrap())
                .py(GAP)
                .fill(p.band.rune()),
            None => codex_row(glyph("•", p.muted), col().gap(GAP).children(self.children)),
        }
    }
}

/// A reply paragraph: ``code`` in green.
pub fn codex_paragraph(text: &str, theme: &Theme) -> El {
    rich_text(&model::rich(text, &palette(theme)), font())
}

/// The model's reasoning summary: bold italic and muted.
pub struct CodexReasoning {
    text: String,
}

impl CodexReasoning {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        codex_row(
            glyph("•", p.muted),
            rich_text(
                &Rich(vec![Run::new(self.text, p.muted).bold().italic()]),
                font(),
            ),
        )
    }
}

/// `• Ran <command>` with its output, the rest behind `+ N lines`.
pub struct CodexExec {
    exec: Exec,
}

impl CodexExec {
    pub fn new(exec: Exec) -> Self {
        Self { exec }
    }

    pub fn build<S: 'static>(
        self,
        theme: &Theme,
        state: &CodexState,
        lens: impl Fn(&mut S) -> &mut CodexState + Copy + 'static,
    ) -> El {
        let p = palette(theme);
        let exec = self.exec;
        let mut bullet = glyph("•", exec.status.bullet(&p)).key(format!("{}-bullet", exec.id));
        if exec.status == ExecStatus::Running {
            bullet = bullet.looping(pulse());
        }
        let headline = Rich::new()
            .push(Run::new(exec.status.label(), p.fg).bold())
            .push(Run::new(" ", p.fg))
            .extend(model::shell_line(&exec.command, &p));
        let (preview, rest) = exec.split_output();
        let mut body = col().child(rich_text(&headline, font()).single_line().truncate());
        if !preview.is_empty() {
            body = body.child(tree_line(
                glyph("└", p.muted),
                glyph(&preview.join("\n"), p.muted).wrap(),
            ));
            if !rest.is_empty() {
                let open = state.is_open(&exec.id);
                if open {
                    body = body.child(tree_line(el(), glyph(&rest.join("\n"), p.muted).wrap()));
                }
                let label = if open {
                    "− collapse".to_owned()
                } else {
                    format!("+ {} lines (click to expand)", rest.len())
                };
                body = body.child(tree_line(
                    el(),
                    row().child(
                        trigger(&exec.id, open, &p, lens).child(trigger_label(label, open, &p)),
                    ),
                ));
            }
        }
        codex_row(bullet, body).key(exec.id.clone())
    }
}

/// `• Explored` (or `Exploring`) and what was read, searched and listed.
pub struct CodexExplored {
    id: String,
    items: Vec<ExploredItem>,
    details: Vec<ExploredItem>,
    active: bool,
}

impl CodexExplored {
    pub fn new(id: impl Into<String>, items: Vec<ExploredItem>) -> Self {
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

    pub fn build<S: 'static>(
        self,
        theme: &Theme,
        state: &CodexState,
        lens: impl Fn(&mut S) -> &mut CodexState + Copy + 'static,
    ) -> El {
        let p = palette(theme);
        let line = |item: &ExploredItem, first: bool| {
            tree_line(
                glyph(if first { "└" } else { "" }, p.muted),
                rich_text(&item.line(&p), font()).single_line().truncate(),
            )
        };
        let mut body = col()
            .child(rich_text(
                &Rich(vec![
                    Run::new(if self.active { "Exploring" } else { "Explored" }, p.fg).bold(),
                ]),
                font(),
            ))
            .children(
                self.items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| line(item, i == 0)),
            );
        if !self.details.is_empty() {
            let open = state.is_open(&self.id);
            if open {
                body = body.children(self.details.iter().map(|item| line(item, false)));
            }
            body = body.child(
                row()
                    .pl(cells(2.0))
                    .child(trigger(&self.id, open, &p, lens).child(trigger_label(
                        if open {
                            "− Hide details"
                        } else {
                            "+ Show details"
                        },
                        open,
                        &p,
                    ))),
            );
        }
        let mut bullet = glyph("•", p.muted).key(format!("{}-bullet", self.id));
        if self.active {
            bullet = bullet.looping(pulse());
        }
        codex_row(bullet, body).key(self.id)
    }
}

const BLINK_BEATS: &[Beat] = &[
    Beat::new(0.0).opacity(1.0).easing(Easing::Linear),
    Beat::new(0.6).opacity(0.35).easing(Easing::Linear),
    Beat::new(1.2).opacity(1.0).easing(Easing::Linear),
];

/// `• Working (13s • esc to interrupt)` with the shimmering label and a blinking bullet.
pub struct CodexWorking {
    key: String,
    label: String,
    elapsed: String,
}

impl CodexWorking {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: "Working".into(),
            elapsed: "0s".into(),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn elapsed(mut self, elapsed: impl Into<String>) -> Self {
        self.elapsed = elapsed.into();
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        let key = self.key;
        let width = self.label.chars().count() as f32 * CH_EM * FONT_SIZE;
        let (start, _, end) = model::SHIMMER_STOPS;
        let band = (end - start) * 3.0 * width;
        let period = model::SHIMMER.period.as_secs_f64();
        let travel =
            |a: f32, b: f32| LoopSpec::slide(period, [a, 0.0], [b, 0.0]).easing(Easing::LINEAR);
        let left_start = -0.5 * width - band / 2.0;
        let left_end = 1.5 * width - band / 2.0;
        let lit = el()
            .key(format!("{key}-lit"))
            .absolute(0.0, 0.0)
            .w(band)
            .h_full()
            .looping(travel(-left_start / band, -left_end / band))
            .child(glyph(&self.label, p.fg).absolute(0.0, 0.0));
        let sheen = el()
            .key(format!("{key}-sheen"))
            .absolute(0.0, 0.0)
            .w(band)
            .h_full()
            .clip_fade_linear(band / 2.0, 0.0, band / 2.0, 0.0)
            .looping(travel(left_start / band, left_end / band))
            .child(lit);
        let label = el()
            .key(format!("{key}-label"))
            .child(glyph(&self.label, p.dim))
            .child(sheen);
        let rest = Rich::new()
            .push(Run::new(format!(" ({} • ", self.elapsed), p.dim))
            .push(Run::new("esc", p.fg).bold())
            .push(Run::new(" to interrupt)", p.dim));
        codex_row(
            glyph("•", p.fg)
                .key(format!("{key}-bullet"))
                .looping(LoopSpec::sequence(Sequence::new(BLINK_BEATS))),
            row().child(label).child(
                rich_text(&rest, font())
                    .single_line()
                    .truncate()
                    .shrink()
                    .min_w(0.0),
            ),
        )
        .key(key)
        .role("status")
    }
}

/// `■ Conversation interrupted…` in red.
pub struct CodexInterrupted {
    text: String,
}

impl CodexInterrupted {
    pub fn new() -> Self {
        Self {
            text: session::INTERRUPTED.into(),
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        codex_row(glyph("■", p.red), glyph(&self.text, p.red).wrap()).role("status")
    }
}

impl Default for CodexInterrupted {
    fn default() -> Self {
        Self::new()
    }
}

/// A bulleted list inside a reply.
pub struct CodexList {
    items: Vec<Rich>,
}

impl CodexList {
    pub fn new(items: Vec<Rich>) -> Self {
        Self { items }
    }

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        col().role("list").children(
            self.items
                .iter()
                .map(|item| tree_line(glyph("•", p.fg), rich_text(item, font()))),
        )
    }
}

/// A table inside a reply: yellow headers over a 2 px rule, 1 px rules between rows.
pub struct CodexTable {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    code_columns: Vec<usize>,
}

impl CodexTable {
    pub fn new(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
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

    pub fn build(self, theme: &Theme) -> El {
        let p = palette(theme);
        let last = self.rows.len().saturating_sub(1);
        let columns = (0..self.headers.len()).map(|c| {
            let longest = std::iter::once(&self.headers[c])
                .chain(self.rows.iter().filter_map(|r| r.get(c)))
                .map(|s| s.chars().count())
                .max()
                .unwrap_or(0);
            let header = rich_text(
                &Rich(vec![Run::new(self.headers[c].clone(), p.yellow).bold()]),
                font(),
            )
            .single_line()
            .px(cells(1.0))
            .pb(HALF_GAP)
            .border_sides([0.0, 0.0, 2.0, 0.0], p.rule.rune());
            let code = self.code_columns.contains(&c);
            let body = self.rows.iter().enumerate().map(|(r, row)| {
                let cell = glyph(
                    row.get(c).map_or("", String::as_str),
                    if code { p.green } else { p.fg },
                )
                .single_line()
                .px(cells(1.0))
                .py(HALF_GAP);
                if r < last {
                    cell.border_sides([0.0, 0.0, 1.0, 0.0], p.rule.rune())
                } else {
                    cell
                }
            });
            col()
                .w(cells(longest as f32 + 2.0))
                .child(header)
                .children(body)
        });
        row()
            .ml(-cells(2.0))
            .pl(cells(2.0))
            .gap(cells(2.0))
            .role("table")
            .children(columns)
    }
}

type KeyHandler<S> = Box<dyn Fn(&mut S, &KeyEvent, &mut Tx) -> bool>;
type SubmitHandler<S> = Box<dyn Fn(&mut S, &mut Tx)>;

/// The composer: the prompt band and the model, cwd and shortcut lines under it.
pub struct CodexComposer<S> {
    key: String,
    draft: TextEdit,
    edit: Rc<dyn Fn(&mut S) -> &mut TextEdit>,
    placeholder: String,
    model: String,
    effort: Option<String>,
    cwd: String,
    task: Option<String>,
    warnings: Option<u32>,
    context: Option<String>,
    submit: Option<SubmitHandler<S>>,
    keys: Option<KeyHandler<S>>,
}

impl<S: 'static> CodexComposer<S> {
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
            placeholder: session::PLACEHOLDER.into(),
            model: session::MODEL.into(),
            effort: None,
            cwd: session::CWD.into(),
            task: None,
            warnings: None,
            context: None,
            submit: None,
            keys: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn effort(mut self, effort: impl Into<String>) -> Self {
        self.effort = Some(effort.into());
        self
    }

    pub fn cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = cwd.into();
        self
    }

    pub fn task(mut self, task: impl Into<String>) -> Self {
        self.task = Some(task.into());
        self
    }

    pub fn warnings(mut self, warnings: u32) -> Self {
        self.warnings = Some(warnings);
        self
    }

    /// Shown on the right when there are no warnings.
    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Return with text in the field.
    pub fn on_submit(mut self, f: impl Fn(&mut S, &mut Tx) + 'static) -> Self {
        self.submit = Some(Box::new(f));
        self
    }

    /// Keys the field leaves alone (Escape); true consumes one.
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
            .label(self.placeholder.clone())
            .color(p.fg.rune())
            .caret(
                if empty {
                    ::rune::Color::TRANSPARENT
                } else {
                    p.cursor.rune()
                },
                p.blue.fade(0.3).rune(),
            )
            .on_edit(move |s: &mut S| edit(s))
            .w_full();
        if let Some(submit) = self.submit {
            input = input.on_submit(move |s: &mut S, tx: &mut Tx| submit(s, tx));
        }
        let mut field = el().grow(1.0).shrink().min_w(0.0).child(input);
        if empty {
            let mut chars = self.placeholder.chars();
            let first = chars.next().map_or(" ".to_owned(), String::from);
            let rest: String = chars.collect();
            field = field.child(
                row()
                    .pin(Some(0.0), Some(0.0), None, None)
                    .child(glyph(&first, p.fg).fill(p.cursor.rune()))
                    .child(glyph(&rest, p.placeholder).single_line().truncate()),
            );
        }
        let mut band = row()
            .key(self.key.clone())
            .w_full()
            .px(cells(1.0))
            .py(GAP)
            .fill(p.composer.rune())
            .child(
                el().w(cells(2.0))
                    .child(rich_text(&Rich(vec![Run::new("›", p.fg).bold()]), font())),
            )
            .child(field);
        if let Some(keys) = self.keys {
            band = band.on_key(move |s: &mut S, k: &KeyEvent, tx: &mut Tx| keys(s, k, tx));
        }
        let dot = || Run::new(" · ", p.dim);
        let key = |k: &str| Run::new(k, p.fg).bold();
        let model_label = match &self.effort {
            Some(effort) => format!("{} {effort}", self.model),
            None => self.model.clone(),
        };
        let mut first = Rich::new()
            .push(Run::new(model_label, p.yellow))
            .push(dot())
            .push(Run::new(self.cwd.clone(), p.green));
        if let Some(task) = &self.task {
            first = first.push(dot()).push(Run::new(task.clone(), p.yellow));
        }
        let shortcuts = Rich::new()
            .push(key("←"))
            .push(Run::new(" for agents", p.dim))
            .push(dot())
            .push(key("?"))
            .push(Run::new(" for shortcuts", p.dim));
        let right = match (self.warnings.filter(|&w| w > 0), &self.context) {
            (Some(w), _) => Some(
                row()
                    .center_y()
                    .child(
                        row()
                            .w(cells(1.0))
                            .justify_center()
                            .child(
                                icon(Icon::TriangleAlert.rune(), FONT_SIZE * 0.8)
                                    .color(p.amber.rune()),
                            )
                            .label("Warning"),
                    )
                    .child(rich_text(
                        &Rich::new()
                            .push(Run::new(
                                format!(" {w} {}", if w == 1 { "warning" } else { "warnings" }),
                                p.amber,
                            ))
                            .push(dot())
                            .push(key("f2"))
                            .push(Run::new(" to view", p.dim)),
                        font(),
                    )),
            ),
            (None, Some(context)) => Some(glyph(context, p.dim)),
            (None, None) => None,
        };
        col().w_full().child(band).child(
            codex_row(
                el(),
                col()
                    .child(rich_text(&first, font()).single_line().truncate())
                    .child(
                        row()
                            .w_full()
                            .flex_wrap()
                            .justify_between()
                            .gap(cells(2.0))
                            .child(rich_text(&shortcuts, font()))
                            .children(right),
                    ),
            )
            .mt(STATUS_GAP),
        )
    }
}

/// One exchange of the demo: prompt, commands, reply, interruption.
pub struct CodexTurn {
    turn: Turn,
}

impl CodexTurn {
    pub fn new(turn: Turn) -> Self {
        Self { turn }
    }

    pub fn build<S: 'static>(
        self,
        theme: &Theme,
        state: &CodexState,
        lens: impl Fn(&mut S) -> &mut CodexState + Copy + 'static,
    ) -> El {
        let turn = self.turn;
        let mut e = col()
            .key(turn.id.clone())
            .w_full()
            .gap(GAP)
            .child(CodexMessage::user(turn.prompt.clone()).build(theme))
            .children(
                turn.execs
                    .iter()
                    .cloned()
                    .map(|exec| CodexExec::new(exec).build(theme, state, lens)),
            );
        if !turn.reply.is_empty() {
            e = e.child(
                CodexMessage::assistant()
                    .children(
                        turn.paragraphs()
                            .into_iter()
                            .map(|t| codex_paragraph(t, theme)),
                    )
                    .build(theme),
            );
        }
        if turn.status == TurnStatus::Interrupted {
            e = e.child(CodexInterrupted::new().build(theme));
        }
        e
    }
}

/// The demo session's opening cells, before any playback.
pub fn codex_session<S: 'static>(
    theme: &Theme,
    state: &CodexState,
    lens: impl Fn(&mut S) -> &mut CodexState + Copy + 'static,
) -> Vec<El> {
    let p = palette(theme);
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
        .map(|row| row.iter().map(|c| (*c).to_owned()).collect())
        .collect();
    vec![
        CodexHeader::new().build(theme),
        CodexMessage::user(session::USER_MESSAGE).build(theme),
        CodexMessage::assistant()
            .child(codex_paragraph(session::INTRO, theme))
            .build(theme),
        CodexExec::new(Exec::new(
            "codex-session-exec",
            session::EXEC_COMMAND,
            Some(session::EXEC_OUTPUT),
            ExecStatus::Ran,
        ))
        .build(theme, state, lens),
        CodexExplored::new("codex-session-explored", session::EXPLORED.to_vec())
            .details(session::EXPLORED_DETAILS.to_vec())
            .build(theme, state, lens),
        CodexMessage::assistant()
            .child(codex_paragraph(session::SUMMARY, theme))
            .child(CodexList::new(highlights).build(theme))
            .child(codex_paragraph(session::TABLE_INTRO, theme))
            .child(
                CodexTable::new(
                    session::TABLE_HEADERS
                        .iter()
                        .map(|h| (*h).to_owned())
                        .collect(),
                    rows,
                )
                .code_columns(vec![0])
                .build(theme),
            )
            .child(codex_paragraph(session::FOLLOW_UP, theme))
            .build(theme),
    ]
}

const DEMO_KEY: &str = "codex-demo";

/// The playable demo's state, owned by the app; drive it from the app's `poll`.
pub struct CodexDemoState {
    pub chat: model::Chat,
    pub draft: TextEdit,
    pub disclosure: CodexState,
    /// Plays replies at the reduced-motion pace.
    pub reduced_motion: bool,
    now: Duration,
    follow: bool,
    shown: Option<model::ChatView>,
}

impl Default for CodexDemoState {
    fn default() -> Self {
        Self {
            chat: model::Chat::demo(),
            draft: TextEdit::new(""),
            disclosure: CodexState::default(),
            reduced_motion: false,
            now: Duration::ZERO,
            follow: false,
            shown: None,
        }
    }
}

impl CodexDemoState {
    /// Advances the playback to `clock`, asks for a wake at the next step and reports
    /// whether what's shown changed.
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

    /// Sends `prompt` at `now` on the app clock and follows the session.
    pub fn send_at(&mut self, prompt: &str, now: Duration) {
        self.now = now;
        self.chat
            .send(prompt, now, model::pace(self.reduced_motion));
        self.follow = true;
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

    /// Escape interrupts the running turn.
    pub fn key(&mut self, key: &KeyEvent, tx: &mut Tx) -> bool {
        if key.key == Key::Escape && self.chat.is_busy() {
            self.chat.stop(Duration::from_secs_f64(tx.clock().max(0.0)));
            return true;
        }
        false
    }
}

/// The playable demo: the session, a composer that sends scripted replies, the
/// working line while one plays and Escape to interrupt it.
pub struct CodexDemo;

impl CodexDemo {
    pub fn build<S: 'static>(
        theme: &Theme,
        state: &CodexDemoState,
        lens: fn(&mut S) -> &mut CodexDemoState,
    ) -> El {
        let view = state.chat.view(state.now);
        let mut footer = col().w_full();
        if let Some(secs) = view.working {
            footer = footer.child(
                CodexWorking::new(format!("{DEMO_KEY}-working"))
                    .elapsed(format!("{secs}s"))
                    .build(theme)
                    .pb(HALF_GAP),
            );
        }
        let composer = CodexComposer::new(
            format!("{DEMO_KEY}-composer"),
            &state.draft,
            move |s: &mut S| &mut lens(s).draft,
        )
        .effort(session::EFFORT)
        .task(session::TASK)
        .warnings(session::WARNINGS)
        .on_submit(move |s: &mut S, tx: &mut Tx| lens(s).send(tx))
        .on_key(move |s: &mut S, k: &KeyEvent, tx: &mut Tx| lens(s).key(k, tx));
        let session_cells = codex_session(theme, &state.disclosure, move |s: &mut S| {
            &mut lens(s).disclosure
        });
        let turns = view.turns.into_iter().map(|turn| {
            CodexTurn::new(turn).build(theme, &state.disclosure, move |s: &mut S| {
                &mut lens(s).disclosure
            })
        });
        CodexTerminal::new(DEMO_KEY)
            .title(session::TITLE)
            .height(model::DEMO_HEIGHT)
            .follow(state.follow)
            .footer(footer.child(composer.build(theme)))
            .children(session_cells)
            .children(turns)
            .build(theme)
    }
}
