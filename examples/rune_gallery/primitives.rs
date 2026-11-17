//! The primitive layer's gallery pages.

use notra_ui::Icon;
use notra_ui::Theme;
use notra_ui::rune::{
    Avatar, AvatarShape, AvatarSize, Badge, BadgeShape, BadgeSize, BadgeVariant, CheckState,
    Checkbox, Collapsible, CollapsibleChevron, Heading, HeadingSize, Kbd, KbdGroup,
    MarketingButton, MarketingSize, MarketingVariant, Separator, Shimmer, Skeleton, Spinner,
    StatusBadge, StatusVariant, Text, TextSize, TextTone, TextWeight, ToRune, TrendBadge,
    TrendTone, sans,
};
use rune::{El, Pixels, col, el, row, text};

use crate::{State, preview, section};

/// State the primitive pages own.
pub struct Demo {
    pub loading: [bool; 2],
    pub checks: [CheckState; 2],
    pub open: bool,
    pub avatar: Pixels,
}

impl Default for Demo {
    fn default() -> Self {
        let avatar = Pixels::new();
        let size = 160u32;
        let rgba = (0..size * size)
            .flat_map(|i| {
                let (x, y) = ((i % size) as f32, (i / size) as f32);
                let d = ((x - 80.0).hypot(y - 80.0) / 113.0).min(1.0);
                let mix = |a: f32, b: f32| (a + (b - a) * d) as u8;
                [mix(200.0, 126.0), mix(178.0, 76.0), mix(238.0, 231.0), 255]
            })
            .collect();
        avatar.set(size, size, rgba);
        Self {
            loading: [false; 2],
            checks: [CheckState::Unchecked, CheckState::Checked],
            open: false,
            avatar,
        }
    }
}

impl Demo {
    /// Every stateful part in its active state.
    pub fn active() -> Self {
        Self {
            loading: [true; 2],
            checks: [CheckState::Checked, CheckState::Indeterminate],
            open: true,
            ..Self::default()
        }
    }
}

fn row_of(t: &Theme) -> El {
    preview(t).gap(24.0)
}

pub fn marketing_button(s: &State, t: &Theme) -> El {
    let demo = &s.primitives;
    let toggle = |i: usize| move |s: &mut State| s.primitives.loading[i] = !s.primitives.loading[i];
    let sizes = [
        ("Small", MarketingSize::Sm),
        ("Default", MarketingSize::Default),
        ("Large", MarketingSize::Lg),
    ]
    .iter()
    .fold(row_of(t), |p, &(label, size)| {
        p.child(
            MarketingButton::new(label)
                .size(size)
                .build(t)
                .key(format!("size-{label}")),
        )
    });
    col()
        .child(section(
            "Demo",
            t,
            row_of(t)
                .child(
                    MarketingButton::new("Start for free")
                        .icon_end(Icon::ArrowRight)
                        .build(t)
                        .key("start"),
                )
                .child(
                    MarketingButton::new("Book a call")
                        .variant(MarketingVariant::Light)
                        .build(t)
                        .key("call"),
                ),
        ))
        .child(section(
            "Variants",
            t,
            row_of(t)
                .child(MarketingButton::new("Primary").build(t).key("primary"))
                .child(
                    MarketingButton::new("Flat")
                        .variant(MarketingVariant::Flat)
                        .build(t)
                        .key("flat"),
                )
                .child(
                    MarketingButton::new("Light")
                        .variant(MarketingVariant::Light)
                        .build(t)
                        .key("light"),
                )
                .child(
                    MarketingButton::new("Disabled")
                        .disabled(true)
                        .build(t)
                        .key("disabled"),
                ),
        ))
        .child(section("Sizes", t, sizes))
        .child(section(
            "Loading",
            t,
            row_of(t)
                .child(
                    MarketingButton::new("Start for free")
                        .icon_end(Icon::ArrowRight)
                        .loading(demo.loading[0])
                        .on_press(toggle(0))
                        .build(t)
                        .key("load-start"),
                )
                .child(
                    MarketingButton::new("Book a call")
                        .variant(MarketingVariant::Light)
                        .loading(demo.loading[1])
                        .on_press(toggle(1))
                        .build(t)
                        .key("load-call"),
                ),
        ))
}

