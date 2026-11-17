//! The primitive layer's gallery pages: marketing button, typography, shimmer, status
//! and trend badges, and the shadcn primitives.

use std::path::PathBuf;
use std::time::Duration;

use gpui::{
    App, AppContext as _, Context, Div, FontWeight, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::*, px,
};
use notra_ui::Icon;
use notra_ui::gpui::{
    ActiveTheme, Avatar, AvatarShape, AvatarSize, Badge, BadgeShape, BadgeSize, BadgeVariant,
    CheckState, Checkbox, Collapsible, CollapsibleChevron, Heading, HeadingSize, Kbd, KbdGroup,
    MarketingButton, MarketingSize, MarketingVariant, Separator, Shimmer, Skeleton, Spinner,
    StatusBadge, StatusVariant, Text, TextSize, TextTone, TextWeight, TrendBadge, TrendTone,
};

use crate::{Page, preview, section};

pub fn marketing_button_page(cx: &mut App) -> Page {
    Page {
        slug: "marketing-button",
        title: "Marketing Button",
        view: cx.new(|_| MarketingPage::default()).into(),
    }
}

pub fn typography_page(cx: &mut App) -> Page {
    Page {
        slug: "typography",
        title: "Typography",
        view: cx.new(|_| TypographyPage).into(),
    }
}

pub fn shimmer_page(cx: &mut App) -> Page {
    Page {
        slug: "shimmer",
        title: "Shimmer",
        view: cx.new(|_| ShimmerPage).into(),
    }
}

pub fn badges_page(cx: &mut App) -> Page {
    Page {
        slug: "badges",
        title: "Status & Trend Badges",
        view: cx.new(|_| BadgesPage).into(),
    }
}

pub fn primitives_page(cx: &mut App) -> Page {
    Page {
        slug: "primitives",
        title: "Primitives",
        view: cx.new(|_| PrimitivesPage::default()).into(),
    }
}

/// A demo avatar image shipped with the gallery.
pub fn avatar_image() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/gpui_gallery/primitives-avatar.png")
}

#[derive(Default)]
struct MarketingPage {
    loading: [bool; 2],
}

impl Render for MarketingPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let row = |cx: &App| preview(cx).gap(px(24.0));
        let loading = |index: usize, button: MarketingButton, cx: &mut Context<Self>| {
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
        let sizes = [
            ("Small", MarketingSize::Sm),
            ("Default", MarketingSize::Default),
            ("Large", MarketingSize::Lg),
        ]
        .map(|(label, size)| MarketingButton::new(label, label).size(size));
        div()
            .flex()
            .flex_col()
            .child(
                section("Demo", cx).child(
                    row(cx)
                        .child(
                            MarketingButton::new("start", "Start for free")
                                .icon_end(Icon::ArrowRight),
                        )
                        .child(
                            MarketingButton::new("call", "Book a call")
                                .variant(MarketingVariant::Light),
                        ),
                ),
            )
            .child(
                section("Variants", cx).child(
                    row(cx)
                        .child(MarketingButton::new("primary", "Primary"))
                        .child(MarketingButton::new("flat", "Flat").variant(MarketingVariant::Flat))
                        .child(
                            MarketingButton::new("light", "Light").variant(MarketingVariant::Light),
                        )
                        .child(MarketingButton::new("disabled", "Disabled").disabled(true)),
                ),
            )
            .child(section("Sizes", cx).child(row(cx).children(sizes)))
            .child(
                section("Loading", cx).child(
                    row(cx)
                        .child(loading(
                            0,
                            MarketingButton::new("load-start", "Start for free")
                                .icon_end(Icon::ArrowRight),
                            cx,
                        ))
                        .child(loading(
                            1,
                            MarketingButton::new("load-call", "Book a call")
                                .variant(MarketingVariant::Light),
                            cx,
                        )),
                ),
            )
    }
}

struct TypographyPage;

