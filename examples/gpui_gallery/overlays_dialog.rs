use std::time::Duration;

use gpui::{
    App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use notra_ui::components::overlays::toast::demo;
use notra_ui::gpui::overlays::{Dialog, Sheet, Toast, Toaster, toast};
use notra_ui::gpui::{ActiveTheme, Button, ButtonVariant};

use crate::overlays_demo::outline_button;
use crate::{Page, preview, section};

pub fn sonner_page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "sonner",
        title: "Sonner",
        view: cx.new(|_| SonnerPage).into(),
    }
}

struct SonnerPage;

impl SonnerPage {
    fn publish(&mut self, cx: &mut Context<Self>) {
        toast(cx, demo::loading());
        cx.spawn(async move |_, cx| {
            cx.background_executor().timer(Duration::from_secs(2)).await;
            cx.update(|cx| toast(cx, demo::published()));
        })
        .detach();
    }
}

impl Render for SonnerPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let button = |id: &'static str, make: fn() -> Toast, cx: &mut Context<Self>| {
            outline_button(id, id, false, cx)
                .on_click(cx.listener(move |_, _, _, cx| toast(cx, make())))
        };
        let types = preview(cx)
            .items_start()
            .min_h(px(320.0))
            .child(button("Default", demo::default, cx))
            .child(button("Success", demo::success, cx))
            .child(button("Info", demo::info, cx))
            .child(button("Warning", demo::warning, cx))
            .child(button("Error", demo::error, cx))
            .child(
                outline_button("Loading", "Loading", false, cx)
                    .on_click(cx.listener(|this, _, _, cx| this.publish(cx))),
            );
        let actions = preview(cx)
            .items_start()
            .min_h(px(256.0))
            .child(button("With action", demo::with_action, cx))
            .child(button("With cancel", demo::with_cancel, cx));
        div()
            .flex()
            .flex_col()
            .child(section("Types", cx).child(types))
            .child(section("Action and cancel", cx).child(actions))
            .child(Toaster::new("toaster").on_action(|t, _, cx| {
                if t.id == "trash" {
                    toast(cx, demo::restored());
                }
            }))
    }
}

pub fn dialog_page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "dialog",
        title: "Dialog",
        view: cx.new(|_| ModalPage::default()).into(),
    }
}

pub fn sheet_page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "sheet",
        title: "Sheet",
        view: cx
            .new(|_| ModalPage {
                sheet: true,
                ..ModalPage::default()
            })
            .into(),
    }
}

#[derive(Default)]
struct ModalPage {
    sheet: bool,
    open: bool,
}

impl Render for ModalPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.notra().muted_foreground;
        let close = cx.listener(|this: &mut Self, _, _, cx| {
            this.open = false;
            cx.notify();
        });
        let close_button = |id: &'static str, cx: &mut Context<Self>| {
            Button::new(id, "Cancel")
                .variant(ButtonVariant::Outline)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.open = false;
                    cx.notify();
                }))
        };
        let body = |text: &'static str| div().text_color(muted).child(SharedString::from(text));
        let trigger = outline_button(
            "open",
            if self.sheet {
                "Open sheet"
            } else {
                "Open dialog"
            },
            self.open,
            cx,
        )
        .on_click(cx.listener(|this, _, _, cx| {
            this.open = true;
            cx.notify();
        }));
        let modal = if self.sheet {
            Sheet::new("sheet", self.open)
                .title("Prospect details")
                .description("devtools.fm, a podcast about developer tools.")
                .child(div().px(px(16.0)).child(body(
                    "Last contacted 12 days ago. Two replies, one meeting booked.",
                )))
                .footer(
                    Button::new("sheet-save", "Save changes").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.open = false;
                            cx.notify();
                        },
                    )),
                )
                .footer(close_button("sheet-cancel", cx))
                .on_close(move |window, cx| close(&gpui::ClickEvent::default(), window, cx))
                .into_any_element()
        } else {
            Dialog::new("dialog", self.open)
                .title("Rename workflow")
                .description("Pick a name your team will recognise in the run history.")
                .child(body("Runs keep their history; only the label changes."))
                .footer(close_button("dialog-cancel", cx))
                .footer(Button::new("dialog-save", "Save").on_click(cx.listener(
                    |this, _, _, cx| {
                        this.open = false;
                        cx.notify();
                    },
                )))
                .on_close(move |window, cx| close(&gpui::ClickEvent::default(), window, cx))
                .into_any_element()
        };
        div()
            .flex()
            .flex_col()
            .child(section("Demo", cx).child(preview(cx).child(trigger)))
            .child(modal)
    }
}
