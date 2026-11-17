use gpui::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled,
    Subscription, Window, div, px,
};
use notra_ui::components::claude_code::{
    ClaudeCodeEffort, ClaudeCodeMode, TurnSummary, examples, session,
};
use notra_ui::gpui::{
    Button, ButtonSize, ButtonVariant, ClaudeCodeDemo, ClaudeCodeHeader, ClaudeCodeMessage,
    ClaudeCodePrompt, ClaudeCodeSpinner, ClaudeCodeTerminal, ClaudeCodeThinking,
    ClaudeCodeTodoList, ClaudeCodeToolCall, ClaudeCodeToolSummary, ClaudeCodeTurnSummary,
    PromptEvent, PromptInput,
};

use crate::{Page, section};

pub fn page(window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "claude-code",
        title: "Claude Code",
        view: cx.new(|cx| ClaudeCodePage::new(window, cx)).into(),
    }
}

struct ClaudeCodePage {
    demo: Entity<ClaudeCodeDemo>,
    input: Entity<PromptInput>,
    mode: ClaudeCodeMode,
    effort: ClaudeCodeEffort,
    _events: Subscription,
}

impl ClaudeCodePage {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(PromptInput::new);
        let events = cx.subscribe(&input, |this: &mut Self, input, event, cx| {
            match event {
                PromptEvent::CycleMode => this.mode = this.mode.next(),
                PromptEvent::Submit(_) => input.update(cx, |input, cx| input.clear(cx)),
                PromptEvent::Escape => {}
            }
            cx.notify();
        });
        Self {
            demo: cx.new(|cx| ClaudeCodeDemo::new(window, cx)),
            input,
            mode: ClaudeCodeMode::Bypass,
            effort: ClaudeCodeEffort::High,
            _events: events,
        }
    }
}

fn terminal(id: &'static str, title: &'static str) -> ClaudeCodeTerminal {
    ClaudeCodeTerminal::new(id).title(title)
}

impl Render for ClaudeCodePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let modes = ClaudeCodeMode::ORDER.map(|mode| {
            Button::new(mode.name(), mode.name())
                .size(ButtonSize::Sm)
                .variant(if mode == self.mode {
                    ButtonVariant::Secondary
                } else {
                    ButtonVariant::Outline
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.mode = mode;
                    cx.notify();
                }))
        });
        let efforts = ClaudeCodeEffort::ORDER.map(|effort| {
            Button::new(effort.name(), effort.name())
                .size(ButtonSize::Sm)
                .variant(if effort == self.effort {
                    ButtonVariant::Secondary
                } else {
                    ButtonVariant::Outline
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.effort = effort;
                    cx.notify();
                }))
        });
        let tools = examples::tool_calls()
            .into_iter()
            .map(|call| ClaudeCodeToolCall::new(format!("tool-{}", call.id), call));

        div()
            .flex()
            .flex_col()
            .w(px(0.0))
            .min_w_full()
            .child(section("Demo", cx).child(self.demo.clone()))
            .child(
                section("Terminal", cx).child(
                    terminal("cc-terminal", session::TITLE)
                        .child(ClaudeCodeMessage::user(
                            "what changed since the last release?",
                        ))
                        .child(ClaudeCodeMessage::assistant(session::FIRST_ANSWER)),
                ),
            )
            .child(
                section("Header", cx).child(
                    terminal("cc-header", "claude — header")
                        .child(ClaudeCodeHeader::new(session::HEADER)),
                ),
            )
            .child(
                section("Message", cx).child(
                    terminal("cc-messages", "claude — messages")
                        .child(ClaudeCodeMessage::user(
                            "what changed since the last release?",
                        ))
                        .child(ClaudeCodeMessage::assistant(session::FIRST_ANSWER)),
                ),
            )
            .child(
                section("Todo list", cx).child(
                    terminal("cc-todos", "claude — todos")
                        .child(ClaudeCodeTodoList::new(examples::todos())),
                ),
            )
            .child(
                section("Thinking", cx).child(
                    terminal("cc-thinking", "claude — thinking")
                        .child(
                            ClaudeCodeThinking::new("thinking-closed")
                                .body(examples::THINKING_BODY),
                        )
                        .child(
                            ClaudeCodeThinking::new("thinking-open")
                                .label("Thought for 3s")
                                .body(examples::THINKING_BODY)
                                .default_open(true),
                        ),
                ),
            )
            .child(
                section("Tool call", cx).child(
                    terminal("cc-tools", "claude — tools")
                        .children(tools)
                        .child(ClaudeCodeToolSummary::new("shell", examples::shell_calls())),
                ),
            )
            .child(
                section("Status", cx).child(
                    terminal("cc-status", "claude — status")
                        .child(ClaudeCodeSpinner::new("spinner-long", examples::spinner()))
                        .child(ClaudeCodeSpinner::new(
                            "spinner-short",
                            examples::short_spinner(),
                        ))
                        .child(ClaudeCodeTurnSummary::new(TurnSummary::new(
                            "Cogitated",
                            "7s",
                            "9:41 AM",
                        )))
                        .child(ClaudeCodeTurnSummary::new(TurnSummary::new(
                            "Worked", "19s", "9:42 AM",
                        ))),
                ),
            )
            .child(
                section("Prompt", cx).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            terminal("cc-prompt", "claude — prompt").child(
                                ClaudeCodePrompt::new("cc-prompt-field", self.input.clone())
                                    .placeholder("Type here · Shift+Tab cycles the mode")
                                    .mode(self.mode)
                                    .effort(Some(self.effort))
                                    .pull_request(Some(session::PULL_REQUEST)),
                            ),
                        )
                        .child(div().flex().flex_wrap().gap(px(8.0)).children(modes))
                        .child(div().flex().flex_wrap().gap(px(8.0)).children(efforts)),
                ),
            )
    }
}