impl Render for TypographyPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let headings = HeadingSize::ALL.map(|size| {
            Heading::new(format!("{size:?}: Changelogs your users read"))
                .size(size)
                .wide(true)
        });
        let texts = TextSize::ALL.map(|size| {
            Text::new(format!(
                "{size:?}: Notra turns merged pull requests into release notes."
            ))
            .size(size)
        });
        let column = |cx: &App| {
            preview(cx)
                .flex_col()
                .items_start()
                .justify_start()
                .gap(px(16.0))
        };
        div()
            .flex()
            .flex_col()
            .child(section("Heading", cx).child(column(cx).children(headings)))
            .child(section("Text sizes", cx).child(column(cx).children(texts)))
            .child(
                section("Tone and weight", cx).child(
                    column(cx)
                        .child(Text::new("Default tone, regular").tone(TextTone::Default))
                        .child(Text::new("Muted tone").tone(TextTone::Muted))
                        .child(
                            Text::new("Medium weight")
                                .tone(TextTone::Default)
                                .weight(TextWeight::Medium),
                        ),
                ),
            )
    }
}

struct ShimmerPage;

impl Render for ShimmerPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.notra();
        div().flex().flex_col().child(
            section("Shimmer", cx).child(
                preview(cx).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .p(px(24.0))
                        .text_size(px(14.0))
                        .line_height(px(20.0))
                        .text_color(theme.muted_foreground)
                        .child(Shimmer::new("thinking", "Thinking..."))
                        .child(
                            Shimmer::new("searching", "Searching the web")
                                .duration(Duration::from_millis(2400)),
                        ),
                ),
            ),
        )
    }
}

struct BadgesPage;

fn caption(text: &'static str, cx: &App) -> Div {
    div()
        .text_size(px(14.0))
        .line_height(px(20.0))
        .text_color(cx.notra().muted_foreground)
        .child(text)
}

