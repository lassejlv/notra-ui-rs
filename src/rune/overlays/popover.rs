//! shadcn's Popover and Hover Card for Rune.
//!
//! The popover opens on a press of its trigger (the app flips a bool), and its surface traps
//! focus while open: Escape and a press outside close it and focus returns to the trigger. The
//! hover card opens after the pointer rests on its trigger and stays open while the pointer is
//! over it; its state is a [`HoverIntent`] with [`HoverCardDelays::NONE`], because Rune runs
//! the delays (`hover_delays`).
//!
//! ```ignore
//! let trigger = outline_button("Open popover", s.open, &theme)
//!     .key("pop-trigger")
//!     .on_press(|s: &mut App| s.open = !s.open);
//! if s.open {
//!     layer.push(Popover::new().child(popover_header("Dimensions", "Set the size.", &theme))
//!         .build(&theme, "pop", "pop-trigger", |s: &mut App| s.open = false));
//! }
//! ```

use std::time::Instant;

use ::rune::{El, col, text};

use super::popup;
use crate::components::overlays::Placement;
use crate::components::overlays::popover::{
    HOVER_CARD_PLACEMENT, MOTION, PLACEMENT, PopoverSkin, metrics,
};
use crate::rune::{ToRune, sans};
use crate::theme::Theme;

pub use crate::components::overlays::popover::{HoverCardDelays, HoverIntent};

fn surface(theme: &Theme, key: &str, width: f32, children: Vec<El>) -> El {
    let skin = PopoverSkin::new(theme);
    let e = col()
        .key(key)
        .w(width)
        .padding(metrics::PADDING)
        .gap(metrics::GAP)
        .radius(metrics::RADIUS)
        .fill(skin.fill.rune())
        .children(children);
    skin.shadows
        .iter()
        .fold(e, |e, shadow| e.box_shadow(shadow.rune()))
}

/// `PopoverTitle`: `font-medium`.
pub fn popover_title(title: &str, theme: &Theme) -> El {
    text(title)
        .font(sans(
            metrics::FONT_SIZE,
            metrics::TITLE_WEIGHT,
            metrics::LINE_HEIGHT,
        ))
        .color(PopoverSkin::new(theme).text.rune())
}

/// `PopoverDescription`: `text-muted-foreground`.
pub fn popover_description(description: &str, theme: &Theme) -> El {
    text(description)
        .wrap()
        .font(sans(metrics::FONT_SIZE, 400.0, metrics::LINE_HEIGHT))
        .color(PopoverSkin::new(theme).muted.rune())
}

/// `PopoverHeader` with a title and a description (`flex-col gap-0.5`).
pub fn popover_header(title: &str, description: &str, theme: &Theme) -> El {
    col()
        .gap(metrics::HEADER_GAP)
        .child(popover_title(title, theme))
        .child(popover_description(description, theme))
}

/// The popover surface: `w-72 p-2.5 gap-2.5 rounded-lg bg-popover shadow-md ring-1`.
pub struct Popover {
    children: Vec<El>,
    width: f32,
    placement: Placement,
}

impl Default for Popover {
    fn default() -> Self {
        Self::new()
    }
}

impl Popover {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            width: metrics::POPOVER_WIDTH,
            placement: PLACEMENT,
        }
    }

    pub fn child(mut self, child: El) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = El>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// The surface keyed `key`, anchored to the trigger keyed `anchor`. It traps focus,
    /// and Escape or a press outside calls `dismiss`; focus returns to the trigger.
    pub fn build<S: 'static>(
        self,
        theme: &Theme,
        key: &str,
        anchor: &str,
        dismiss: impl Fn(&mut S) + 'static,
    ) -> El {
        let e = surface(theme, key, self.width, self.children)
            .role("dialog")
            .focus_return(anchor)
            .modal(true, dismiss);
        popup(e, anchor, self.placement, &MOTION)
    }
}

/// Wires a hover card's trigger, keyed `key`: resting on it for `delays.open` opens the card,
/// staying off it for `delays.close` lets it close unless the pointer is over the card.
pub fn hover_card_trigger<S: 'static>(
    trigger: El,
    key: &str,
    delays: HoverCardDelays,
    lens: impl Fn(&mut S) -> &mut HoverIntent + 'static,
) -> El {
    trigger
        .key(key)
        .hover_delays(delays.open, delays.close)
        .on_hover_change(move |s: &mut S, inside, _| {
            lens(s).set_trigger(inside, Instant::now());
        })
}

/// The hover card surface: `w-64 p-2.5 rounded-lg bg-popover shadow-md ring-1`.
pub struct HoverCard {
    children: Vec<El>,
    width: f32,
    placement: Placement,
    delays: HoverCardDelays,
}

impl Default for HoverCard {
    fn default() -> Self {
        Self::new()
    }
}

impl HoverCard {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            width: metrics::HOVER_CARD_WIDTH,
            placement: HOVER_CARD_PLACEMENT,
            delays: HoverCardDelays::BASE_UI,
        }
    }

    pub fn child(mut self, child: El) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = El>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// The close delay used while the pointer is over the card.
    pub fn delays(mut self, delays: HoverCardDelays) -> Self {
        self.delays = delays;
        self
    }

    /// The card keyed `key`, anchored to the trigger keyed `anchor`, reporting its hover to
    /// `lens`.
    pub fn build<S: 'static>(
        self,
        theme: &Theme,
        key: &str,
        anchor: &str,
        lens: impl Fn(&mut S) -> &mut HoverIntent + 'static,
    ) -> El {
        let e = surface(theme, key, self.width, self.children)
            .gap(0.0)
            .role("dialog")
            .hover_delays(std::time::Duration::ZERO, self.delays.close)
            .on_hover_change(move |s: &mut S, inside, _| {
                lens(s).set_card(inside, Instant::now());
            });
        popup(e, anchor, self.placement, &MOTION)
    }
}
