use std::time::Duration;

use gpui::{
    App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window, div,
};
use notra_ui::Icon;
use notra_ui::gpui::{Button, ButtonSize, ButtonVariant};

use crate::{Page, preview, section};

pub fn page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "button",
        title: "Button",
        view: cx.new(|_| ButtonPage::default()).into(),
    }
}

#[derive(Default)]
struct ButtonPage {
    loading: [bool; 5],
    progress: Option<f32>,
}

impl ButtonPage {
    fn upload(&mut self, cx: &mut Context<Self>) {
        if self.progress.is_some() {
            return;
        }
        self.progress = Some(0.0);
        cx.notify();
        cx.spawn(async move |this, cx| {
            for step in 1..=10 {
                cx.background_executor()
                    .timer(Duration::from_millis(280))
                    .await;
                let done = step == 10;
                if this
                    .update(cx, |page, cx| {
                        page.progress = (!done).then_some(step as f32 * 10.0);
                        cx.notify();
                    })
                    .is_err()
                {
                    return;
                }
            }
        })
        .detach();
    }
}

impl Render for ButtonPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let variants = ButtonVariant::ALL.iter().map(|&variant| {
            let label = match variant {
                ButtonVariant::Primary => "Primary",
                ButtonVariant::Secondary => "Secondary",
                ButtonVariant::Outline => "Outline",
                ButtonVariant::Ghost => "Ghost",
                ButtonVariant::GhostMuted => "Ghost muted",
                ButtonVariant::Destructive => "Destructive",
                ButtonVariant::Link => "Link",
            };
            Button::new(label, label).variant(variant)
        });
        let sizes = [
            ("Extra small", ButtonSize::Xs),
            ("Small", ButtonSize::Sm),
            ("Default", ButtonSize::Default),
            ("Large", ButtonSize::Lg),
        ]
        .map(|(label, size)| Button::new(label, label).size(size));

        let loading = |index: usize, button: notra_ui::gpui::Button| {
            button
                .loading(self.loading[index])
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.loading[index] = true;
                    cx.notify();
                    cx.spawn(async move |this, cx| {
                        cx.background_executor()
                            .timer(Duration::from_millis(1800))
                            .await;
                        this.update(cx, |page, cx| {
                            page.loading[index] = false;
                            cx.notify();
                        })
                        .ok();
                    })
                    .detach();
                }))
        };

        div()
            .flex()
            .flex_col()
            .child(section("Variants", cx).child(preview(cx).children(variants)))
            .child(
                section("Sizes", cx).child(
                    preview(cx)
                        .children(sizes)
                        .child(Button::new("disabled", "Disabled").disabled(true))
                        .child(
                            Button::new("disabled-secondary", "Disabled")
                                .variant(ButtonVariant::Secondary)
                                .disabled(true),
                        ),
                ),
            )
            .child(
                section("Loading", cx).child(
                    preview(cx)
                        .child(loading(0, Button::new("save", "Save changes")))
                        .child(loading(
                            1,
                            Button::new("continue", "Continue")
                                .variant(ButtonVariant::Secondary)
                                .icon_end(Icon::ArrowRight),
                        ))
                        .child(loading(
                            2,
                            Button::new("generate", "Generate")
                                .variant(ButtonVariant::Outline)
                                .size(ButtonSize::Sm),
                        ))
                        .child(loading(
                            3,
                            Button::new("delete", "Delete").variant(ButtonVariant::Destructive),
                        ))
                        .child(loading(4, Button::icon_button("send", Icon::ArrowUp))),
                ),
            )
            .child(
                section("Progress", cx).child(
                    preview(cx).child(
                        Button::new("upload", "Upload file")
                            .variant(ButtonVariant::Secondary)
                            .icon(Icon::ArrowUp)
                            .progress(self.progress)
                            .on_click(cx.listener(|this, _, _, cx| this.upload(cx))),
                    ),
                ),
            )
    }
}
