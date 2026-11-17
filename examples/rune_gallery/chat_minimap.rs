use notra_ui::Theme;
use notra_ui::components::chat_minimap::{
    DEMO_START, DEMO_TURNS, LONG_START, LONG_TURNS, TurnWindow, demo_items,
};
use notra_ui::rune::{ChatMinimap, ChatMinimapSide, ChatMinimapState, ToRune};
use rune::{El, col, row};

use crate::{State, section};

/// The three live examples: default, `side="left"` and fifty turns.
pub struct Demo {
    pub windows: [TurnWindow; 3],
    pub minimaps: [ChatMinimapState; 3],
    /// Show only this example (a scene's focus).
    pub only: Option<usize>,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            windows: [
                TurnWindow::new(DEMO_TURNS.len(), DEMO_START),
                TurnWindow::new(DEMO_TURNS.len(), DEMO_START),
                TurnWindow::new(LONG_TURNS, LONG_START),
            ],
            minimaps: [ChatMinimapState::default(); 3],
            only: None,
        }
    }
}

impl Demo {
    /// Example `slot` alone with the pointer resting on turn `index` and its card open.
    pub fn hovering(slot: usize, index: usize) -> Self {
        let mut demo = Self::focused(slot);
        demo.minimaps[slot] = ChatMinimapState {
            hovered: Some(index),
            open: Some(index),
            on_card: false,
        };
        demo
    }

    /// Example `slot` alone.
    pub fn focused(slot: usize) -> Self {
        Self {
            only: Some(slot),
            ..Self::default()
        }
    }
}

fn lens<const SLOT: usize>(s: &mut State) -> &mut ChatMinimapState {
    &mut s.chat_minimap.minimaps[SLOT]
}

fn example(s: &State, t: &Theme, slot: usize, side: ChatMinimapSide) -> El {
    let demo = &s.chat_minimap;
    let window = demo.windows[slot];
    let lens = [lens::<0>, lens::<1>, lens::<2>][slot];
    let minimap = ChatMinimap::new(format!("chat-minimap-{slot}"), &demo.minimaps[slot], lens)
        .side(side)
        .items(demo_items(window))
        .on_select(move |s: &mut State, index| s.chat_minimap.windows[slot].select(index))
        .previous(window.can_previous(), move |s: &mut State| {
            s.chat_minimap.windows[slot].previous()
        })
        .next(window.can_next(), move |s: &mut State| {
            s.chat_minimap.windows[slot].next()
        })
        .build(t);
    let surface = row()
        .w_full()
        .px(24.0)
        .py(40.0)
        .radius(12.0)
        .border(1.0, t.border.rune())
        .fill(t.canvas.rune());
    match side {
        ChatMinimapSide::Left => surface.justify_end(),
        ChatMinimapSide::Right => surface.justify_start(),
    }
    .child(minimap)
}

pub fn view(s: &State, t: &Theme) -> El {
    let examples = [
        ("Default", ChatMinimapSide::Right),
        ("Left", ChatMinimapSide::Left),
        ("Long", ChatMinimapSide::Right),
    ];
    examples
        .iter()
        .enumerate()
        .filter(|(slot, _)| s.chat_minimap.only.is_none_or(|only| only == *slot))
        .fold(col(), |page, (slot, &(title, side))| {
            page.child(section(title, t, example(s, t, slot, side)))
        })
}
