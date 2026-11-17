use gpui::{
    App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString, Styled,
    Window, div, px,
};
use notra_ui::components::overlays::menu::demo_entries;
use notra_ui::components::overlays::select::{demo_flat, demo_grouped};
use notra_ui::gpui::ActiveTheme;
use notra_ui::gpui::overlays::{DropdownMenu, MenuItem, Select, SelectSize, SelectVariant};

use crate::overlays_demo::outline_button;
use crate::{Page, preview, section};

pub fn dropdown_menu_page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "dropdown-menu",
        title: "Dropdown Menu",
        view: cx
            .new(|_| MenuPage {
                activity: true,
                view: "team".into(),
            })
            .into(),
    }
}

struct MenuPage {
    activity: bool,
    view: String,
}

impl Render for MenuPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let menu = DropdownMenu::new("demo-menu", demo_entries(self.activity, &self.view))
            .trigger(|open, _, cx| {
                outline_button("open-menu", "Open menu", open, cx).into_any_element()
            })
            .width(224.0)
            .on_select(cx.listener(|this, item: &MenuItem, _, cx| {
                match item.id.as_str() {
                    "activity" => this.activity = !this.activity,
                    "team" | "personal" => this.view = item.id.clone(),
                    _ => {}
                }
                cx.notify();
            }));
        div().flex().flex_col().child(
            section("Demo", cx).child(preview(cx).items_start().min_h(px(448.0)).child(menu)),
        )
    }
}

pub fn select_page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "select",
        title: "Select",
        view: cx
            .new(|_| SelectPage {
                frequency: "weekly".into(),
                compact: "weekly".into(),
            })
            .into(),
    }
}

struct SelectPage {
    frequency: String,
    compact: String,
}

impl Render for SelectPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label = |text: &'static str, cx: &App| {
            div()
                .text_size(px(14.0))
                .line_height(px(20.0))
                .text_color(cx.notra().foreground)
                .child(SharedString::from(text))
        };
        let frequency = Select::new("frequency", demo_grouped())
            .value(Some(self.frequency.clone()))
            .width(192.0)
            .on_change(cx.listener(|this, value: &str, _, cx| {
                this.frequency = value.into();
                cx.notify();
            }));
        let compact = Select::new("compact", demo_flat())
            .value(Some(self.compact.clone()))
            .variant(SelectVariant::Ghost)
            .size(SelectSize::Sm)
            .on_change(cx.listener(|this, value: &str, _, cx| {
                this.compact = value.into();
                cx.notify();
            }));
        div().flex().flex_col().child(
            section("Demo", cx).child(
                preview(cx)
                    .items_start()
                    .gap(px(32.0))
                    .min_h(px(320.0))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(label("Frequency", cx))
                            .child(frequency),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(label("Compact property", cx))
                            .child(compact),
                    ),
            ),
        )
    }
}