impl Render for BadgesPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.notra();
        let production = || StatusBadge::new("Production").icon(Icon::HugeRocket01);
        let cell = |content: gpui::AnyElement| {
            div()
                .flex()
                .flex_1()
                .items_center()
                .h_full()
                .px(px(16.0))
                .child(content)
        };
        let rows = [
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    StatusBadge::new("Successful")
                        .variant(StatusVariant::Success)
                        .icon(Icon::HugeTick02)
                        .square(true),
                )
                .child(div().text_color(theme.muted_foreground).child("11s"))
                .into_any_element(),
            StatusBadge::new("Superseded").into_any_element(),
            StatusBadge::new("Skipped").into_any_element(),
        ];
        let count = rows.len();
        let body = rows.into_iter().enumerate().map(|(i, status)| {
            div()
                .flex()
                .h(px(48.0))
                .when(i + 1 < count, |d| {
                    d.border_b_1().border_color(theme.table_row_border)
                })
                .child(cell(production().into_any_element()))
                .child(cell(status))
        });
        let head = |label: &'static str| {
            div()
                .flex()
                .flex_1()
                .items_center()
                .px(px(16.0))
                .text_color(theme.muted_foreground)
                .font_weight(FontWeight(500.0))
                .child(label)
        };
        let table = div()
            .w(px(624.0))
            .p(px(2.0))
            .rounded(px(16.0))
            .border_1()
            .border_color(theme.shell_border)
            .bg(theme.shell)
            .text_size(px(14.0))
            .line_height(px(20.0))
            .child(
                div()
                    .flex()
                    .h(px(36.0))
                    .child(head("Environment"))
                    .child(head("Status")),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .rounded(px(14.0))
                    .border_1()
                    .border_color(theme.table_card_border)
                    .bg(theme.table_card)
                    .overflow_hidden()
                    .children(body),
            );
        let wrap = |gap: f32| div().flex().flex_wrap().items_center().gap(px(gap));
        div().flex().flex_col().child(
            section("Showcase", cx).child(
                preview(cx).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(24.0))
                        .child(table)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(12.0))
                                .child(caption("Status variants", cx))
                                .child(
                                    wrap(8.0)
                                        .child(
                                            StatusBadge::new("Queued")
                                                .variant(StatusVariant::Info)
                                                .icon(Icon::HugeClock01),
                                        )
                                        .child(
                                            StatusBadge::new("Building")
                                                .variant(StatusVariant::Warning)
                                                .icon(Icon::HugeLoading03)
                                                .spin(true),
                                        )
                                        .child(
                                            StatusBadge::new("Failed")
                                                .variant(StatusVariant::Destructive)
                                                .icon(Icon::HugeCancel01)
                                                .square(true),
                                        )
                                        .child(StatusBadge::new("Canceled")),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(12.0))
                                .child(caption("Trends · visibility, mentions, and readiness", cx))
                                .child(
                                    wrap(8.0)
                                        .child(TrendBadge::new(TrendTone::Up, "+12.4%"))
                                        .child(TrendBadge::new(TrendTone::Down, "−3.2%"))
                                        .child(TrendBadge::new(TrendTone::Flat, "0%"))
                                        .child(
                                            TrendBadge::new(TrendTone::Up, "New").show_icon(false),
                                        ),
                                )
                                .child(
                                    wrap(8.0)
                                        .child(
                                            TrendBadge::new(TrendTone::Up, "+8 pts")
                                                .size(BadgeSize::Sm),
                                        )
                                        .child(TrendBadge::new(TrendTone::Up, "+8 pts"))
                                        .child(
                                            TrendBadge::new(TrendTone::Up, "+8 pts")
                                                .size(BadgeSize::Md),
                                        )
                                        .child(
                                            StatusBadge::new("Learning").size(BadgeSize::Default),
                                        ),
                                )
                                .child(
                                    wrap(12.0)
                                        .child(TrendBadge::new(TrendTone::Up, "12.4%").plain())
                                        .child(TrendBadge::new(TrendTone::Down, "3.2%").plain())
                                        .child(TrendBadge::new(TrendTone::Flat, "0%").plain()),
                                )
                                .child(
                                    wrap(8.0)
                                        .child(
                                            StatusBadge::new("Merged")
                                                .variant(StatusVariant::Merged)
                                                .shape(BadgeShape::Pill),
                                        )
                                        .child(
                                            TrendBadge::new(TrendTone::Down, "−1")
                                                .shape(BadgeShape::Pill),
                                        ),
                                ),
                        ),
                ),
            ),
        )
    }
}

#[derive(Default)]
struct PrimitivesPage {
    checks: [CheckState; 2],
    open: bool,
}