pub fn typography(_s: &State, t: &Theme) -> El {
    let column = || preview(t).flex_wrap().gap(16.0);
    let headings = HeadingSize::ALL.iter().fold(col().gap(16.0), |c, &size| {
        c.child(
            Heading::new(format!("{size:?}: Changelogs your users read"))
                .size(size)
                .build(t)
                .key(format!("heading-{size:?}")),
        )
    });
    let texts = TextSize::ALL.iter().fold(col().gap(16.0), |c, &size| {
        c.child(
            Text::new(format!(
                "{size:?}: Notra turns merged pull requests into release notes."
            ))
            .size(size)
            .build(t)
            .key(format!("text-{size:?}")),
        )
    });
    col()
        .child(section("Heading", t, column().child(headings)))
        .child(section("Text sizes", t, column().child(texts)))
        .child(section(
            "Tone and weight",
            t,
            column().child(
                col()
                    .gap(16.0)
                    .child(Text::new("Default tone, regular").build(t))
                    .child(Text::new("Muted tone").tone(TextTone::Muted).build(t))
                    .child(
                        Text::new("Medium weight")
                            .weight(TextWeight::Medium)
                            .build(t),
                    ),
            ),
        ))
}

pub fn shimmer(_s: &State, t: &Theme) -> El {
    col().child(section(
        "Shimmer",
        t,
        preview(t).child(
            col()
                .gap(12.0)
                .padding(24.0)
                .child(Shimmer::new("Thinking...").build(t).key("thinking"))
                .child(
                    Shimmer::new("Searching the web")
                        .duration(std::time::Duration::from_millis(2400))
                        .build(t)
                        .key("searching"),
                ),
        ),
    ))
}

fn caption(label: &str, t: &Theme) -> El {
    text(label)
        .font(sans(14.0, 400.0, 20.0))
        .color(t.muted_foreground.rune())
}

