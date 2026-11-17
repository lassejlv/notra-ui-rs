//! GPUI rendering of the code block.

use ::gpui::{
    AnyElement, App, ElementId, FontFeatures, HighlightStyle, IntoElement, ParentElement,
    RenderOnce, SharedString, StyledText, Window, div, prelude::*, px,
};

use super::data_parts::{CopyButton, CopyLabel, ease_on_hover, use_eased};
use super::{ActiveTheme, shadows};
use crate::components::code_block::{COPY_REVEAL, Language, highlight, labels, metrics};
use crate::components::data_parts::copy;
use crate::fonts;

/// Notra's code block: `CodeBlock::new("proxy", source).filename("proxy.ts")`, or
/// `.command()` for the one-line variant.
#[derive(IntoElement)]
pub struct CodeBlock {
    id: ElementId,
    code: SharedString,
    language: Language,
    filename: Option<SharedString>,
    tabs: Option<AnyElement>,
    header_end: Option<AnyElement>,
    command: bool,
    label: Option<SharedString>,
    wrap: bool,
}

impl CodeBlock {
    pub fn new(id: impl Into<ElementId>, code: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            language: Language::default(),
            filename: None,
            tabs: None,
            header_end: None,
            command: false,
            label: None,
            wrap: false,
        }
    }

    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// A file name on the shell, above the code.
    pub fn filename(mut self, filename: impl Into<SharedString>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Tabs on the shell (see [`super::data_parts::Tabs`]).
    pub fn tabs(mut self, tabs: impl IntoElement) -> Self {
        self.tabs = Some(tabs.into_any_element());
        self
    }

    /// Content at the end of the shell's header.
    pub fn header_end(mut self, end: impl IntoElement) -> Self {
        self.header_end = Some(end.into_any_element());
        self
    }

    /// The one-line command variant (an env var, a shell command).
    pub fn command(mut self) -> Self {
        self.command = true;
        self
    }

    /// The accessible name the copy button builds on (`Snippet`, `Command`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Wrap long lines instead of scrolling sideways.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }
}

impl RenderOnce for CodeBlock {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let id = self.id.clone();
        if self.command {
            let label = self.label.unwrap_or_else(|| labels::COMMAND.into());
            return div()
                .id(self.id)
                .flex()
                .items_center()
                .h(px(metrics::COMMAND_HEIGHT))
                .min_w_0()
                .pl(px(metrics::COMMAND_PADDING_START))
                .pr(px(metrics::COMMAND_PADDING_END))
                .rounded(px(metrics::COMMAND_RADIUS))
                .border_1()
                .border_color(theme.border.fade(0.6))
                .bg(theme.muted.fade(0.4))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .font_family(fonts::MONO)
                        .font_features(FontFeatures::disable_ligatures())
                        .text_size(px(metrics::CODE_SIZE))
                        .line_height(px(metrics::CODE_SIZE))
                        .text_color(theme.foreground)
                        .child(self.code.clone()),
                )
                .child(CopyButton::new(
                    SharedString::from(format!("{id}-copy")),
                    self.code,
                    copy::SNIPPET,
                    CopyLabel::Snippet(label),
                ))
                .into_any_element();
        }

        let syntax = theme.syntax;
        let lines = highlight(&self.code, self.language);
        let source = self.code.clone();
        let mut offset = 0;
        let rows = source.split('\n').zip(lines).map(|(line, tokens)| {
            let start = offset;
            offset += line.len() + 1;
            let highlights: Vec<_> = tokens
                .iter()
                .filter_map(|t| {
                    let color = t.kind.color(&syntax)?;
                    Some((
                        t.range.start - start..t.range.end - start,
                        HighlightStyle {
                            color: Some(color.into()),
                            ..Default::default()
                        },
                    ))
                })
                .collect();
            let text: SharedString = if line.is_empty() {
                " ".into()
            } else {
                SharedString::from(line.to_owned())
            };
            div()
                .when(!self.wrap, |d| d.whitespace_nowrap())
                .child(StyledText::new(text).with_highlights(highlights))
        });
        let (reveal_state, reveal) = use_eased(
            SharedString::from(format!("{id}-reveal")),
            COPY_REVEAL,
            window,
            cx,
        );
        let header = div()
            .flex()
            .items_center()
            .h(px(metrics::HEADER_HEIGHT))
            .min_w_0()
            .gap(px(metrics::HEADER_GAP))
            .pl(px(metrics::HEADER_PADDING_START))
            .pr(px(metrics::HEADER_PADDING_END))
            .map(|d| {
                let has_tabs = self.tabs.is_some();
                let has_name = self.filename.is_some();
                let d = d.children(self.tabs.map(|t| div().flex_1().min_w_0().child(t)));
                let d = d.children(self.filename.map(|name| {
                    div()
                        .truncate()
                        .font_family(fonts::MONO)
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(theme.muted_foreground)
                        .map(|f| {
                            if has_tabs {
                                f.flex_none().pr(px(metrics::FILENAME_PADDING_END))
                            } else {
                                f.flex_1().min_w_0()
                            }
                        })
                        .child(name)
                }));
                d.when(!has_tabs && !has_name, |d| d.child(div().flex_1()))
            })
            .children(self.header_end);
        let label = self.label.unwrap_or_else(|| labels::SNIPPET.into());
        let body = div()
            .id(SharedString::from(format!("{id}-body")))
            .relative()
            .min_w_0()
            .rounded(px(metrics::BODY_RADIUS))
            .border_1()
            .border_color(theme.table_card_border)
            .bg(theme.table_card)
            .shadow(shadows(theme.table_card_shadow))
            .on_hover(ease_on_hover(reveal_state))
            .child(
                div()
                    .id(SharedString::from(format!("{id}-code")))
                    .when(!self.wrap, |d| d.overflow_x_scroll())
                    .p(px(metrics::CODE_PADDING))
                    .pr(px(metrics::CODE_PADDING_END))
                    .font_family(fonts::MONO)
                    .font_features(FontFeatures::disable_ligatures())
                    .text_size(px(metrics::CODE_SIZE))
                    .line_height(px(metrics::CODE_LINE_HEIGHT))
                    .text_color(theme.foreground)
                    .children(rows),
            )
            .child(
                div()
                    .absolute()
                    .top(px(metrics::COPY_INSET))
                    .right(px(metrics::COPY_INSET))
                    .rounded(px(metrics::COPY_RADIUS))
                    .bg(theme.table_card)
                    .opacity(reveal)
                    .child(CopyButton::new(
                        SharedString::from(format!("{id}-copy")),
                        self.code.clone(),
                        copy::SNIPPET,
                        CopyLabel::Snippet(label),
                    )),
            );
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .min_w_0()
            .px(px(metrics::SHELL_PADDING))
            .pb(px(metrics::SHELL_PADDING))
            .rounded(px(metrics::SHELL_RADIUS))
            .border_1()
            .border_color(theme.shell_border)
            .bg(theme.shell)
            .child(header)
            .child(body)
            .into_any_element()
    }
}
