use gpui::{
    App, AppContext as _, Context, FontWeight, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, prelude::*, px,
};
use notra_ui::components::card::demo;
use notra_ui::gpui::{
    ActiveTheme, Button, ButtonSize, ButtonVariant, Card, CardEmpty, CardGrid, CardReveal,
    CardSection, CardVariant,
};

use crate::{Page, section};

pub fn page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "card",
        title: "Card",
        view: cx.new(|_| CardPage::default()).into(),
    }
}

#[derive(Default)]
struct CardPage {
    revealed: bool,
}

/// shadcn's outline button at `size="sm"`, which the card demo puts in the action slot.
fn view_all(id: &'static str, cx: &App) -> impl IntoElement {
    let theme = cx.notra();
    let dark = theme.is_dark();
    div()
        .id(id)
        .flex()
        .items_center()
        .h(px(28.0))
        .px(px(10.0))
        .rounded(px(6.0))
        .border_1()
        .border_color(if dark { theme.input } else { theme.border })
        .bg(if dark {
            theme.input.fade(0.3)
        } else {
            theme.background
        })
        .text_size(px(12.8))
        .font_weight(FontWeight(500.0))
        .text_color(theme.foreground)
        .cursor_pointer()
        .hover(|s| {
            s.bg(if dark {
                theme.input.fade(0.5)
            } else {
                theme.muted
            })
        })
        .child("View all")
}

fn engines(cx: &App) -> impl IntoElement {
    let theme = cx.notra();
    let last = demo::ENGINES.len() - 1;
    div()
        .flex()
        .flex_col()
        .children(demo::ENGINES.iter().enumerate().map(|(i, (name, rate))| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .py(px(demo::ROW_PADDING_Y))
                .when(i < last, |d| d.border_b_1().border_color(theme.border))
                .child(div().font_weight(FontWeight(500.0)).child(*name))
                .child(div().text_color(theme.muted_foreground).child(*rate))
        }))
}

impl Render for CardPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.notra();
        let demo_grid = CardGrid::new(2)
            .child(
                Card::new("mention-rate", "Mention rate")
                    .variant(CardVariant::Table)
                    .hint(demo::HINT)
                    .readout("Last 7 days")
                    .action(view_all("view-all", cx))
                    .child(engines(cx)),
            )
            .child(
                Card::new("conversations", "Conversations")
                    .variant(CardVariant::Table)
                    .child(CardEmpty::new("collecting", "Collecting answers").busy(true)),
            );
        let variants = CardGrid::new(2)
            .child(
                Card::new("panel", "Visibility")
                    .variant(CardVariant::Panel)
                    .description("How often engines cite your pages.")
                    .readout("30 days")
                    .child(engines(cx)),
            )
            .child(
                Card::new("compact", "Top engines")
                    .variant(CardVariant::TableCompact)
                    .readout("4 engines")
                    .child(engines(cx)),
            )
            .child(
                Card::new("flat", "Flat card")
                    .hint(demo::HINT)
                    .readout("last 7 days")
                    .child(engines(cx)),
            )
            .child(
                Card::new("bare", "Bare body")
                    .variant(CardVariant::Panel)
                    .bare_body(true)
                    .child(engines(cx)),
            );
        let empties = CardGrid::new(2)
            .child(
                Card::new("empty-action", "Citations")
                    .variant(CardVariant::Table)
                    .child(
                        CardEmpty::new("no-citations", "no citations yet").action(
                            Button::new("connect", "Connect a source")
                                .size(ButtonSize::Sm)
                                .variant(ButtonVariant::Outline),
                        ),
                    ),
            )
            .child(
                Card::new("empty-preview", "Answers")
                    .variant(CardVariant::Table)
                    .child(
                        CardEmpty::new("preview-empty", "Waiting for the first run")
                            .preview(div().p(px(8.0)).child(engines(cx))),
                    ),
            );
        let revealed = self.revealed;
        let reveal = div()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                Button::new("replay", if revealed { "Hide" } else { "Reveal" })
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.revealed = !this.revealed;
                        cx.notify();
                    })),
            )
            .child(CardGrid::new(3).children((0..3).map(|i| {
                CardReveal::new(SharedString::from(format!("reveal-{i}")), revealed)
                    .order(i)
                    .child(
                        Card::new(
                            SharedString::from(format!("reveal-card-{i}")),
                            format!("Module {}", i + 1),
                        )
                        .variant(CardVariant::TableCompact)
                        .child(div().h(px(48.0))),
                    )
            })));
        let heading = CardSection::new("section", "Engines")
            .description("Answers per engine over the last week.")
            .hint(demo::HINT)
            .readout("updated just now")
            .action(view_all("section-view-all", cx))
            .child(engines(cx));
        div()
            .flex()
            .flex_col()
            .text_color(theme.foreground)
            .child(section("Demo", cx).child(div().max_w(px(768.0)).child(demo_grid)))
            .child(section("Variants", cx).child(div().max_w(px(768.0)).child(variants)))
            .child(section("Empty states", cx).child(div().max_w(px(768.0)).child(empties)))
            .child(section("Section heading", cx).child(div().max_w(px(768.0)).child(heading)))
            .child(section("Reveal", cx).child(div().max_w(px(768.0)).child(reveal)))
    }
}
