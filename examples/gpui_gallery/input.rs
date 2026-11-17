use gpui::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    px,
};
use notra_ui::Icon;
use notra_ui::components::input::AddonAlign;
use notra_ui::gpui::{Input, InputGroup, InputGroupAddon, InputGroupButton, TextField, Textarea};

use crate::{Page, preview, section};

pub fn page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "input",
        title: "Input",
        view: cx.new(InputPage::new).into(),
    }
}

struct InputPage {
    email: Entity<TextField>,
    disabled: Entity<TextField>,
    message: Entity<TextField>,
    search: Entity<TextField>,
    url: Entity<TextField>,
    prompt: Entity<TextField>,
}

impl InputPage {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            email: cx.new(|cx| TextField::new(cx).placeholder("Email").label("Email")),
            disabled: cx.new(|cx| {
                TextField::new(cx)
                    .placeholder("Email")
                    .label("Email")
                    .disabled(true)
            }),
            message: cx.new(|cx| {
                TextField::new(cx)
                    .multiline()
                    .placeholder("Type your message here.")
                    .label("Message")
            }),
            search: cx.new(|cx| TextField::new(cx).placeholder("Search...").label("Search")),
            url: cx.new(|cx| TextField::new(cx).placeholder("example.com").label("URL")),
            prompt: cx.new(|cx| {
                TextField::new(cx)
                    .multiline()
                    .placeholder("Ask, search or chat...")
                    .label("Prompt")
            }),
        }
    }
}

impl Render for InputPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let column = || div().w(px(320.)).flex().flex_col().gap(px(16.));
        div()
            .flex()
            .flex_col()
            .child(
                section("Input", cx).child(
                    preview(cx).child(
                        column()
                            .child(Input::new(self.email.clone()))
                            .child(Input::new(self.disabled.clone())),
                    ),
                ),
            )
            .child(
                section("Textarea", cx)
                    .child(preview(cx).child(column().child(Textarea::new(self.message.clone())))),
            )
            .child(
                section("Input group", cx).child(
                    preview(cx).child(
                        column()
                            .child(
                                InputGroup::new(self.search.clone())
                                    .addon(
                                        InputGroupAddon::new(AddonAlign::InlineStart)
                                            .icon(Icon::Search),
                                    )
                                    .addon(
                                        InputGroupAddon::new(AddonAlign::InlineEnd)
                                            .text("12 results"),
                                    ),
                            )
                            .child(
                                InputGroup::new(self.url.clone())
                                    .addon(
                                        InputGroupAddon::new(AddonAlign::InlineStart)
                                            .text("https://"),
                                    )
                                    .addon(InputGroupAddon::new(AddonAlign::InlineEnd).button(
                                        InputGroupButton::icon_button(
                                            "copy-url",
                                            Icon::Copy,
                                            "Copy",
                                        ),
                                    )),
                            )
                            .child(
                                InputGroup::new(self.prompt.clone()).addon(
                                    InputGroupAddon::new(AddonAlign::BlockEnd).button(
                                        InputGroupButton::new("add-context", "Add context")
                                            .icon(Icon::Plus),
                                    ),
                                ),
                            ),
                    ),
                ),
            )
    }
}
