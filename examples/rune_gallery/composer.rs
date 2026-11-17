use notra_ui::Icon;
use notra_ui::Theme;
use notra_ui::components::composer::{Queue, Tone, demo};
use notra_ui::rune::{
    Button, ButtonSize, ButtonVariant, ChipLead, ComposerChip, ComposerFrame, ComposerInput,
    ComposerNotice, ComposerNudge, ComposerSend, ComposerToolbar, ComposerToolbarButton,
};
use rune::{El, TextEdit, col, row};

use crate::{State, preview, section};

/// The static state examples, in docs order: name, draft, placeholder.
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

pub struct Demo {
    pub main: TextEdit,
    pub queue_field: TextEdit,
    pub queue: Queue,
    pub has_file: bool,
    pub states: Vec<TextEdit>,
    pub reply: TextEdit,
    /// Shows only these examples (by title), so a scene fits the window.
    pub only: Option<&'static [&'static str]>,
}

/// Scene groups of examples that fit one window.
pub const STATE_EXAMPLES: &[&str] = &["Empty", "Ready", "Streaming", "Stopping", "Queued"];
pub const TRAY_EXAMPLES: &[&str] = &["Waiting", "Uploading", "Notice", "Limit", "Flat"];

impl Default for Demo {
    fn default() -> Self {
        Self {
            main: TextEdit::new(""),
            queue_field: TextEdit::new(""),
            queue: Queue::new(demo::QUEUE),
            has_file: true,
            states: STATES
                .iter()
                .map(|(_, text, _)| TextEdit::new(*text))
                .collect(),
            reply: TextEdit::new(""),
            only: None,
        }
    }
}

impl Demo {
    pub fn only(examples: &'static [&'static str]) -> Self {
        Self {
            only: Some(examples),
            ..Self::default()
        }
    }

    /// The main composer with a long draft and the caret in it.
    pub fn typed() -> Self {
        let mut main = TextEdit::new(
            "Draft release notes for the October launch, then turn the highlights into a short thread for the changelog",
        );
        main.focused = true;
        Self {
            main,
            ..Self::default()
        }
    }

    fn queue_message(&mut self) {
        if self.queue.push(&self.queue_field.text).is_some() {
            self.queue_field.clear();
        }
    }
}

fn width(el: El) -> El {
    col().w(528.0).child(el)
}

fn state_toolbar(t: &Theme, disabled: bool, send: ComposerSend) -> El {
    ComposerToolbar::new()
        .child(
            ComposerToolbarButton::new("Auto")
                .chevron()
                .disabled(disabled)
                .build(t),
        )
        .trailing(
            ComposerToolbarButton::icon_button(Icon::Plus, "Attach files")
                .disabled(disabled)
                .build(t),
        )
        .trailing(send.build(t))
        .build()
}

fn upgrade(t: &Theme) -> El {
    Button::new("Upgrade")
        .variant(ButtonVariant::Outline)
        .size(ButtonSize::Xs)
        .build(t)
}

fn main_composer(s: &State, t: &Theme) -> El {
    let empty = s.composer.main.text.trim().is_empty();
    ComposerFrame::new()
        .child(
            ComposerInput::new(&s.composer.main)
                .placeholder(demo::PLACEHOLDER)
                .label("Message")
                .on_edit(|s: &mut State| &mut s.composer.main)
                .build(t),
        )
        .child(
            ComposerToolbar::new()
                .child(ComposerToolbarButton::new("Auto").chevron().build(t))
                .child(
                    ComposerToolbarButton::new("Medium")
                        .icon(Icon::Brain)
                        .chevron()
                        .build(t),
                )
                .trailing(ComposerToolbarButton::icon_button(Icon::AtSign, "Add context").build(t))
                .trailing(ComposerToolbarButton::icon_button(Icon::Plus, "Attach files").build(t))
                .trailing(
                    ComposerSend::new("main-send", "Send message")
                        .disabled(empty)
                        .tooltip("Send")
                        .on_press(|s: &mut State| s.composer.main.clear())
                        .build(t),
                )
                .build(),
        )
        .build(t)
}

fn queue_composer(s: &State, t: &Theme) -> El {
    let d = &s.composer;
    let has_text = !d.queue_field.text.trim().is_empty();
    let nudge = (!d.queue.is_empty() || d.has_file).then(|| {
        let mut n = ComposerNudge::new();
        for m in d.queue.messages() {
            let (id, text) = (m.id, m.text.clone());
            n = n.child(
                ComposerChip::new(m.text.clone())
                    .on_steer(move |s: &mut State| {
                        s.composer.queue.take(id);
                    })
                    .on_edit(move |s: &mut State| {
                        s.composer.queue.take(id);
                        s.composer.queue_field.replace_text(text.clone());
                        s.composer.queue_field.focused = true;
                    })
                    .on_remove(move |s: &mut State| {
                        s.composer.queue.take(id);
                    })
                    .build(t)
                    .key(format!("queued-{id}")),
            );
        }
        if d.has_file {
            n = n.child(
                ComposerChip::new(demo::QUEUE_FILE)
                    .lead(ChipLead::Icon(Icon::FileText))
                    .on_remove(|s: &mut State| s.composer.has_file = false)
                    .build(t)
                    .key("file"),
            );
        }
        n.build(t)
    });
    ComposerFrame::new()
        .nudge(nudge)
        .child(
            ComposerInput::new(&d.queue_field)
                .placeholder(demo::QUEUE_ANOTHER_PLACEHOLDER)
                .label("Message")
                .on_edit(|s: &mut State| &mut s.composer.queue_field)
                .build(t),
        )
        .child(
            ComposerToolbar::new()
                .trailing(
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
                    .on_press(|s: &mut State| s.composer.queue_message())
                    .build(t),
                )
                .build(),
        )
        .build(t)
}

