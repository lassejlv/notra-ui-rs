//! shadcn's base-nova Popover and Hover Card (Base UI `Popover` and `PreviewCard`), the
//! surfaces Notra's blocks build on: placement, measured metrics, the resolved skin, the
//! `tw-animate-css` enter and exit, and [`HoverIntent`], the hover card's open/close delays.

use std::time::{Duration, Instant};

use super::{Align, Placement, PopupMotion, Side};
use crate::motion::{Tween, ease};
use crate::theme::{Color, Shadow, Theme};

/// The popover: `side="bottom"`, `align="center"`, `sideOffset={4}`.
pub const PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Center).side_offset(4.0);

/// The hover card: the popover's placement with `alignOffset={4}`.
pub const HOVER_CARD_PLACEMENT: Placement = PLACEMENT.align_offset(4.0);

/// Box metrics, in logical pixels.
pub mod metrics {
    /// `w-72`.
    pub const POPOVER_WIDTH: f32 = 288.0;
    /// `w-64`.
    pub const HOVER_CARD_WIDTH: f32 = 256.0;
    /// `p-2.5`.
    pub const PADDING: f32 = 10.0;
    /// The popover's `flex-col gap-2.5`.
    pub const GAP: f32 = 10.0;
    /// `rounded-lg`.
    pub const RADIUS: f32 = 8.0;
    /// `text-sm`.
    pub const FONT_SIZE: f32 = 14.0;
    pub const LINE_HEIGHT: f32 = 20.0;
    /// `PopoverHeader`'s `gap-0.5`.
    pub const HEADER_GAP: f32 = 2.0;
    /// `PopoverTitle`'s `font-medium`.
    pub const TITLE_WEIGHT: f32 = 500.0;
}

/// Tailwind's `shadow-md`.
pub const SHADOW_MD: [Shadow; 2] = [
    Shadow::drop(4.0, 6.0, -1.0, Color::rgba(0x0000001a)),
    Shadow::drop(2.0, 4.0, -2.0, Color::rgba(0x0000001a)),
];

/// The surface's paint for one theme.
#[derive(Clone, Debug, PartialEq)]
pub struct PopoverSkin {
    /// `bg-popover`.
    pub fill: Color,
    /// `text-popover-foreground`.
    pub text: Color,
    /// `PopoverDescription`'s `text-muted-foreground`.
    pub muted: Color,
    /// `ring-1 ring-foreground/10` under `shadow-md`.
    pub shadows: Vec<Shadow>,
}

impl PopoverSkin {
    pub fn new(theme: &Theme) -> Self {
        let ring = Shadow::drop(0.0, 0.0, 1.0, theme.foreground.fade(0.1));
        Self {
            fill: theme.popover,
            text: theme.popover_foreground,
            muted: theme.muted_foreground,
            shadows: vec![ring, SHADOW_MD[0], SHADOW_MD[1]],
        }
    }
}

/// `duration-100` with `tw-animate-css`'s `ease`: in, `fade-in-0 zoom-in-95` and
/// `slide-in-from-*-2` (8 px from the anchor); out, `fade-out-0 zoom-out-95`.
pub const MOTION: PopupMotion = PopupMotion {
    enter: Tween::new(Duration::from_millis(100), ease::EASE),
    exit: Tween::new(Duration::from_millis(100), ease::EASE),
    scale: 0.95,
    enter_travel: 8.0,
    exit_travel: 0.0,
};

/// Base UI `PreviewCard` timing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoverCardDelays {
    /// Rest on the trigger this long before the card opens.
    pub open: Duration,
    /// Away from both the trigger and the card this long before it closes.
    pub close: Duration,
}

impl HoverCardDelays {
    /// Base UI's defaults: 600 ms to open, 300 ms to close.
    pub const BASE_UI: HoverCardDelays = HoverCardDelays {
        open: Duration::from_millis(600),
        close: Duration::from_millis(300),
    };
}

impl HoverCardDelays {
    /// No delays: for a [`HoverIntent`] whose toolkit runs the delays itself (Rune's
    /// `hover_delays`), so it only combines the trigger's and the card's hover.
    pub const NONE: HoverCardDelays = HoverCardDelays {
        open: Duration::ZERO,
        close: Duration::ZERO,
    };
}

