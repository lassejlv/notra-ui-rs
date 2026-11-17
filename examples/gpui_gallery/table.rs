use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, Context, FontWeight, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::*, px,
};
use notra_ui::Icon;
use notra_ui::components::table::{Align, ColumnWidth, Paging, demo};
use notra_ui::gpui::data_parts::{Badge, BadgeVariant, EmptyState, glyph};
use notra_ui::gpui::{ActiveTheme, Button, ButtonSize, DataTable, Table, TableHead};

use crate::{Page, section};

pub fn page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "table",
        title: "Table",
        view: cx
            .new(|_| TablePage {
                prompts: Rc::new(demo::prompts()),
                visits: Rc::new(demo::visits(0)),
                loading: false,
                empty_filled: false,
            })
            .into(),
    }
}

struct TablePage {
    prompts: Rc<Vec<demo::Prompt>>,
    visits: Rc<Vec<demo::Visit>>,
    loading: bool,
    empty_filled: bool,
}

impl TablePage {
    fn load_more(&mut self, cx: &mut Context<Self>) {
        if self.loading || self.visits.len() >= demo::MAX_VISITS {
            return;
        }
        self.loading = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(demo::LOAD_DELAY).await;
            this.update(cx, |page, cx| {
                let mut rows = (*page.visits).clone();
                rows.extend(demo::visits(rows.len()));
                page.visits = Rc::new(rows);
                page.loading = false;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

fn text(s: impl Into<SharedString>) -> AnyElement {
    div().child(s.into()).into_any_element()
}

fn visit_cell(v: &demo::Visit, key: &str, cx: &App) -> AnyElement {
    match key {
        "bot" => div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(glyph(v.provider, 16.0, cx.notra().foreground))
            .child(v.bot)
            .into_any_element(),
        "path" => text(v.path),
        _ => text(v.status.to_string()),
    }
}

impl Render for TablePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.notra();
        let prompts = self.prompts.clone();
        let sorted = self.prompts.clone();
        let ids = self.prompts.clone();
        let demo_table = DataTable::new("prompts", demo::prompt_columns(), prompts.len())
            .row_id(move |i| ids[i].id.clone().into())
            .sort_value(move |i, key| sorted[i].sort_value(key))
            .cell(move |i, key, _, _| {
                let p = &prompts[i];
                match key {
                    "prompt" => text(p.prompt),
                    "engine" => text(p.engine),
                    "status" => if p.mentioned {
                        Badge::new("Mentioned", BadgeVariant::Secondary)
                    } else {
                        Badge::new("Not mentioned", BadgeVariant::Outline)
                    }
                    .into_any_element(),
                    _ => text(p.position_text()),
                }
            })
            .selectable(true)
            .resizable(true)
            .height(11.0 * 48.0)
            .paging(Paging::client(1, 10).item_label("prompts"));

        let filled = self.empty_filled;
        let samples = Rc::new(if filled {
            demo::sample_visits()
        } else {
            Vec::new()
        });
        let sample_cells = samples.clone();
        let empty = DataTable::new("visits-empty", demo::visit_columns(), samples.len())
            .row_id(move |i| samples[i].id.clone().into())
            .cell(move |i, key, _, cx| visit_cell(&sample_cells[i], key, cx))
            .height(300.0)
            .empty_state(
                EmptyState::new(Icon::Bot, "No crawler visits yet")
                    .description(
                        "Add the tracking snippet to your site and AI crawlers show up here as they read your pages.",
                    )
                    .child(
                        Button::new("add-snippet", "Add snippet")
                            .size(ButtonSize::Sm)
                            .icon(Icon::Plus)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.empty_filled = true;
                                cx.notify();
                            })),
                    ),
            );

