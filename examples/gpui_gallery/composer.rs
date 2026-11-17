use gpui::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled,
    Subscription, Window, div, px,
};
use notra_ui::Icon;
use notra_ui::components::composer::{ComposerShape, Queue, Tone, demo};
use notra_ui::gpui::{
    Button, ButtonSize, ButtonVariant, ChipLead, ComposerChip, ComposerFrame, ComposerInput,
    ComposerNotice, ComposerNudge, ComposerSend, ComposerToolbar, ComposerToolbarButton, TextField,
    TextFieldEvent,
};

use crate::{Page, preview, section};

pub fn page(window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "composer",
        title: "Composer",
        view: cx.new(|cx| ComposerPage::new(window, cx)).into(),
    }
}

/// The static state examples, in docs order.
const STATES: [(&str, &str, &str); 9] = [
    ("empty", "", demo::PLACEHOLDER),
    ("ready", demo::READY_DRAFT, demo::PLACEHOLDER),
    ("streaming", "", demo::QUEUE_PLACEHOLDER),
    ("stopping", "", demo::QUEUE_PLACEHOLDER),
    ("queued", demo::QUEUED_DRAFT, demo::QUEUE_PLACEHOLDER),
    ("waiting", demo::WAITING_DRAFT, demo::PLACEHOLDER),
    ("uploading", demo::UPLOAD_DRAFT, demo::PLACEHOLDER),
    ("notice", demo::NOTICE_DRAFT, demo::PLACEHOLDER),
    ("limit", "", demo::PLACEHOLDER),
];

struct ComposerPage {
    main: Entity<TextField>,
    queue_field: Entity<TextField>,
    queue: Queue,
    has_file: bool,
    states: Vec<Entity<TextField>>,
    reply: Entity<TextField>,
    _subscriptions: Vec<Subscription>,
}

fn card_field(
    cx: &mut App,
    text: &str,
    placeholder: &'static str,
    disabled: bool,
) -> Entity<TextField> {
    let text = text.to_owned();
    cx.new(|cx| {
        ComposerInput::field(cx, ComposerShape::Card)
            .placeholder(placeholder)
            .label("Message")
            .default_value(text)
            .disabled(disabled)
    })
}

