use notra_ui::Icon;
use notra_ui::Theme;
use notra_ui::components::input::AddonAlign;
use notra_ui::rune::{Input, InputGroup, InputGroupAddon, InputGroupButton, Textarea};
use rune::{El, TextEdit, col};

use crate::{State, preview, section};

pub struct Demo {
    pub email: TextEdit,
    pub disabled: TextEdit,
    pub message: TextEdit,
    pub search: TextEdit,
    pub url: TextEdit,
    pub prompt: TextEdit,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            email: TextEdit::new(""),
            disabled: TextEdit::new(""),
            message: TextEdit::new(""),
            search: TextEdit::new(""),
            url: TextEdit::new(""),
            prompt: TextEdit::new(""),
        }
    }
}

impl Demo {
    /// The email field focused with text in it.
    pub fn focused() -> Self {
        let mut email = TextEdit::new("lasse@usenotra.com");
        email.focused = true;
        Self {
            email,
            message: TextEdit::new(
                "Lines wrap and the textarea grows with them, as field-sizing-content does on the site.",
            ),
            ..Self::default()
        }
    }
}

pub fn view(s: &State, t: &Theme) -> El {
    let d = &s.input;
    let column = || col().w(320.0).gap(16.0);
    let inputs = column()
        .child(
            Input::new(&d.email)
                .placeholder("Email")
                .on_edit(|s: &mut State| &mut s.input.email)
                .build(t)
                .key("email"),
        )
        .child(
            Input::new(&d.disabled)
                .placeholder("Email")
                .disabled(true)
                .build(t)
                .key("disabled"),
        );
    let textarea = column().child(
        Textarea::new(&d.message)
            .placeholder("Type your message here.")
            .on_edit(|s: &mut State| &mut s.input.message)
            .build(t)
            .key("message"),
    );
    let groups = column()
        .child(
            InputGroup::new(&d.search)
                .placeholder("Search...")
                .on_edit(|s: &mut State| &mut s.input.search)
                .addon(InputGroupAddon::new(AddonAlign::InlineStart).icon(Icon::Search))
                .addon(InputGroupAddon::new(AddonAlign::InlineEnd).text("12 results"))
                .build(t)
                .key("search"),
        )
        .child(
            InputGroup::new(&d.url)
                .placeholder("example.com")
                .on_edit(|s: &mut State| &mut s.input.url)
                .addon(InputGroupAddon::new(AddonAlign::InlineStart).text("https://"))
                .addon(
                    InputGroupAddon::new(AddonAlign::InlineEnd)
                        .button(InputGroupButton::icon_button(Icon::Copy, "Copy")),
                )
                .build(t)
                .key("url"),
        )
        .child(
            InputGroup::textarea(&d.prompt)
                .placeholder("Ask, search or chat...")
                .on_edit(|s: &mut State| &mut s.input.prompt)
                .addon(
                    InputGroupAddon::new(AddonAlign::BlockEnd)
                        .button(InputGroupButton::new("Add context").icon(Icon::Plus)),
                )
                .build(t)
                .key("prompt"),
        );
    col()
        .child(section("Input", t, preview(t).child(inputs)))
        .child(section("Textarea", t, preview(t).child(textarea)))
        .child(section("Input group", t, preview(t).child(groups)))
}
