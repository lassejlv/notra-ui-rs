use notra_ui::Theme;
use notra_ui::components::overlays::menu::demo_entries;
use notra_ui::components::overlays::select::{demo_flat, demo_grouped};
use notra_ui::rune::overlays::{
    DropdownMenu, MenuState, Select, SelectSize, SelectState, SelectVariant,
};
use notra_ui::rune::{ToRune, sans};
use rune::{El, col, text};

use crate::overlays_demo::outline_button;
use crate::{State, preview, section};

/// The dropdown menu and select demos' state.
pub struct Demo {
    pub menu: MenuState,
    pub activity: bool,
    pub view: String,
    pub frequency: String,
    pub frequency_select: SelectState,
    pub compact: String,
    pub compact_select: SelectState,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            menu: MenuState::default(),
            activity: true,
            view: "team".into(),
            frequency: "weekly".into(),
            frequency_select: SelectState::default(),
            compact: "weekly".into(),
            compact_select: SelectState::default(),
        }
    }
}

fn menu_lens(s: &mut State) -> &mut MenuState {
    &mut s.overlays_menu.menu
}

pub fn menu_view(s: &State, t: &Theme) -> El {
    let d = &s.overlays_menu;
    let entries = demo_entries(d.activity, &d.view);
    let menu = DropdownMenu::new("demo-menu", &entries).width(224.0);
    let trigger = menu.trigger(
        outline_button("Open menu", d.menu.open, t),
        &d.menu,
        menu_lens,
    );
    let popups = menu.popups(&d.menu, t, menu_lens, |s: &mut State, item| {
        let d = &mut s.overlays_menu;
        match item.id.as_str() {
            "activity" => d.activity = !d.activity,
            "team" | "personal" => d.view = item.id.clone(),
            _ => {}
        }
    });
    col()
        .child(section(
            "Demo",
            t,
            preview(t).items_start().min_h(448.0).child(trigger),
        ))
        .children(popups)
}

fn frequency_lens(s: &mut State) -> &mut SelectState {
    &mut s.overlays_menu.frequency_select
}

fn compact_lens(s: &mut State) -> &mut SelectState {
    &mut s.overlays_menu.compact_select
}

pub fn select_view(s: &State, t: &Theme) -> El {
    let d = &s.overlays_menu;
    let grouped = demo_grouped();
    let flat = demo_flat();
    let frequency = Select::new("frequency", &grouped)
        .value(Some(&d.frequency))
        .label("Frequency")
        .width(192.0);
    let compact = Select::new("compact", &flat)
        .value(Some(&d.compact))
        .label("Compact property")
        .variant(SelectVariant::Ghost)
        .size(SelectSize::Sm);
    let field = |label: &str, trigger: El| {
        col()
            .gap(8.0)
            .child(
                text(label)
                    .font(sans(14.0, 400.0, 20.0))
                    .color(t.foreground.rune()),
            )
            .child(trigger)
    };
    col()
        .child(section(
            "Demo",
            t,
            preview(t)
                .items_start()
                .gap(32.0)
                .min_h(320.0)
                .child(field(
                    "Frequency",
                    frequency.trigger(&d.frequency_select, t, frequency_lens),
                ))
                .child(field(
                    "Compact property",
                    compact.trigger(&d.compact_select, t, compact_lens),
                )),
        ))
        .children(frequency.popup(
            &d.frequency_select,
            t,
            frequency_lens,
            |s: &mut State, v| s.overlays_menu.frequency = v.into(),
        ))
        .children(
            compact.popup(&d.compact_select, t, compact_lens, |s: &mut State, v| {
                s.overlays_menu.compact = v.into()
            }),
        )
}