impl Default for HoverCardDelays {
    fn default() -> Self {
        Self::BASE_UI
    }
}

/// Whether a hover card is open: it opens once the pointer rests on its trigger, and stays
/// open while the pointer is over the trigger or the card, closing a moment after it leaves
/// both. Feed it hover changes and call [`HoverIntent::tick`] at [`HoverIntent::deadline`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoverIntent {
    delays: HoverCardDelays,
    trigger: bool,
    card: bool,
    open: bool,
    pending: Option<(bool, Instant)>,
}

impl Default for HoverIntent {
    fn default() -> Self {
        Self::new(HoverCardDelays::default())
    }
}

impl HoverIntent {
    pub fn new(delays: HoverCardDelays) -> Self {
        Self {
            delays,
            trigger: false,
            card: false,
            open: false,
            pending: None,
        }
    }

    pub fn delays(&self) -> HoverCardDelays {
        self.delays
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// When the next timer fires.
    pub fn deadline(&self) -> Option<Instant> {
        self.pending.map(|(_, at)| at)
    }

    /// The pointer entered or left the trigger. Returns whether the card opened or closed.
    pub fn set_trigger(&mut self, inside: bool, now: Instant) -> bool {
        self.trigger = inside;
        self.settle(now)
    }

    /// The pointer entered or left the open card. Returns whether the card opened or closed.
    pub fn set_card(&mut self, inside: bool, now: Instant) -> bool {
        self.card = inside;
        self.settle(now)
    }

    /// Escape, a press outside or a press on the trigger: closes at once.
    pub fn close(&mut self) -> bool {
        self.pending = None;
        self.card = false;
        std::mem::replace(&mut self.open, false)
    }

    /// Runs the timer due at `now`. Returns whether the card opened or closed.
    pub fn tick(&mut self, now: Instant) -> bool {
        match self.pending {
            Some((open, at)) if now >= at => {
                self.pending = None;
                self.open = open;
                true
            }
            _ => false,
        }
    }

    fn settle(&mut self, now: Instant) -> bool {
        let want = self.trigger || (self.open && self.card);
        if want == self.open {
            self.pending = None;
            return false;
        }
        if self.pending.is_some_and(|(open, _)| open == want) {
            return false;
        }
        let delay = if want {
            self.delays.open
        } else {
            self.delays.close
        };
        if delay.is_zero() {
            self.pending = None;
            self.open = want;
            return true;
        }
        self.pending = Some((want, now + delay));
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Appearance;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn skin_uses_the_popover_tokens() {
        let t = Theme::new(Appearance::Light);
        let s = PopoverSkin::new(&t);
        assert_eq!(s.fill, t.popover);
        assert_eq!(s.shadows[0].spread, 1.0);
        assert_eq!(s.shadows.len(), 3);
    }

    #[test]
    fn hover_card_opens_after_the_delay_and_stays_open_over_the_card() {
        let t0 = Instant::now();
        let mut h = HoverIntent::default();
        assert!(!h.set_trigger(true, t0));
        assert_eq!(h.deadline(), Some(t0 + ms(600)));
        assert!(h.tick(t0 + ms(600)));
        assert!(h.is_open());
        assert!(!h.set_trigger(false, t0 + ms(700)));
        assert!(!h.set_card(true, t0 + ms(800)));
        assert_eq!(h.deadline(), None);
        assert!(!h.tick(t0 + ms(2000)));
        assert!(h.is_open());
        h.set_card(false, t0 + ms(2000));
        assert!(h.tick(t0 + ms(2300)));
        assert!(!h.is_open());
    }

    #[test]
    fn a_brief_pass_never_opens() {
        let t0 = Instant::now();
        let mut h = HoverIntent::default();
        h.set_trigger(true, t0);
        h.set_trigger(false, t0 + ms(200));
        assert_eq!(h.deadline(), None);
        assert!(!h.tick(t0 + ms(900)));
        assert!(!h.is_open());
    }

    #[test]
    fn close_is_immediate() {
        let t0 = Instant::now();
        let mut h = HoverIntent::new(HoverCardDelays {
            open: Duration::ZERO,
            close: ms(300),
        });
        assert!(h.set_trigger(true, t0));
        assert!(h.close());
        assert!(!h.is_open());
    }
}