pub fn badges(_s: &State, t: &Theme) -> El {
    let production = || {
        StatusBadge::new("Production")
            .icon(Icon::HugeRocket01)
            .build(t)
    };
    let cell = |content: El| row().grow(1.0).h_full().px(16.0).center_y().child(content);
    let statuses = [
        row()
            .gap(8.0)
            .center_y()
            .child(
                StatusBadge::new("Successful")
                    .variant(StatusVariant::Success)
                    .icon(Icon::HugeTick02)
                    .square(true)
                    .build(t),
            )
            .child(caption("11s", t)),
        StatusBadge::new("Superseded").build(t),
        StatusBadge::new("Skipped").build(t),
    ];
    let count = statuses.len();
    let body = statuses
        .into_iter()
        .enumerate()
        .fold(col(), |body, (i, status)| {
            let r = row()
                .key(format!("row-{i}"))
                .h(48.0)
                .child(cell(production()))
                .child(cell(status));
            body.child(if i + 1 < count {
                r.border_sides([0.0, 0.0, 1.0, 0.0], t.table_row_border.rune())
            } else {
                r
            })
        });
    let head = |label: &str| {
        row().grow(1.0).px(16.0).center_y().child(
            text(label)
                .font(sans(14.0, 500.0, 20.0))
                .color(t.muted_foreground.rune()),
        )
    };
    let table = col()
        .w(624.0)
        .padding(2.0)
        .radius(16.0)
        .border(1.0, t.shell_border.rune())
        .fill(t.shell.rune())
        .child(
            row()
                .h(36.0)
                .child(head("Environment"))
                .child(head("Status")),
        )
        .child(
            body.radius(14.0)
                .clip_rounded(14.0)
                .border(1.0, t.table_card_border.rune())
                .fill(t.table_card.rune()),
        );
    let wrap = |gap: f32| row().flex_wrap().center_y().gap(gap);
    let group =
        |label: &str, rows: Vec<El>| col().gap(12.0).child(caption(label, t)).children(rows);
    col().child(section(
        "Showcase",
        t,
        preview(t).child(
            col()
                .gap(24.0)
                .child(table)
                .child(group(
                    "Status variants",
                    vec![
                        wrap(8.0)
                            .child(
                                StatusBadge::new("Queued")
                                    .variant(StatusVariant::Info)
                                    .icon(Icon::HugeClock01)
                                    .build(t),
                            )
                            .child(
                                StatusBadge::new("Building")
                                    .variant(StatusVariant::Warning)
                                    .icon(Icon::HugeLoading03)
                                    .spin(true)
                                    .build(t),
                            )
                            .child(
                                StatusBadge::new("Failed")
                                    .variant(StatusVariant::Destructive)
                                    .icon(Icon::HugeCancel01)
                                    .square(true)
                                    .build(t),
                            )
                            .child(StatusBadge::new("Canceled").build(t)),
                    ],
                ))
                .child(group(
                    "Trends · visibility, mentions, and readiness",
                    vec![
                        wrap(8.0)
                            .child(TrendBadge::new(TrendTone::Up, "+12.4%").build(t))
                            .child(TrendBadge::new(TrendTone::Down, "−3.2%").build(t))
                            .child(TrendBadge::new(TrendTone::Flat, "0%").build(t))
                            .child(
                                TrendBadge::new(TrendTone::Up, "New")
                                    .show_icon(false)
                                    .build(t),
                            ),
                        wrap(8.0)
                            .child(
                                TrendBadge::new(TrendTone::Up, "+8 pts")
                                    .size(BadgeSize::Sm)
                                    .build(t),
                            )
                            .child(TrendBadge::new(TrendTone::Up, "+8 pts").build(t))
                            .child(
                                TrendBadge::new(TrendTone::Up, "+8 pts")
                                    .size(BadgeSize::Md)
                                    .build(t),
                            )
                            .child(
                                StatusBadge::new("Learning")
                                    .size(BadgeSize::Default)
                                    .build(t),
                            ),
                        wrap(12.0)
                            .child(TrendBadge::new(TrendTone::Up, "12.4%").plain().build(t))
                            .child(TrendBadge::new(TrendTone::Down, "3.2%").plain().build(t))
                            .child(TrendBadge::new(TrendTone::Flat, "0%").plain().build(t)),
                        wrap(8.0)
                            .child(
                                StatusBadge::new("Merged")
                                    .variant(StatusVariant::Merged)
                                    .shape(BadgeShape::Pill)
                                    .build(t),
                            )
                            .child(
                                TrendBadge::new(TrendTone::Down, "−1")
                                    .shape(BadgeShape::Pill)
                                    .build(t),
                            ),
                    ],
                )),
        ),
    ))
}