impl ComposerPage {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let main = card_field(cx, "", demo::PLACEHOLDER, false);
        let queue_field = card_field(cx, "", demo::QUEUE_ANOTHER_PLACEHOLDER, false);
        let states = STATES
            .iter()
            .map(|&(name, text, placeholder)| card_field(cx, text, placeholder, name == "waiting"))
            .collect();
        let reply = cx.new(|cx| {
            ComposerInput::field(cx, ComposerShape::Flat)
                .placeholder(demo::REPLY_PLACEHOLDER)
                .label("Reply to Dominik")
        });
        let subscriptions = vec![
            cx.observe(&main, |_, _, cx| cx.notify()),
            cx.observe(&queue_field, |_, _, cx| cx.notify()),
            cx.subscribe(&main, |_, field, event, cx| {
                if *event == TextFieldEvent::Submit {
                    field.update(cx, |f, cx| f.set_text("", cx));
                }
            }),
            cx.subscribe(&queue_field, |this, _, event, cx| {
                if *event == TextFieldEvent::Submit {
                    this.queue_message(cx);
                }
            }),
        ];
        Self {
            main,
            queue_field,
            queue: Queue::new(demo::QUEUE),
            has_file: true,
            states,
            reply,
            _subscriptions: subscriptions,
        }
    }

    fn queue_message(&mut self, cx: &mut Context<Self>) {
        let text = self.queue_field.read(cx).text().to_owned();
        if self.queue.push(&text).is_some() {
            self.queue_field.update(cx, |f, cx| f.set_text("", cx));
        }
        cx.notify();
    }

    fn main_composer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let empty = self.main.read(cx).text().trim().is_empty();
        let main = self.main.clone();
        ComposerFrame::new("composer-main")
            .child(ComposerInput::new(self.main.clone()))
            .child(
                ComposerToolbar::new()
                    .child(ComposerToolbarButton::new("main-model", "Auto").chevron())
                    .child(
                        ComposerToolbarButton::new("main-effort", "Medium")
                            .icon(Icon::Brain)
                            .chevron(),
                    )
                    .child(
                        div()
                            .ml_auto()
                            .flex()
                            .items_center()
                            .gap(px(4.))
                            .child(ComposerToolbarButton::icon_button(
                                "main-context",
                                Icon::AtSign,
                                "Add context",
                            ))
                            .child(ComposerToolbarButton::icon_button(
                                "main-attach",
                                Icon::Plus,
                                "Attach files",
                            ))
                            .child(
                                ComposerSend::new("main-send", "Send message")
                                    .disabled(empty)
                                    .tooltip("Send")
                                    .on_click(move |_, _, cx| {
                                        main.update(cx, |f, cx| f.set_text("", cx));
                                    }),
                            ),
                    ),
            )
    }

    fn queue_composer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let has_text = !self.queue_field.read(cx).text().trim().is_empty();
        let view = cx.entity();
        let messages = self.queue.messages().to_vec();
        let has_file = self.has_file;
        let has_nudge = !messages.is_empty() || has_file;
        let mut frame = ComposerFrame::new("composer-queue");
        if has_nudge {
            let view = view.clone();
            frame = frame.nudge(move |_, _| {
                let chips = messages.iter().map(|m| {
                    let (id, text) = (m.id, m.text.clone());
                    let (v1, v2, v3) = (view.clone(), view.clone(), view.clone());
                    ComposerChip::new(("queued", id), m.text.clone())
                        .on_steer(move |_, _, cx| {
                            v1.update(cx, |p, cx| {
                                p.queue.take(id);
                                cx.notify();
                            });
                        })
                        .on_edit(move |_, window, cx| {
                            let text = text.clone();
                            v2.update(cx, |p, cx| {
                                p.queue.take(id);
                                p.queue_field.update(cx, |f, cx| f.set_text(text, cx));
                                p.queue_field.update(cx, |f, cx| f.focus(window, cx));
                                cx.notify();
                            });
                        })
                        .on_remove(move |_, _, cx| {
                            v3.update(cx, |p, cx| {
                                p.queue.take(id);
                                cx.notify();
                            });
                        })
                });
                let file = view.clone();
                ComposerNudge::new()
                    .children(chips)
                    .children(has_file.then(|| {
                        ComposerChip::new("queue-file", demo::QUEUE_FILE)
                            .lead(ChipLead::Icon(Icon::FileText))
                            .on_remove(move |_, _, cx| {
                                file.update(cx, |p, cx| {
                                    p.has_file = false;
                                    cx.notify();
                                });
                            })
                    }))
                    .into_any_element()
            });
        }
        frame
            .child(ComposerInput::new(self.queue_field.clone()))
            .child(
                ComposerToolbar::new().child(
                    div().ml_auto().flex().items_center().gap(px(4.)).child(
                        ComposerSend::new(
                            "queue-send",
                            if has_text { "Queue message" } else { "Stop" },
                        )
                        .icon(if has_text {
                            Icon::ArrowUp
                        } else {
                            Icon::SquareSolid
                        })
                        .tooltip(if has_text { "Queue" } else { "Stop" })
                        .on_click(cx.listener(|this, _, _, cx| this.queue_message(cx))),
                    ),
                ),
            )
    }
}

fn state_toolbar(name: &str, disabled: bool, send: ComposerSend) -> ComposerToolbar {
    ComposerToolbar::new()
        .child(
            ComposerToolbarButton::new(format!("{name}-model"), "Auto")
                .chevron()
                .disabled(disabled),
        )
        .child(
            div()
                .ml_auto()
                .flex()
                .items_center()
                .gap(px(4.))
                .child(
                    ComposerToolbarButton::icon_button(
                        format!("{name}-attach"),
                        Icon::Plus,
                        "Attach files",
                    )
                    .disabled(disabled),
                )
                .child(send),
        )
}

fn upgrade(id: &'static str) -> Button {
    Button::new(id, "Upgrade")
        .variant(ButtonVariant::Outline)
        .size(ButtonSize::Xs)
}

