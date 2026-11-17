use std::time::Duration;

use notra_ui::Theme;
use notra_ui::components::claude_code::{
    ClaudeCodeEffort, ClaudeCodeMode, TurnSummary, examples, session,
};
use notra_ui::rune::{
    Button, ButtonSize, ButtonVariant, ClaudeCodeDemo, ClaudeCodeDemoState, ClaudeCodeHeader,
    ClaudeCodeMessage, ClaudeCodePrompt, ClaudeCodeSpinner, ClaudeCodeState, ClaudeCodeTerminal,
    ClaudeCodeThinking, ClaudeCodeTodoList, ClaudeCodeToolCall, ClaudeCodeToolSummary,
    ClaudeCodeTurnSummary,
};
use rune::{El, KeyEvent, TextEdit, Tx, col, row};

use crate::{State, section};

pub struct Demo {
    pub demo: ClaudeCodeDemoState,
    pub open: ClaudeCodeState,
    pub draft: TextEdit,
    pub mode: ClaudeCodeMode,
    pub effort: ClaudeCodeEffort,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            demo: ClaudeCodeDemoState::default(),
            open: ClaudeCodeState::with_open(["thinking-open".to_owned()]),
            draft: TextEdit::new(""),
            mode: ClaudeCodeMode::Bypass,
            effort: ClaudeCodeEffort::High,
        }
    }
}

impl Demo {
    /// The demo two and a half seconds into a reply, with every collapsible open.
    pub fn playing() -> Self {
        let mut demo = Self::default();
        demo.demo
            .send_at("write a LinkedIn post about the release", Duration::ZERO);
        for id in ["shell", "tool-read", "thinking-closed"] {
            demo.open.toggle(id);
        }
        demo
    }
}

pub fn poll(s: &mut State, clock: f64, tx: &mut Tx) -> bool {
    s.claude_code.demo.poll(clock, tx)
}

fn open(s: &mut State) -> &mut ClaudeCodeState {
    &mut s.claude_code.open
}

fn terminal(key: &str, title: &str) -> ClaudeCodeTerminal {
    ClaudeCodeTerminal::new(key).title(title)
}

pub fn view(s: &State, t: &Theme) -> El {
    let d = &s.claude_code;
    let tools = examples::tool_calls().into_iter().map(|call| {
        let call = notra_ui::components::claude_code::ToolCall {
            id: format!("tool-{}", call.id),
            ..call
        };
        ClaudeCodeToolCall::new(call).build(t, &d.open, open)
    });
    let modes = ClaudeCodeMode::ORDER
        .iter()
        .fold(row().flex_wrap().gap(8.0), |r, &mode| {
            r.child(
                Button::new(mode.name())
                    .size(ButtonSize::Sm)
                    .variant(if mode == d.mode {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Outline
                    })
                    .on_press(move |s: &mut State| s.claude_code.mode = mode)
                    .build(t)
                    .key(format!("cc-mode-{}", mode.name())),
            )
        });
    let efforts = ClaudeCodeEffort::ORDER
        .iter()
        .fold(row().flex_wrap().gap(8.0), |r, &effort| {
            r.child(
                Button::new(effort.name())
                    .size(ButtonSize::Sm)
                    .variant(if effort == d.effort {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Outline
                    })
                    .on_press(move |s: &mut State| s.claude_code.effort = effort)
                    .build(t)
                    .key(format!("cc-effort-{}", effort.name())),
            )
        });
    let prompt = ClaudeCodePrompt::new("cc-prompt-field", &d.draft, |s: &mut State| {
        &mut s.claude_code.draft
    })
    .placeholder("Type here · Shift+Tab cycles the mode")
    .mode(d.mode)
    .effort(Some(d.effort))
    .pull_request(Some(session::PULL_REQUEST))
    .on_submit(|s: &mut State, _: &mut Tx| s.claude_code.draft = TextEdit::new(""))
    .on_key(|s: &mut State, k: &KeyEvent, _: &mut Tx| {
        let cycle = k.key == rune::Key::Tab && k.mods.shift;
        if cycle {
            s.claude_code.mode = s.claude_code.mode.next();
        }
        cycle
    });

    col()
        .w_full()
        .child(section(
            "Demo",
            t,
            ClaudeCodeDemo::build(t, &d.demo, |s: &mut State| &mut s.claude_code.demo),
        ))
        .child(section(
            "Terminal",
            t,
            terminal("cc-terminal", session::TITLE)
                .child(ClaudeCodeMessage::user("what changed since the last release?").build(t))
                .child(ClaudeCodeMessage::assistant(session::FIRST_ANSWER).build(t))
                .build(t),
        ))
        .child(section(
            "Header",
            t,
            terminal("cc-header", "claude — header")
                .child(ClaudeCodeHeader::new(session::HEADER).build(t))
                .build(t),
        ))
        .child(section(
            "Message",
            t,
            terminal("cc-messages", "claude — messages")
                .child(ClaudeCodeMessage::user("what changed since the last release?").build(t))
                .child(ClaudeCodeMessage::assistant(session::FIRST_ANSWER).build(t))
                .build(t),
        ))
        .child(section(
            "Todo list",
            t,
            terminal("cc-todos", "claude — todos")
                .child(ClaudeCodeTodoList::new(examples::todos()).build(t))
                .build(t),
        ))
        .child(section(
            "Thinking",
            t,
            terminal("cc-thinking", "claude — thinking")
                .child(
                    ClaudeCodeThinking::new("thinking-closed")
                        .body(examples::THINKING_BODY)
                        .build(t, &d.open, open),
                )
                .child(
                    ClaudeCodeThinking::new("thinking-open")
                        .label("Thought for 3s")
                        .body(examples::THINKING_BODY)
                        .build(t, &d.open, open),
                )
                .build(t),
        ))
        .child(section(
            "Tool call",
            t,
            terminal("cc-tools", "claude — tools")
                .children(tools)
                .child(
                    ClaudeCodeToolSummary::new("shell", examples::shell_calls())
                        .build(t, &d.open, open),
                )
                .build(t),
        ))
        .child(section(
            "Status",
            t,
            terminal("cc-status", "claude — status")
                .child(ClaudeCodeSpinner::new("cc-spinner-long", examples::spinner()).build(t))
                .child(
                    ClaudeCodeSpinner::new("cc-spinner-short", examples::short_spinner()).build(t),
                )
                .child(
                    ClaudeCodeTurnSummary::new(TurnSummary::new("Cogitated", "7s", "9:41 AM"))
                        .build(t),
                )
                .child(
                    ClaudeCodeTurnSummary::new(TurnSummary::new("Worked", "19s", "9:42 AM"))
                        .build(t),
                )
                .build(t),
        ))
        .child(section(
            "Prompt",
            t,
            col()
                .w_full()
                .gap(12.0)
                .child(
                    terminal("cc-prompt", "claude — prompt")
                        .child(prompt.build(t))
                        .build(t),
                )
                .child(modes)
                .child(efforts),
        ))
}