pub fn primitives(s: &State, t: &Theme) -> El {
    let demo = &s.primitives;
    let badges = BadgeVariant::ALL.iter().fold(preview(t), |p, &variant| {
        p.child(
            Badge::new(format!("{variant:?}"))
                .variant(variant)
                .build(t)
                .key(format!("badge-{variant:?}")),
        )
    });
    let checkbox = |i: usize| {
        Checkbox::new(demo.checks[i])
            .label(format!("Option {}", i + 1))
            .on_change(move |s: &mut State, next| s.primitives.checks[i] = next)
            .build(t)
            .key(format!("check-{i}"))
    };
    let sources = ["usenotra.com", "github.com/usenotra", "docs.usenotra.com"]
        .iter()
        .fold(col().gap(6.0).pt(10.0), |c, src| {
            c.child(
                text(*src)
                    .font(sans(14.0, 400.0, 20.0))
                    .color(t.muted_foreground.rune()),
            )
        });
    let body = |label: &str| {
        text(label)
            .font(sans(14.0, 400.0, 20.0))
            .color(t.foreground.rune())
    };
    col()
        .child(section(
            "Badge",
            t,
            badges
                .child(
                    Badge::new("Verified")
                        .variant(BadgeVariant::Secondary)
                        .icon(Icon::Check)
                        .build(t)
                        .key("badge-verified"),
                )
                .child(
                    Badge::new("Link")
                        .variant(BadgeVariant::Outline)
                        .on_press(|_: &mut State| {})
                        .build(t)
                        .key("badge-link"),
                ),
        ))
        .child(section(
            "Kbd",
            t,
            preview(t)
                .child(KbdGroup::build([
                    Kbd::new("⌘").build(t),
                    Kbd::new("K").build(t),
                ]))
                .child(Kbd::new("Esc").build(t))
                .child(Kbd::icon(Icon::ArrowUp).build(t))
                .child(
                    row()
                        .gap(6.0)
                        .px(8.0)
                        .py(4.0)
                        .center_y()
                        .radius(6.0)
                        .fill(t.foreground.rune())
                        .child(
                            text("Search")
                                .font(sans(12.0, 400.0, 16.0))
                                .color(t.background.rune()),
                        )
                        .child(Kbd::new("/").in_tooltip().build(t)),
                ),
        ))
        .child(section(
            "Spinner",
            t,
            preview(t)
                .child(Spinner::new().size(12.0).build(t).key("spin-12"))
                .child(Spinner::new().size(14.0).build(t).key("spin-14"))
                .child(Spinner::new().build(t).key("spin-16"))
                .child(
                    row()
                        .gap(8.0)
                        .center_y()
                        .child(
                            Spinner::new()
                                .color(t.muted_foreground)
                                .build(t)
                                .key("spin-label"),
                        )
                        .child(caption("Uploading", t)),
                ),
        ))
        .child(section(
            "Skeleton",
            t,
            preview(t).child(
                row()
                    .gap(16.0)
                    .center_y()
                    .child(
                        Skeleton::build(t)
                            .size(40.0, 40.0)
                            .radius(20.0)
                            .key("sk-avatar"),
                    )
                    .child(
                        col()
                            .gap(8.0)
                            .child(Skeleton::build(t).w(250.0).h(16.0).key("sk-1"))
                            .child(Skeleton::build(t).w(200.0).h(16.0).key("sk-2")),
                    ),
            ),
        ))
        .child(section(
            "Avatar",
            t,
            preview(t)
                .child(Avatar::new("LV").size(AvatarSize::Sm).build(t))
                .child(Avatar::new("LV").build(t))
                .child(Avatar::new("LV").size(AvatarSize::Lg).build(t))
                .child(Avatar::new("NO").image(&demo.avatar).build(t))
                .child(
                    Avatar::new("NO")
                        .image(&demo.avatar)
                        .size(AvatarSize::Lg)
                        .shape(AvatarShape::Rounded(8.0))
                        .build(t),
                ),
        ))
        .child(section(
            "Separator",
            t,
            preview(t).child(
                col()
                    .w(280.0)
                    .gap(16.0)
                    .child(
                        col()
                            .gap(4.0)
                            .child(
                                text("Notra UI")
                                    .font(sans(14.0, 500.0, 20.0))
                                    .color(t.foreground.rune()),
                            )
                            .child(caption("Components for changelog products.", t)),
                    )
                    .child(Separator::horizontal().build(t))
                    .child(
                        row()
                            .h(20.0)
                            .gap(16.0)
                            .center_y()
                            .child(body("Blog"))
                            .child(Separator::vertical().build(t))
                            .child(body("Docs"))
                            .child(Separator::vertical().build(t))
                            .child(body("Source")),
                    ),
            ),
        ))
        .child(section(
            "Collapsible",
            t,
            preview(t).min_h(200.0).child(
                el().w(280.0).child(
                    Collapsible::new(demo.open)
                        .trigger(
                            row()
                                .gap(6.0)
                                .center_y()
                                .child(
                                    text("Reviewed 3 sources")
                                        .font(sans(14.0, 500.0, 20.0))
                                        .color(t.foreground.rune()),
                                )
                                .child(CollapsibleChevron::new(demo.open).build(t.foreground)),
                        )
                        .content(sources)
                        .on_toggle(|s: &mut State| s.primitives.open = !s.primitives.open)
                        .build()
                        .key("sources"),
                ),
            ),
        ))
        .child(section(
            "Checkbox",
            t,
            row_of(t)
                .child(checkbox(0))
                .child(checkbox(1))
                .child(Checkbox::new(CheckState::Checked).build(t))
                .child(Checkbox::new(CheckState::Indeterminate).build(t))
                .child(Checkbox::new(CheckState::Unchecked).disabled(true).build(t))
                .child(Checkbox::new(CheckState::Checked).disabled(true).build(t)),
        ))
}
