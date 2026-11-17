use gpui::{
    App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use notra_ui::components::package_command::demo;
use notra_ui::gpui::PackageCommand;

use crate::{Page, section};

pub fn page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "package-command",
        title: "Package Command",
        view: cx.new(|_| PackageCommandPage).into(),
    }
}

struct PackageCommandPage;

impl Render for PackageCommandPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        section("Demo", cx).child(
            div()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .max_w(px(528.0))
                .w_full()
                .children(
                    demo::COMMANDS
                        .iter()
                        .enumerate()
                        .map(|(i, command)| PackageCommand::new(("package-command", i), *command)),
                ),
        )
    }
}
