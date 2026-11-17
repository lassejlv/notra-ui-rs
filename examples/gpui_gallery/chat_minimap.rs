use gpui::{
    App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::*, px,
};
use notra_ui::components::chat_minimap::{
    DEMO_START, DEMO_TURNS, LONG_START, LONG_TURNS, TurnWindow, demo_items,
};
use notra_ui::gpui::{ActiveTheme, ChatMinimap, ChatMinimapSide};

use crate::{Page, section};

pub fn page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "chat-minimap",
        title: "Chat Minimap",
        view: cx.new(|_| ChatMinimapPage::new()).into(),
    }
}

/// The three live examples: default, `side="left"` and fifty turns.
struct ChatMinimapPage {
    windows: [TurnWindow; 3],
}

impl ChatMinimapPage {
    fn new() -> Self {
        Self {
            windows: [
                TurnWindow::new(DEMO_TURNS.len(), DEMO_START),
                TurnWindow::new(DEMO_TURNS.len(), DEMO_START),
                TurnWindow::new(LONG_TURNS, LONG_START),
            ],
        }
    }

    fn example(
        &self,
        slot: usize,
        side: ChatMinimapSide,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.notra();
        let window = self.windows[slot];
        let minimap = ChatMinimap::new(("chat-minimap", slot))
            .side(side)
            .items(demo_items(window))
            .on_select(cx.listener(move |this, index: &usize, _, cx| {
                this.windows[slot].select(*index);
                cx.notify();
            }))
            .previous(
                window.can_previous(),
                cx.listener(move |this, _, _, cx| {
                    this.windows[slot].previous();
                    cx.notify();
                }),
            )
            .next(
                window.can_next(),
                cx.listener(move |this, _, _, cx| {
                    this.windows[slot].next();
                    cx.notify();
                }),
            );
        div()
            .flex()
            .w_full()
            .px(px(24.0))
            .py(px(40.0))
            .rounded(px(12.0))
            .border_1()
            .border_color(theme.border)
            .bg(theme.canvas)
            .map(|d| match side {
                ChatMinimapSide::Left => d.justify_end(),
                ChatMinimapSide::Right => d.justify_start(),
            })
            .child(minimap)
    }
}

impl Render for ChatMinimapPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .child(section("Default", cx).child(self.example(0, ChatMinimapSide::Right, cx)))
            .child(section("Left", cx).child(self.example(1, ChatMinimapSide::Left, cx)))
            .child(section("Long", cx).child(self.example(2, ChatMinimapSide::Right, cx)))
    }
}
