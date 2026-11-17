//! Rune rendering of the code block.

use ::rune::{El, Span, col, el, row, text};

use super::data_parts::{CopyButton, anim};
use super::package_command::NO_LIGATURES;
use super::{ToRune, mono};
use crate::components::code_block::{COPY_REVEAL, Language, highlight, labels, metrics};
use crate::components::data_parts::copy;
use crate::theme::Theme;

type CopyHook = Box<dyn FnOnce(CopyButton) -> CopyButton>;

/// Notra's code block: `CodeBlock::new(source).filename("proxy.ts").build(&theme)`, or
/// `.command()` for the one-line variant.
pub struct CodeBlock {
    code: String,
    language: Language,
    filename: Option<String>,
    tabs: Option<El>,
    header_end: Option<El>,
    command: bool,
    label: Option<String>,
    wrap: bool,
    copies: u32,
    on_copy: Option<CopyHook>,
}

impl CodeBlock {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            language: Language::default(),
            filename: None,
            tabs: None,
            header_end: None,
            command: false,
            label: None,
            wrap: false,
            copies: 0,
            on_copy: None,
        }
    }

    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    pub fn filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Tabs on the shell (see [`super::data_parts::Tabs`]).
    pub fn tabs(mut self, tabs: El) -> Self {
        self.tabs = Some(tabs);
        self
    }

    pub fn header_end(mut self, end: El) -> Self {
        self.header_end = Some(end);
        self
    }

    /// The one-line command variant.
    pub fn command(mut self) -> Self {
        self.command = true;
        self
    }

    /// The accessible name the copy button builds on (`Snippet`, `Command`).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// How many times the app has seen this block copied; each new count replays the tick.
    pub fn copies(mut self, copies: u32) -> Self {
        self.copies = copies;
        self
    }

    pub fn on_copy<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.on_copy = Some(Box::new(move |b: CopyButton| b.on_copy(f)));
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let label = self.label.clone().unwrap_or_else(|| {
            if self.command {
                labels::COMMAND
            } else {
                labels::SNIPPET
            }
            .to_owned()
        });
        let mut button = CopyButton::new(
            self.code.clone(),
            copy::SNIPPET,
            copy::snippet_label(copy::CopyStatus::Idle, &label),
        )
        .copies(self.copies);
        if let Some(hook) = self.on_copy {
            button = hook(button);
        }
        let button = button.build(theme).key("copy");
        if self.command {
            return row()
                .h(metrics::COMMAND_HEIGHT)
                .center_y()
                .pl(metrics::COMMAND_PADDING_START)
                .pr(metrics::COMMAND_PADDING_END)
                .radius(metrics::COMMAND_RADIUS)
                .border(1.0, theme.border.fade(0.6).rune())
                .fill(theme.muted.fade(0.4).rune())
                .child(
                    text(self.code.as_str())
                        .key("command")
                        .font(
                            mono(metrics::CODE_SIZE, 400.0, metrics::CODE_SIZE)
                                .features(NO_LIGATURES),
                        )
                        .color(theme.foreground.rune())
                        .label(label)
                        .selectable(true)
                        .grow(1.0)
                        .min_w(0.0)
                        .truncate(),
                )
                .child(button);
        }

        let syntax = theme.syntax;
        let font =
            mono(metrics::CODE_SIZE, 400.0, metrics::CODE_LINE_HEIGHT).features(NO_LIGATURES);
        let mut lines = col().key("code-lines");
        let mut offset = 0;
        for (i, (line, tokens)) in self
            .code
            .split('\n')
            .zip(highlight(&self.code, self.language))
            .enumerate()
        {
            let start = offset;
            offset += line.len() + 1;
            let spans: Vec<Span> = tokens
                .iter()
                .filter_map(|t| {
                    let color = t.kind.color(&syntax)?;
                    Some(Span::new(t.range.start - start, t.range.end - start).color(color.rune()))
                })
                .collect();
            let mut e = ::rune::rich(if line.is_empty() { " " } else { line }, spans)
                .key(format!("line-{i}"))
                .font(font)
                .color(theme.foreground.rune());
            if self.wrap {
                e = e.wrap_anywhere();
            }
            lines = lines.child(e);
        }
        let mut code = el()
            .key("code")
            .padding(metrics::CODE_PADDING)
            .pr(metrics::CODE_PADDING_END)
            .selectable(true)
            .child(lines);
        if !self.wrap {
            code = code.scroll_x();
        }
        let reveal = anim(COPY_REVEAL);
        let body = el()
            .key("code-body")
            .radius(metrics::BODY_RADIUS)
            .border(1.0, theme.table_card_border.rune())
            .fill(theme.table_card.rune())
            .hover_group()
            .child(code)
            .child(
                el().key("copy-slot")
                    .pin(
                        None,
                        Some(metrics::COPY_INSET),
                        Some(metrics::COPY_INSET),
                        None,
                    )
                    .radius(metrics::COPY_RADIUS)
                    .fill(theme.table_card.rune())
                    .group_opacity(0.0, 1.0)
                    .opacity_motion(reveal)
                    .child(button),
            );
        let body = theme
            .table_card_shadow
            .iter()
            .fold(body, |b, s| b.box_shadow(s.rune()));

        let has_tabs = self.tabs.is_some();
        let has_name = self.filename.is_some();
        let mut header = row()
            .key("code-header")
            .h(metrics::HEADER_HEIGHT)
            .center_y()
            .gap(metrics::HEADER_GAP)
            .pl(metrics::HEADER_PADDING_START)
            .pr(metrics::HEADER_PADDING_END);
        if let Some(tabs) = self.tabs {
            header = header.child(el().grow(1.0).min_w(0.0).child(tabs));
        }
        if let Some(name) = self.filename {
            let mut name = text(name)
                .key("filename")
                .font(mono(12.0, 400.0, 16.0))
                .color(theme.muted_foreground.rune())
                .truncate();
            name = if has_tabs {
                name.pr(metrics::FILENAME_PADDING_END)
            } else {
                name.grow(1.0).min_w(0.0)
            };
            header = header.child(name);
        }
        if !has_tabs && !has_name {
            header = header.child(el().grow(1.0));
        }
        if let Some(end) = self.header_end {
            header = header.child(end);
        }
        col()
            .min_w(0.0)
            .px(metrics::SHELL_PADDING)
            .pb(metrics::SHELL_PADDING)
            .radius(metrics::SHELL_RADIUS)
            .border(1.0, theme.shell_border.rune())
            .fill(theme.shell.rune())
            .child(header)
            .child(body)
    }
}