pub fn view(s: &State, t: &Theme) -> El {
    let d = &s.composer;
    let input = |i: usize| {
        let (name, _, placeholder) = STATES[i];
        ComposerInput::new(&d.states[i])
            .placeholder(placeholder)
            .label("Message")
            .disabled(name == "waiting")
            .on_edit(move |s: &mut State| &mut s.composer.states[i])
            .build(t)
    };
    let frame = |i: usize, nudge: Option<El>, disabled: bool, send: ComposerSend| {
        ComposerFrame::new()
            .nudge(nudge)
            .child(input(i))
            .child(state_toolbar(t, disabled, send))
            .build(t)
    };
    let queued = ComposerNudge::new()
        .child(
            ComposerChip::new(demo::QUEUE[1])
                .on_steer(|_: &mut State| {})
                .on_edit(|_: &mut State| {})
                .on_remove(|_: &mut State| {})
                .build(t)
                .key("queued"),
        )
        .build(t);
    let uploading = ComposerNudge::new()
        .child(
            ComposerChip::new(demo::UPLOADED_FILE)
                .lead(ChipLead::Icon(Icon::FileText))
                .on_click(|_: &mut State| {})
                .on_remove(|_: &mut State| {})
                .build(t)
                .key("uploaded"),
        )
        .child(
            ComposerChip::new(demo::UPLOADING_FILE)
                .lead(ChipLead::Spinner)
                .pending(true)
                .build(t)
                .key("uploading"),
        )
        .build(t);
    let notice = ComposerNudge::new()
        .title_el(ComposerNotice::new(Tone::Info, Icon::CircleAlert, demo::NOTICE).build(t))
        .action(upgrade(t))
        .build(t);
    let limit = ComposerNudge::new()
        .child(
            ComposerNotice::new(Tone::Warning, Icon::TriangleAlert, demo::LIMIT)
                .body()
                .build(t)
                .key("limit"),
        )
        .action(upgrade(t))
        .build(t);
    let flat = ComposerFrame::new()
        .flat()
        .nudge(Some(
            ComposerNudge::new()
                .title(demo::REPLY_TITLE)
                .action(
                    Button::icon_button(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconXs)
                        .build(t),
                )
                .build(t),
        ))
        .child(
            row()
                .items_end()
                .gap(8.0)
                .padding(8.0)
                .child(
                    ComposerInput::new(&d.reply)
                        .flat()
                        .placeholder(demo::REPLY_PLACEHOLDER)
                        .label("Reply to Dominik")
                        .on_edit(|s: &mut State| &mut s.composer.reply)
                        .build(t),
                )
                .child(ComposerSend::new("flat-send", "Send").round(true).build(t)),
        )
        .build(t);

    let examples = [
        ("Composer", main_composer(s, t)),
        ("Queue", queue_composer(s, t)),
        (
            "Empty",
            frame(
                0,
                None,
                false,
                ComposerSend::new("empty-send", "Send").disabled(true),
            ),
        ),
        (
            "Ready",
            frame(
                1,
                None,
                false,
                ComposerSend::new("ready-send", "Send message"),
            ),
        ),
        (
            "Streaming",
            frame(
                2,
                None,
                false,
                ComposerSend::new("streaming-send", "Stop").icon(Icon::SquareSolid),
            ),
        ),
        (
            "Stopping",
            frame(
                3,
                None,
                false,
                ComposerSend::new("stopping-send", "Stopping")
                    .icon(Icon::SquareSolid)
                    .busy(true),
            ),
        ),
        (
            "Queued",
            frame(
                4,
                Some(queued),
                false,
                ComposerSend::new("queued-send", "Queue message"),
            ),
        ),
        (
            "Waiting",
            frame(
                5,
                None,
                true,
                ComposerSend::new("waiting-send", "Cancel queued message").busy(true),
            ),
        ),
        (
            "Uploading",
            frame(
                6,
                Some(uploading),
                false,
                ComposerSend::new("uploading-send", "Send").disabled(true),
            ),
        ),
        (
            "Notice",
            frame(
                7,
                Some(notice),
                false,
                ComposerSend::new("notice-send", "Send message"),
            ),
        ),
        (
            "Limit",
            frame(
                8,
                Some(limit),
                false,
                ComposerSend::new("limit-send", "Send").disabled(true),
            ),
        ),
        ("Flat", flat),
    ];
    examples
        .into_iter()
        .filter(|(title, _)| d.only.is_none_or(|only| only.contains(title)))
        .fold(col(), |page, (title, el)| {
            page.child(
                section(
                    title,
                    t,
                    preview(t).child(width(el).key(format!("example-{title}"))),
                )
                .key(format!("section-{title}")),
            )
        })
}