impl Render for ComposerPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let width = |el: AnyComposer| div().w(px(528.)).child(el);
        let state = |i: usize| ComposerInput::new(self.states[i].clone());
        let send = |name: &str| ComposerSend::new(format!("{name}-send"), "Send message");

        let empty = ComposerFrame::new("state-empty")
            .child(state(0))
            .child(state_toolbar(
                "empty",
                false,
                ComposerSend::new("empty-send", "Send").disabled(true),
            ));
        let ready = ComposerFrame::new("state-ready")
            .child(state(1))
            .child(state_toolbar("ready", false, send("ready")));
        let streaming = ComposerFrame::new("state-streaming")
            .child(state(2))
            .child(state_toolbar(
                "streaming",
                false,
                ComposerSend::new("streaming-send", "Stop").icon(Icon::SquareSolid),
            ));
        let stopping = ComposerFrame::new("state-stopping")
            .child(state(3))
            .child(state_toolbar(
                "stopping",
                false,
                ComposerSend::new("stopping-send", "Stopping")
                    .icon(Icon::SquareSolid)
                    .busy(true),
            ));
        let queued = ComposerFrame::new("state-queued")
            .nudge(|_, _| {
                ComposerNudge::new()
                    .child(
                        ComposerChip::new("queued-chip", demo::QUEUE[1])
                            .on_steer(|_, _, _| {})
                            .on_edit(|_, _, _| {})
                            .on_remove(|_, _, _| {}),
                    )
                    .into_any_element()
            })
            .child(state(4))
            .child(state_toolbar(
                "queued",
                false,
                ComposerSend::new("queued-send", "Queue message"),
            ));
        let waiting = ComposerFrame::new("state-waiting")
            .child(state(5))
            .child(state_toolbar(
                "waiting",
                true,
                ComposerSend::new("waiting-send", "Cancel queued message").busy(true),
            ));
        let uploading = ComposerFrame::new("state-uploading")
            .nudge(|_, _| {
                ComposerNudge::new()
                    .child(
                        ComposerChip::new("uploaded", demo::UPLOADED_FILE)
                            .lead(ChipLead::Icon(Icon::FileText))
                            .on_click(|_, _, _| {})
                            .on_remove(|_, _, _| {}),
                    )
                    .child(
                        ComposerChip::new("uploading", demo::UPLOADING_FILE)
                            .lead(ChipLead::Spinner)
                            .pending(true),
                    )
                    .into_any_element()
            })
            .child(state(6))
            .child(state_toolbar(
                "uploading",
                false,
                ComposerSend::new("uploading-send", "Send").disabled(true),
            ));
        let notice = ComposerFrame::new("state-notice")
            .nudge(|_, _| {
                ComposerNudge::new()
                    .title(ComposerNotice::new(
                        Tone::Info,
                        Icon::CircleAlert,
                        demo::NOTICE,
                    ))
                    .action(upgrade("notice-upgrade"))
                    .into_any_element()
            })
            .child(state(7))
            .child(state_toolbar("notice", false, send("notice")));
        let limit = ComposerFrame::new("state-limit")
            .nudge(|_, _| {
                ComposerNudge::new()
                    .child(
                        ComposerNotice::new(Tone::Warning, Icon::TriangleAlert, demo::LIMIT).body(),
                    )
                    .action(upgrade("limit-upgrade"))
                    .into_any_element()
            })
            .child(state(8))
            .child(state_toolbar(
                "limit",
                false,
                ComposerSend::new("limit-send", "Send").disabled(true),
            ));
        let flat = ComposerFrame::new("state-flat")
            .flat()
            .nudge(|_, _| {
                ComposerNudge::new()
                    .title(demo::REPLY_TITLE)
                    .action(
                        Button::icon_button("cancel-reply", Icon::X)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconXs),
                    )
                    .into_any_element()
            })
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(8.))
                    .p(px(8.))
                    .child(ComposerInput::new(self.reply.clone()))
                    .child(ComposerSend::new("flat-send", "Send").round(true)),
            );

        let examples: Vec<(&'static str, AnyComposer)> = vec![
            ("Composer", self.main_composer(cx).into_any_element()),
            ("Queue", self.queue_composer(cx).into_any_element()),
            ("Empty", empty.into_any_element()),
            ("Ready", ready.into_any_element()),
            ("Streaming", streaming.into_any_element()),
            ("Stopping", stopping.into_any_element()),
            ("Queued", queued.into_any_element()),
            ("Waiting", waiting.into_any_element()),
            ("Uploading", uploading.into_any_element()),
            ("Notice", notice.into_any_element()),
            ("Limit", limit.into_any_element()),
            ("Flat", flat.into_any_element()),
        ];
        div().flex().flex_col().children(
            examples
                .into_iter()
                .map(|(title, el)| section(title, cx).child(preview(cx).child(width(el)))),
        )
    }
}

type AnyComposer = gpui::AnyElement;
