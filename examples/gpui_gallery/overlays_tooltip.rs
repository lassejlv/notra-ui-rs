//! Tooltip, Duotone Tooltip, Popover and Hover Card pages.

use std::time::Duration;

use gpui::{
    App, AppContext as _, Context, FontWeight, IntoElement, ParentElement, Render, Styled, Window,
    div, prelude::*, px,
};
use notra_ui::components::overlays::Side;
use notra_ui::gpui::ActiveTheme;
use notra_ui::gpui::overlays::popover::{
    HoverCard, Popover, popover_description, popover_header, popover_title,
};
use notra_ui::gpui::overlays::tooltip::{DuotoneTooltip, Tooltip, TooltipDelays, TooltipProvider};
use notra_ui::gpui::shadows;
use notra_ui::{Color, Icon, Shadow};

use crate::overlays_demo::{ghost_icon_button, outline_button};
use crate::{Page, preview, section};

const SIDES: [(Side, &str); 4] = [
    (Side::Top, "top"),
    (Side::Right, "right"),
    (Side::Bottom, "bottom"),
    (Side::Left, "left"),
];

/// The toolbar example: icon, label and shortcut keys.
const TOOLBAR: [(Icon, &str, &[&str]); 7] = [
    (Icon::Bold, "Bold", &["⌘", "B"]),
    (Icon::Italic, "Italic", &["⌘", "I"]),
    (Icon::Underline, "Underline", &["⌘", "U"]),
    (Icon::Link, "Add link", &["⌘", "K"]),
    (Icon::List, "Bulleted list", &[]),
    (Icon::Quote, "Quote", &[]),
    (Icon::Code, "Inline code", &["⌘", "E"]),
];

pub fn pages(_window: &mut Window, cx: &mut App) -> Vec<Page> {
    vec![
        Page {
            slug: "tooltip",
            title: "Tooltip",
            view: cx.new(|_| TooltipPage).into(),
        },
        Page {
            slug: "duotone-tooltip",
            title: "Duotone Tooltip",
            view: cx.new(|_| DuotonePage).into(),
        },
        Page {
            slug: "popover",
            title: "Popover",
            view: cx.new(|_| PopoverPage).into(),
        },
        Page {
            slug: "hover-card",
            title: "Hover Card",
            view: cx.new(|_| HoverCardPage).into(),
        },
    ]
}

struct TooltipPage;

impl Render for TooltipPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.notra();
        let sides = TooltipProvider::new("sides", TooltipDelays::PROVIDER, window, cx);
        let toolbar = TooltipProvider::new(
            "toolbar",
            TooltipDelays::PROVIDER.with_open(Duration::from_millis(400)),
            window,
            cx,
        );
        let side_triggers = SIDES.iter().map(|(side, name)| {
            let label = format!("{}{}", name[..1].to_uppercase(), &name[1..]);
            Tooltip::new(
                *name,
                outline_button(*name, label, false, cx),
                format!("Opens on the {name}"),
            )
            .side(*side)
            .provider(&sides)
        });
        let buttons = TOOLBAR.iter().map(|(icon, label, keys)| {
            Tooltip::new(*label, ghost_icon_button(*label, *icon, cx), *label)
                .keys(keys)
                .provider(&toolbar)
        });
        div()
            .child(
                section("Sides", cx).child(
                    preview(cx)
                        .p(px(48.0))
                        .children(side_triggers)
                        .child(sides.popup()),
                ),
            )
            .child(
                section("Toolbar", cx).child(
                    preview(cx).p(px(40.0)).child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .p(px(4.0))
                            .rounded(px(12.0))
                            .border_1()
                            .border_color(theme.border)
                            .bg(theme.background)
                            .shadow(shadows(&[Shadow::drop(
                                1.0,
                                2.0,
                                0.0,
                                Color::rgba(0x0000000d),
                            )]))
                            .children(buttons)
                            .child(toolbar.popup()),
                    ),
                ),
            )
    }
}

struct DuotonePage;

impl Render for DuotonePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.notra();
        section("Preview", cx).child(
            preview(cx)
                .items_start()
                .rounded(px(16.0))
                .bg(theme.muted.fade(0.4))
                .pt(px(40.0))
                .pb(px(176.0))
                .child(
                    DuotoneTooltip::new(
                        "duotone",
                        "Hover me",
                        "Weekly summary",
                        "Visits grew 18% compared to the previous week, led by returning readers.",
                    )
                    .aside("Last 7 days"),
                ),
        )
    }
}

struct PopoverPage;

impl Render for PopoverPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        section("Preview", cx).child(
            preview(cx).items_start().pt(px(40.0)).pb(px(200.0)).child(
                Popover::new("dimensions")
                    .trigger(|open, _, cx| {
                        outline_button("open-popover", "Open popover", open, cx).into_any_element()
                    })
                    .content(|_, cx| {
                        let theme = *cx.notra();
                        let row = |label: &'static str, value: &'static str| {
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(div().text_color(theme.muted_foreground).child(label))
                                .child(div().font_weight(FontWeight(500.0)).child(value))
                        };
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(10.0))
                            .child(popover_header(
                                "Dimensions",
                                "Set the dimensions for the layer.",
                                cx,
                            ))
                            .child(row("Width", "100%"))
                            .child(row("Height", "25px"))
                            .into_any_element()
                    }),
            ),
        )
    }
}

struct HoverCardPage;

impl Render for HoverCardPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.notra();
        let link = div()
            .id("nextjs-link")
            .text_size(px(14.0))
            .line_height(px(20.0))
            .font_weight(FontWeight(500.0))
            .text_color(theme.foreground)
            .cursor_pointer()
            .hover(|s| s.underline())
            .child("@nextjs");
        section("Preview", cx).child(preview(cx).items_start().pt(px(40.0)).pb(px(200.0)).child(
            HoverCard::new("nextjs", link).content(|_, cx| {
                let theme = *cx.notra();
                div()
                    .flex()
                    .gap(px(16.0))
                    .child(
                        div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .justify_center()
                            .size(px(40.0))
                            .rounded_full()
                            .bg(theme.muted)
                            .font_weight(FontWeight(600.0))
                            .child("N"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .min_w_0()
                            .child(popover_title("@nextjs", cx))
                            .child("The React Framework – created and maintained by @vercel.")
                            .child(popover_description("Joined December 2021", cx).pt(px(4.0))),
                    )
                    .into_any_element()
            }),
        ))
    }
}