impl Render for PrimitivesPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.notra();
        let badges =
            BadgeVariant::ALL.map(|variant| Badge::new(format!("{variant:?}")).variant(variant));
        let checkbox = |index: usize, cx: &mut Context<Self>| {
            Checkbox::new(("check", index), self.checks[index]).on_change(cx.listener(
                move |this, next: &CheckState, _, cx| {
                    this.checks[index] = *next;
                    cx.notify();
                },
            ))
        };
        let sources = ["usenotra.com", "github.com/usenotra", "docs.usenotra.com"]
            .map(|s| div().text_color(theme.muted_foreground).child(s));
        let open = self.open;
        div()
            .flex()
            .flex_col()
            .child(
                section("Badge", cx).child(
                    preview(cx)
                        .children(badges)
                        .child(
                            Badge::new("Verified")
                                .variant(BadgeVariant::Secondary)
                                .icon(Icon::Check),
                        )
                        .child(
                            Badge::new("Link")
                                .variant(BadgeVariant::Outline)
                                .on_click("badge-link", |_, _, _| {}),
                        ),
                ),
            )
            .child(
                section("Kbd", cx).child(
                    preview(cx)
                        .child(KbdGroup::new().child(Kbd::new("⌘")).child(Kbd::new("K")))
                        .child(Kbd::new("Esc"))
                        .child(Kbd::icon(Icon::ArrowUp))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .px(px(8.0))
                                .py(px(4.0))
                                .rounded(px(6.0))
                                .bg(theme.foreground)
                                .text_color(theme.background)
                                .text_size(px(12.0))
                                .child("Search")
                                .child(Kbd::new("/").in_tooltip()),
                        ),
                ),
            )
            .child(
                section("Spinner", cx).child(
                    preview(cx)
                        .child(Spinner::new("spin-12").size(12.0))
                        .child(Spinner::new("spin-14").size(14.0))
                        .child(Spinner::new("spin-16"))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .text_size(px(14.0))
                                .text_color(theme.muted_foreground)
                                .child(Spinner::new("spin-label"))
                                .child("Uploading"),
                        ),
                ),
            )
            .child(
                section("Skeleton", cx).child(
                    preview(cx).child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(16.0))
                            .child(Skeleton::new("sk-avatar").size(px(40.0)).rounded_full())
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.0))
                                    .child(Skeleton::new("sk-1").h(px(16.0)).w(px(250.0)))
                                    .child(Skeleton::new("sk-2").h(px(16.0)).w(px(200.0))),
                            ),
                    ),
                ),
            )
            .child(
                section("Avatar", cx).child(
                    preview(cx)
                        .child(Avatar::new("LV").size(AvatarSize::Sm))
                        .child(Avatar::new("LV"))
                        .child(Avatar::new("LV").size(AvatarSize::Lg))
                        .child(Avatar::new("NO").image(avatar_image()))
                        .child(
                            Avatar::new("NO")
                                .image(avatar_image())
                                .size(AvatarSize::Lg)
                                .shape(AvatarShape::Rounded(8.0)),
                        ),
                ),
            )
            .child(
                section("Separator", cx).child(
                    preview(cx).child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(280.0))
                            .gap(px(16.0))
                            .text_size(px(14.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(div().font_weight(FontWeight(500.0)).child("Notra UI"))
                                    .child(
                                        div()
                                            .text_color(theme.muted_foreground)
                                            .child("Components for changelog products."),
                                    ),
                            )
                            .child(Separator::horizontal())
                            .child(
                                div()
                                    .flex()
                                    .h(px(20.0))
                                    .items_center()
                                    .gap(px(16.0))
                                    .child("Blog")
                                    .child(Separator::vertical())
                                    .child("Docs")
                                    .child(Separator::vertical())
                                    .child("Source"),
                            ),
                    ),
                ),
            )
            .child(
                section("Collapsible", cx).child(
                    preview(cx).items_start().min_h(px(200.0)).child(
                        div()
                            .w(px(280.0))
                            .text_size(px(14.0))
                            .line_height(px(20.0))
                            .child(
                                Collapsible::new("sources", open)
                                    .trigger(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(6.0))
                                            .font_weight(FontWeight(500.0))
                                            .child("Reviewed 3 sources")
                                            .child(CollapsibleChevron::new(
                                                "sources-chevron",
                                                open,
                                            )),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(6.0))
                                            .pt(px(10.0))
                                            .children(sources),
                                    )
                                    .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                                        this.open = *open;
                                        cx.notify();
                                    })),
                            ),
                    ),
                ),
            )
            .child(
                section("Checkbox", cx).child(
                    preview(cx)
                        .gap(px(24.0))
                        .child(checkbox(0, cx))
                        .child(checkbox(1, cx))
                        .child(Checkbox::new("check-on", CheckState::Checked))
                        .child(Checkbox::new("check-mixed", CheckState::Indeterminate))
                        .child(
                            Checkbox::new("check-off-disabled", CheckState::Unchecked)
                                .disabled(true),
                        )
                        .child(
                            Checkbox::new("check-on-disabled", CheckState::Checked).disabled(true),
                        ),
                ),
            )
    }
}
