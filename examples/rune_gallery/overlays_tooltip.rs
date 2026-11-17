//! Tooltip, Duotone Tooltip, Popover and Hover Card pages.

use std::time::{Duration, Instant};

use notra_ui::Icon;
use notra_ui::Theme;
use notra_ui::components::overlays::Side;
use notra_ui::rune::overlays::popover::{
    HoverCard, HoverCardDelays, HoverIntent, Popover, hover_card_trigger, popover_description,
    popover_header, popover_title,
};
use notra_ui::rune::overlays::tooltip::{
    DuotoneTooltip, Tooltip, TooltipDelays, TooltipState, duotone_trigger, trigger,
};
use notra_ui::rune::{ToRune, sans};
use rune::{El, col, el, row, text};

use crate::overlays_demo::{ghost_icon_button, outline_button};
use crate::{State, preview, section};

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

/// State the pages own.
pub struct Demo {
    pub sides: TooltipState,
    pub toolbar: TooltipState,
    pub duotone: HoverIntent,
    pub popover: bool,
    pub hover_card: HoverIntent,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            sides: TooltipState::new(TooltipDelays::PROVIDER),
            toolbar: TooltipState::new(
                TooltipDelays::PROVIDER.with_open(Duration::from_millis(400)),
            ),
            duotone: HoverIntent::new(HoverCardDelays::NONE),
            popover: false,
            hover_card: HoverIntent::new(HoverCardDelays::NONE),
        }
    }
}

impl Demo {
    /// The sides example with its top tooltip open.
    pub fn side_open() -> Self {
        let mut d = Self::default();
        d.sides.show("top");
        d
    }

    /// The toolbar with the Bold tooltip open.
    pub fn toolbar_open() -> Self {
        let mut d = Self::default();
        d.toolbar.show("Bold");
        d
    }

    pub fn duotone_open() -> Self {
        let mut d = Self::default();
        d.duotone.set_trigger(true, Instant::now());
        d
    }

    pub fn popover_open() -> Self {
        Self {
            popover: true,
            ..Self::default()
        }
    }

    pub fn hover_card_open() -> Self {
        let mut d = Self::default();
        d.hover_card.set_trigger(true, Instant::now());
        d
    }
}

pub fn tooltip(s: &State, t: &Theme) -> El {
    let d = &s.overlays_tooltip;
    let sides = SIDES.iter().map(|(_, name)| {
        let label = format!("{}{}", name[..1].to_uppercase(), &name[1..]);
        trigger(
            outline_button(&label, false, t),
            name,
            &format!("Opens on the {name}"),
            &d.sides,
            |s: &mut State| &mut s.overlays_tooltip.sides,
        )
    });
    let buttons = TOOLBAR.iter().map(|(icon, label, _)| {
        trigger(
            ghost_icon_button(*icon, label, t),
            label,
            label,
            &d.toolbar,
            |s: &mut State| &mut s.overlays_tooltip.toolbar,
        )
    });
    let toolbar = row()
        .role("toolbar")
        .label("Formatting")
        .gap(2.0)
        .padding(4.0)
        .center_y()
        .radius(12.0)
        .border(1.0, t.border.rune())
        .fill(t.background.rune())
        .box_shadow(notra_ui::Shadow::drop(1.0, 2.0, 0.0, notra_ui::Color::rgba(0x0000000d)).rune())
        .children(buttons);

    let mut page = col()
        .child(section(
            "Sides",
            t,
            preview(t).gap(12.0).padding(48.0).children(sides),
        ))
        .child(section(
            "Toolbar",
            t,
            preview(t).padding(40.0).child(toolbar),
        ));
    if let Some(key) = d.sides.open_key()
        && let Some((side, name)) = SIDES.iter().find(|(_, n)| *n == key)
    {
        page = page.child(
            Tooltip::new(format!("Opens on the {name}"))
                .side(*side)
                .build(t, "sides-tooltip", &d.sides),
        );
    }
    if let Some(key) = d.toolbar.open_key()
        && let Some((_, label, keys)) = TOOLBAR.iter().find(|(_, l, _)| *l == key)
    {
        page = page.child(
            Tooltip::new(*label)
                .keys(keys)
                .build(t, "toolbar-tooltip", &d.toolbar),
        );
    }
    page
}

