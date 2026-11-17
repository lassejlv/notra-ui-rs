//! The GPUI gallery: every Notra component on one window.
//!
//! ```sh
//! cargo run --example gpui_gallery --features gpui -- [--page button] [--dark]
//! ```

mod button;
mod composer;
mod input;

use gpui::{
    AnyView, App, AppContext as _, Bounds, Context, FontWeight, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_platform::application;
use notra_ui::gpui::{ActiveTheme, Assets, init, set_appearance};
use notra_ui::{Appearance, fonts};

/// One gallery page: a title and the view that renders it.
pub struct Page {
    pub slug: &'static str,
    pub title: &'static str,
    pub view: AnyView,
}

fn pages(window: &mut Window, cx: &mut App) -> Vec<Page> {
    vec![
        button::page(window, cx),
        composer::page(window, cx),
        input::page(window, cx),
    ]
}

struct Gallery {
    pages: Vec<Page>,
    selected: usize,
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.notra();
        let dark = theme.is_dark();
        let nav = self.pages.iter().enumerate().map(|(index, page)| {
            let selected = index == self.selected;
            div()
                .id(SharedString::from(format!("nav-{}", page.slug)))
                .px(px(10.0))
                .py(px(6.0))
                .rounded(px(8.0))
                .text_size(px(13.0))
                .cursor_pointer()
                .text_color(if selected {
                    theme.foreground
                } else {
                    theme.muted_foreground
                })
                .when(selected, |d| d.bg(theme.muted))
                .hover(|s| s.bg(theme.muted))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.selected = index;
                    cx.notify();
                }))
                .child(page.title)
        });
        let page = &self.pages[self.selected];
        div()
            .size_full()
            .flex()
            .bg(theme.canvas)
            .text_color(theme.foreground)
            .font_family(fonts::SANS)
            .child(
                div()
                    .id("nav")
                    .w(px(220.0))
                    .h_full()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .p(px(12.0))
                    .border_r_1()
                    .border_color(theme.border)
                    .bg(theme.sidebar)
                    .overflow_y_scroll()
                    .child(
                        div()
                            .px(px(10.0))
                            .pb(px(10.0))
                            .text_size(px(15.0))
                            .font_weight(FontWeight(600.0))
                            .child("Notra UI"),
                    )
                    .children(nav)
                    .child(
                        div()
                            .id("appearance")
                            .mt(px(12.0))
                            .px(px(10.0))
                            .py(px(6.0))
                            .rounded(px(8.0))
                            .text_size(px(13.0))
                            .text_color(theme.muted_foreground)
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.muted))
                            .on_click(cx.listener(move |_, _, _, cx| {
                                set_appearance(
                                    cx,
                                    if dark {
                                        Appearance::Light
                                    } else {
                                        Appearance::Dark
                                    },
                                );
                            }))
                            .child(if dark { "Light mode" } else { "Dark mode" }),
                    ),
            )
            .child(
                div()
                    .id("page")
                    .flex_1()
                    .h_full()
                    .overflow_y_scroll()
                    .p(px(40.0))
                    .child(
                        div()
                            .pb(px(24.0))
                            .text_size(px(28.0))
                            .font_weight(FontWeight(500.0))
                            .font_family(fonts::DISPLAY)
                            .child(page.title),
                    )
                    .child(page.view.clone()),
            )
    }
}

/// A titled preview area, like the site's Preview tab.
pub fn section(title: &'static str, cx: &App) -> gpui::Div {
    let theme = cx.notra();
    div().flex().flex_col().gap(px(10.0)).mb(px(28.0)).child(
        div()
            .text_size(px(13.0))
            .font_weight(FontWeight(500.0))
            .text_color(theme.muted_foreground)
            .child(title),
    )
}

/// The bordered preview surface components sit on.
pub fn preview(cx: &App) -> gpui::Div {
    let theme = cx.notra();
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .justify_center()
        .gap(px(12.0))
        .min_h(px(140.0))
        .p(px(32.0))
        .rounded(px(12.0))
        .border_1()
        .border_color(theme.border)
        .bg(theme.canvas)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dark = args.iter().any(|a| a == "--dark");
    let page = args
        .iter()
        .position(|a| a == "--page")
        .and_then(|i| args.get(i + 1).cloned());

    application()
        .with_assets(Assets(()))
        .run(move |cx: &mut App| {
            init(cx);
            if dark {
                set_appearance(cx, Appearance::Dark);
            }
            let bounds = Bounds::centered(None, size(px(1200.0), px(820.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    focus: true,
                    ..Default::default()
                },
                |window, cx| {
                    let pages = pages(window, cx);
                    let selected = page
                        .as_deref()
                        .and_then(|slug| pages.iter().position(|p| p.slug == slug))
                        .unwrap_or(0);
                    cx.new(|_| Gallery { pages, selected })
                },
            )
            .expect("open the gallery window");
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            cx.activate(true);
        });
}
