use std::time::Duration;

use notra_ui::Theme;
use notra_ui::components::codex::{CodexPalette, session};
use notra_ui::components::terminal::{Rich, Run};
use notra_ui::rune::{
    CodexComposer, CodexDemo, CodexDemoState, CodexExec, CodexExplored, CodexHeader, CodexList,
    CodexMessage, CodexReasoning, CodexState, CodexTable, CodexTerminal, CodexWorking, code_font,
    rich_text,
};
use rune::{El, TextEdit, Tx, col, el};

use crate::{State, section};

#[derive(Default)]
pub struct Demo {
    pub demo: CodexDemoState,
    pub open: CodexState,
    pub draft: TextEdit,
}

impl Demo {
    /// The demo two seconds into a reply, with the examples' collapsibles open.
    pub fn playing() -> Self {
        let mut demo = Self::default();
        demo.demo.send_at("publish it", Duration::ZERO);
        for id in ["exec-ran", "explored-done"] {
            demo.open.toggle(id);
        }
        demo
    }
}

pub fn poll(s: &mut State, clock: f64, tx: &mut Tx) -> bool {
    s.codex.demo.poll(clock, tx)
}

fn open(s: &mut State) -> &mut CodexState {
    &mut s.codex.open
}

/// The examples sit inset on the page, as the docs frame them.
fn inset(key: &str, t: &Theme, children: Vec<El>) -> El {
    el().w_full().padding(24.0).child(
        CodexTerminal::new(key)
            .title(session::TITLE)
            .children(children)
            .build(t),
    )
}

pub fn view(s: &State, t: &Theme) -> El {
    let d = &s.codex;
    let p = CodexPalette::new(t.appearance);
    let font = code_font(13.0, 400.0, 16.9);
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
    let drafted = Rich::new()
        .push(Run::new("Drafted from ", p.fg))
        .push(Run::new("git log", p.green))
        .push(Run::new(" and ", p.fg))
        .push(Run::new("CHANGELOG.md", p.fg).bold())
        .push(Run::new(":", p.fg));
    let execs = session::exec_states()
        .into_iter()
        .map(|exec| {
            let exec = notra_ui::components::codex::Exec {
                id: format!("exec-{}", exec.id),
                ..exec
            };
            CodexExec::new(exec).build(t, &d.open, open)
        })
        .collect();
    let composer = CodexComposer::new("cx-composer-field", &d.draft, |s: &mut State| {
        &mut s.codex.draft
    })
    .effort(session::EFFORT)
    .task(session::TASK)
    .warnings(session::WARNINGS)
    .on_submit(|s: &mut State, _: &mut Tx| s.codex.draft = TextEdit::new(""));

    col()
        .w_full()
        .child(section(
            "Demo",
            t,
            CodexDemo::build(t, &d.demo, |s: &mut State| &mut s.codex.demo),
        ))
        .child(section(
            "Terminal",
            t,
            inset(
                "cx-terminal",
                t,
                vec![
                    CodexMessage::assistant()
                        .child(notra_ui::rune::codex_paragraph(
                            "Any Codex content goes here.",
                            t,
                        ))
                        .build(t),
                ],
            ),
        ))
        .child(section(
            "Header",
            t,
            inset("cx-header", t, vec![CodexHeader::new().build(t)]),
        ))
        .child(section(
            "Message",
            t,
            inset(
                "cx-message",
                t,
                vec![
                    CodexMessage::user(session::USER_MESSAGE).build(t),
                    CodexMessage::assistant()
                        .child(notra_ui::rune::codex_paragraph(session::INTRO, t))
                        .build(t),
                ],
            ),
        ))
        .child(section(
            "Reasoning",
            t,
            inset(
                "cx-reasoning",
                t,
                vec![CodexReasoning::new(session::REASONING).build(t)],
            ),
        ))
        .child(section("Exec", t, inset("cx-exec", t, execs)))
        .child(section(
            "Explored",
            t,
            inset(
                "cx-explored",
                t,
                vec![
                    CodexExplored::new("explored-done", session::EXPLORED.to_vec())
                        .details(session::EXPLORED_DETAILS.to_vec())
                        .build(t, &d.open, open),
                    CodexExplored::new("explored-active", session::EXPLORED_DETAILS.to_vec())
                        .active(true)
                        .build(t, &d.open, open),
                ],
            ),
        ))
        .child(section(
            "Working",
            t,
            inset(
                "cx-working",
                t,
                vec![CodexWorking::new("cx-working-line").elapsed("13s").build(t)],
            ),
        ))
        .child(section(
            "Markdown",
            t,
            inset(
                "cx-markdown",
                t,
                vec![
                    CodexMessage::assistant()
                        .child(rich_text(&drafted, font))
                        .child(CodexList::new(highlights).build(t))
                        .child(
                            CodexTable::new(
                                session::TABLE_HEADERS
                                    .iter()
                                    .map(|h| (*h).to_owned())
                                    .collect(),
                                rows,
                            )
                            .code_columns(vec![0])
                            .build(t),
                        )
                        .build(t),
                ],
            ),
        ))
        .child(section(
            "Composer",
            t,
            el().w_full().padding(24.0).child(
                CodexTerminal::new("cx-composer")
                    .title(session::TITLE)
                    .child(CodexHeader::new().build(t))
                    .footer(composer.build(t))
                    .build(t),
            ),
        ))
}