pub fn duotone(s: &State, t: &Theme) -> El {
    let d = &s.overlays_tooltip;
    let trigger = duotone_trigger("Hover me", t)
        .key("duotone-trigger")
        .description("Weekly summary")
        .on_hover_change(|s: &mut State, inside, _| {
            s.overlays_tooltip
                .duotone
                .set_trigger(inside, Instant::now());
        })
        .on_key(|s: &mut State, ev, _| {
            ev.key == rune::Key::Escape && s.overlays_tooltip.duotone.close()
        });
    let mut page = col().child(section(
        "Preview",
        t,
        preview(t)
            .fill(t.muted.fade(0.4).rune())
            .radius(16.0)
            .pt(40.0)
            .pb(176.0)
            .center_y()
            .child(trigger),
    ));
    if d.duotone.is_open() {
        page = page.child(
            DuotoneTooltip::new(
                "Weekly summary",
                "Visits grew 18% compared to the previous week, led by returning readers.",
            )
            .aside("Last 7 days")
            .build(t, "duotone-card", "duotone-trigger", |s: &mut State| {
                &mut s.overlays_tooltip.duotone
            }),
        );
    }
    page
}

pub fn popover(s: &State, t: &Theme) -> El {
    let open = s.overlays_tooltip.popover;
    let trigger = outline_button("Open popover", open, t)
        .key("popover-trigger")
        .on_press(|s: &mut State| s.overlays_tooltip.popover = !s.overlays_tooltip.popover);
    let mut page = col().child(section(
        "Preview",
        t,
        preview(t).min_h(320.0).child(trigger),
    ));
    if open {
        let row_of = |label: &str, value: &str| {
            row()
                .center_y()
                .child(
                    text(label)
                        .font(sans(14.0, 400.0, 20.0))
                        .color(t.muted_foreground.rune()),
                )
                .child(el().grow(1.0))
                .child(
                    text(value)
                        .font(sans(14.0, 500.0, 20.0))
                        .color(t.foreground.rune()),
                )
        };
        page = page.child(
            Popover::new()
                .child(popover_header(
                    "Dimensions",
                    "Set the dimensions for the layer.",
                    t,
                ))
                .child(row_of("Width", "100%"))
                .child(row_of("Height", "25px"))
                .build(t, "popover", "popover-trigger", |s: &mut State| {
                    s.overlays_tooltip.popover = false
                }),
        );
    }
    page
}

pub fn hover_card(s: &State, t: &Theme) -> El {
    let link = text("@nextjs")
        .font(sans(14.0, 500.0, 20.0))
        .color(t.foreground.rune())
        .role("link")
        .label("@nextjs")
        .focusable(true)
        .cursor(rune::Cursor::Pointer);
    let trigger = hover_card_trigger(
        link,
        "hover-card-trigger",
        HoverCardDelays::BASE_UI,
        |s: &mut State| &mut s.overlays_tooltip.hover_card,
    );
    let mut page = col().child(section(
        "Preview",
        t,
        preview(t).min_h(280.0).child(trigger),
    ));
    if s.overlays_tooltip.hover_card.is_open() {
        let avatar = row()
            .size(40.0, 40.0)
            .radius(20.0)
            .center_y()
            .justify_center()
            .fill(t.muted.rune())
            .child(
                text("N")
                    .font(sans(14.0, 600.0, 20.0))
                    .color(t.foreground.rune()),
            );
        let body = col()
            .gap(4.0)
            .shrink()
            .child(popover_title("@nextjs", t))
            .child(
                text("The React Framework – created and maintained by @vercel.")
                    .wrap()
                    .font(sans(14.0, 400.0, 20.0))
                    .color(t.popover_foreground.rune()),
            )
            .child(popover_description("Joined December 2021", t).pt(4.0));
        page = page.child(
            HoverCard::new()
                .child(row().gap(16.0).child(avatar).child(body))
                .build(t, "hover-card", "hover-card-trigger", |s: &mut State| {
                    &mut s.overlays_tooltip.hover_card
                }),
        );
    }
    page
}
