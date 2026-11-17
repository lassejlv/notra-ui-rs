use notra_ui::Theme;
use notra_ui::components::card::demo;
use notra_ui::rune::{
    Button, ButtonSize, ButtonVariant, Card, CardEmpty, CardSection, CardVariant, ToRune,
    card_grid, card_reveal, sans,
};
use rune::{El, col, el, row, text};

use crate::{State, section};

#[derive(Default)]
pub struct Demo {
    pub revealed: bool,
}

/// shadcn's outline button at `size="sm"`, which the card demo puts in the action slot.
fn view_all(t: &Theme) -> El {
    let dark = t.is_dark();
    row()
        .h(28.0)
        .px(10.0)
        .center_y()
        .radius(6.0)
        .border(1.0, if dark { t.input } else { t.border }.rune())
        .fill(
            if dark {
                t.input.fade(0.3)
            } else {
                t.background
            }
            .rune(),
        )
        .hover(if dark { t.input.fade(0.2) } else { t.muted }.rune())
        .role("button")
        .child(
            text("View all")
                .font(sans(12.8, 500.0, 18.0))
                .color(t.foreground.rune()),
        )
}

fn engines(t: &Theme) -> El {
    let last = demo::ENGINES.len() - 1;
    col().children(demo::ENGINES.iter().enumerate().map(|(i, (name, rate))| {
        let mut line = row()
            .key(format!("engine-{name}"))
            .justify_between()
            .center_y()
            .py(demo::ROW_PADDING_Y);
        if i < last {
            line = line.border_sides([0.0, 0.0, 1.0, 0.0], t.border.rune());
        }
        line.child(
            text(*name)
                .font(sans(14.0, 500.0, 20.0))
                .color(t.foreground.rune()),
        )
        .child(
            text(*rate)
                .font(sans(14.0, 400.0, 20.0))
                .color(t.muted_foreground.rune()),
        )
    }))
}

pub fn view(s: &State, t: &Theme) -> El {
    let demo_grid = card_grid(
        2,
        vec![
            Card::new("Mention rate")
                .variant(CardVariant::Table)
                .hint(demo::HINT)
                .readout("Last 7 days")
                .action(view_all(t).key("view-all"))
                .child(engines(t))
                .build(t)
                .key("mention-rate"),
            Card::new("Conversations")
                .variant(CardVariant::Table)
                .child(CardEmpty::new("Collecting answers").busy(true).build(t))
                .build(t)
                .key("conversations"),
        ],
    );
    let variants = card_grid(
        2,
        vec![
            Card::new("Visibility")
                .variant(CardVariant::Panel)
                .description("How often engines cite your pages.")
                .readout("30 days")
                .child(engines(t))
                .build(t)
                .key("panel"),
            Card::new("Top engines")
                .variant(CardVariant::TableCompact)
                .readout("4 engines")
                .child(engines(t))
                .build(t)
                .key("compact"),
            Card::new("Flat card")
                .hint(demo::HINT)
                .readout("last 7 days")
                .child(engines(t))
                .build(t)
                .key("flat"),
            Card::new("Bare body")
                .variant(CardVariant::Panel)
                .bare_body(true)
                .child(engines(t))
                .build(t)
                .key("bare"),
        ],
    );
    let empties = card_grid(
        2,
        vec![
            Card::new("Citations")
                .variant(CardVariant::Table)
                .child(
                    CardEmpty::new("no citations yet")
                        .action(
                            Button::new("Connect a source")
                                .size(ButtonSize::Sm)
                                .variant(ButtonVariant::Outline)
                                .build(t)
                                .key("connect"),
                        )
                        .build(t),
                )
                .build(t)
                .key("empty-action"),
            Card::new("Answers")
                .variant(CardVariant::Table)
                .child(
                    CardEmpty::new("Waiting for the first run")
                        .preview(el().padding(8.0).child(engines(t)))
                        .build(t),
                )
                .build(t)
                .key("empty-preview"),
        ],
    );
    let revealed = s.card.revealed;
    let reveal = col()
        .gap(12.0)
        .items_start()
        .child(
            Button::new(if revealed { "Hide" } else { "Reveal" })
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .on_press(|s: &mut State| s.card.revealed = !s.card.revealed)
                .build(t)
                .key("replay"),
        )
        .child(card_grid(
            3,
            (0..3)
                .map(|i| {
                    card_reveal(
                        revealed,
                        i,
                        Card::new(format!("Module {}", i + 1))
                            .variant(CardVariant::TableCompact)
                            .child(el().h(48.0))
                            .build(t),
                    )
                    .key(format!("reveal-{i}"))
                })
                .collect(),
        ));
    let heading = CardSection::new("Engines")
        .description("Answers per engine over the last week.")
        .hint(demo::HINT)
        .readout("updated just now")
        .action(view_all(t).key("section-view-all"))
        .child(engines(t))
        .build(t);
    let width = |e: El| el().max_w(768.0).w_full().child(e);
    col()
        .child(section("Demo", t, width(demo_grid)))
        .child(section("Variants", t, width(variants)))
        .child(section("Empty states", t, width(empties)))
        .child(section("Section heading", t, width(heading)))
        .child(section("Reveal", t, width(reveal)))
}