        let visits = self.visits.clone();
        let visit_ids = self.visits.clone();
        let has_more = self.visits.len() < demo::MAX_VISITS;
        let mut infinite = DataTable::new("visits", demo::visit_columns(), visits.len())
            .row_id(move |i| visit_ids[i].id.clone().into())
            .cell(move |i, key, _, cx| visit_cell(&visits[i], key, cx))
            .height(400.0)
            .loading(self.loading)
            .loading_more(self.loading);
        if has_more && !self.loading {
            let this = cx.entity();
            infinite = infinite
                .on_end_reached(move |_, cx| this.update(cx, |page, cx| page.load_more(cx)));
        }

        let toolbar = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(16.0))
            .min_h(px(44.0))
            .px(px(16.0))
            .py(px(8.0))
            .children(
                demo::SUMMARY
                    .iter()
                    .enumerate()
                    .map(|(i, (label, up, down))| {
                        let chip = |n: u32, rising: bool| {
                            let (bg, ink) = demo::chip(&theme, rising);
                            div()
                                .flex()
                                .items_center()
                                .gap(px(2.0))
                                .px(px(6.0))
                                .rounded(px(6.0))
                                .bg(bg)
                                .text_color(ink)
                                .child(glyph(
                                    if rising {
                                        Icon::ChevronUp
                                    } else {
                                        Icon::ChevronDown
                                    },
                                    14.0,
                                    ink,
                                ))
                                .child(n.to_string())
                        };
                        div()
                            .flex()
                            .items_center()
                            .gap(px(16.0))
                            .when(i > 0, |d| {
                                d.child(div().w(px(1.0)).h(px(12.0)).bg(theme.border))
                            })
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(div().text_color(theme.muted_foreground).child(*label))
                                    .when(*up > 0, |d| d.child(chip(*up, true)))
                                    .when(*down > 0, |d| d.child(chip(*down, false))),
                            )
                    }),
            );
        let changes = DataTable::new("changes", demo::change_columns(), demo::CHANGES.len())
            .row_id(|i| demo::CHANGES[i].id.into())
            .cell(move |i, key, _, _| {
                let c = &demo::CHANGES[i];
                match key {
                    "kind" => div()
                        .font_weight(FontWeight(500.0))
                        .text_color(c.kind.color(&theme))
                        .child(c.kind.label())
                        .into_any_element(),
                    "engine" => text(c.engine),
                    "prompt" => text(c.prompt),
                    _ => text(c.position),
                }
            })
            .toolbar(toolbar);

        let mut primitives = Table::new(
            "invoices",
            demo::INVOICE_HEADERS
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    let head = TableHead::new(*h);
                    if i == 3 {
                        head.align(Align::Right)
                    } else {
                        head
                    }
                })
                .collect(),
        );
        for invoice in demo::INVOICES {
            primitives = primitives.row(invoice.iter().enumerate().map(|(i, v)| {
                div()
                    .when(i == 0, |d| d.font_weight(FontWeight(500.0)))
                    .child(*v)
            }));
        }

        let skeleton = DataTable::skeleton(
            "skeleton",
            ["1fr", "1fr", "8rem", "6rem"]
                .iter()
                .enumerate()
                .map(|(i, w)| {
                    notra_ui::components::table::Column::new(format!("c{i}"), "").width(match *w {
                        "8rem" => ColumnWidth::Rem(8.0),
                        "6rem" => ColumnWidth::Rem(6.0),
                        _ => ColumnWidth::Fr(1.0),
                    })
                })
                .collect(),
            5,
        );

        let wide = |e: AnyElement| div().max_w(px(848.0)).w_full().child(e);
        div()
            .flex()
            .flex_col()
            .text_color(theme.foreground)
            .child(section("Data table", cx).child(wide(demo_table.into_any_element())))
            .child(section("Empty", cx).child(div().max_w(px(672.0)).child(empty)))
            .child(section("Infinite", cx).child(wide(infinite.into_any_element())))
            .child(section("Toolbar", cx).child(div().max_w(px(768.0)).child(changes)))
            .child(section("Primitives", cx).child(div().max_w(px(624.0)).child(primitives)))
            .child(section("Skeleton", cx).child(wide(skeleton.into_any_element())))
    }
}
