use gpui::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, px,
};
use notra_ui::components::codex::{CodexPalette, session};
use notra_ui::components::terminal::{Rich, Run};
use notra_ui::gpui::{
    ActiveTheme, CodexComposer, CodexDemo, CodexExec, CodexExplored, CodexHeader, CodexList,
    CodexMessage, CodexReasoning, CodexTable, CodexTerminal, CodexWorking, PromptInput, rich_text,
};

use crate::{Page, section};

pub fn page(window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "codex",
        title: "Codex",
        view: cx.new(|cx| CodexPage::new(window, cx)).into(),
    }
}

struct CodexPage {
    demo: Entity<CodexDemo>,
    input: Entity<PromptInput>,
}

impl CodexPage {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            demo: cx.new(|cx| CodexDemo::new(window, cx)),
            input: cx.new(PromptInput::new),
        }
    }
}

fn terminal(id: &'static str) -> CodexTerminal {
    CodexTerminal::new(id).title(session::TITLE)
}

/// The examples sit inset on the page, as the docs frame them.
fn inset(terminal: CodexTerminal) -> gpui::Div {
    div().p(px(24.0)).child(terminal)
}

impl Render for CodexPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = CodexPalette::new(cx.notra().appearance);
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
        let drafted = Rich::new()
            .push(Run::new("Drafted from ", p.fg))
            .push(Run::new("git log", p.green))
            .push(Run::new(" and ", p.fg))
            .push(Run::new("CHANGELOG.md", p.fg).bold())
            .push(Run::new(":", p.fg));
        let execs = session::exec_states()
            .into_iter()
            .map(|exec| CodexExec::new(SharedString::from(format!("exec-{}", exec.id)), exec));

        div()
            .flex()
            .flex_col()
            .w(px(0.0))
            .min_w_full()
            .child(section("Demo", cx).child(self.demo.clone()))
            .child(
                section("Terminal", cx)
                    .child(inset(terminal("cx-terminal").child(
                        CodexMessage::assistant().child("Any Codex content goes here."),
                    ))),
            )
            .child(
                section("Header", cx).child(inset(terminal("cx-header").child(CodexHeader::new()))),
            )
            .child(
                section("Message", cx).child(inset(
                    terminal("cx-message")
                        .child(CodexMessage::user(session::USER_MESSAGE))
                        .child(CodexMessage::assistant().child(session::INTRO)),
                )),
            )
            .child(section("Reasoning", cx).child(inset(
                terminal("cx-reasoning").child(CodexReasoning::new(session::REASONING)),
            )))
            .child(section("Exec", cx).child(inset(terminal("cx-exec").children(execs))))
            .child(
                section("Explored", cx).child(inset(
                    terminal("cx-explored")
                        .child(
                            CodexExplored::new("explored-done", session::EXPLORED.to_vec())
                                .details(session::EXPLORED_DETAILS.to_vec()),
                        )
                        .child(
                            CodexExplored::new(
                                "explored-active",
                                session::EXPLORED_DETAILS.to_vec(),
                            )
                            .active(true),
                        ),
                )),
            )
            .child(section("Working", cx).child(inset(
                terminal("cx-working").child(CodexWorking::new("working-example").elapsed("13s")),
            )))
            .child(
                section("Markdown", cx).child(inset(
                    terminal("cx-markdown").child(
                        CodexMessage::assistant()
                            .child(rich_text(&drafted))
                            .child(CodexList::new(highlights))
                            .child(
                                CodexTable::new(
                                    session::TABLE_HEADERS
                                        .iter()
                                        .map(|h| SharedString::from(*h))
                                        .collect(),
                                    rows,
                                )
                                .code_columns(vec![0]),
                            ),
                    ),
                )),
            )
            .child(
                section("Composer", cx).child(inset(
                    terminal("cx-composer").child(CodexHeader::new()).footer(
                        CodexComposer::new("cx-composer-field", self.input.clone())
                            .effort(session::EFFORT)
                            .task(session::TASK)
                            .warnings(session::WARNINGS),
                    ),
                )),
            )
    }
}
