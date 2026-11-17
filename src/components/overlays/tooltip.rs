//! Notra's tooltip (`notra-tooltip`) and the CSS-only duotone tooltip: metrics measured on
//! ui.usenotra.com, resolved skins, the arrow geometry, motion, and [`TooltipGroup`], the
//! shared-tooltip state machine behind `TooltipProvider` (one popup that glides between
//! neighbouring triggers).

use std::time::{Duration, Instant};

use super::{Align, Placed, Placement, PopupMotion, Side};
use crate::motion::{Tween, ease};
use crate::theme::{Color, Shadow, Theme};

/// `side="top"`, `align="center"`, `sideOffset={4}`.
pub const PLACEMENT: Placement = Placement::new(Side::Top, Align::Center).side_offset(4.0);

/// Box metrics of the tooltip popup, in logical pixels.
pub mod metrics {
    /// `px-2.5`.
    pub const PADDING_X: f32 = 10.0;
    /// `py-1`.
    pub const PADDING_Y: f32 = 4.0;
    /// `has-[[data-slot=kbd]]:pr-1`: the end padding when the content ends in keys.
    pub const PADDING_END_WITH_KBD: f32 = 4.0;
    /// `rounded-[0.625rem]` with `corner-shape: squircle`.
    pub const RADIUS: f32 = 10.0;
    /// The circular radius that draws the squircle's corner: a `superellipse(2)` corner of
    /// radius 10 cuts 1.6 px in at 45°, as a circular corner of radius 5.5 does. Renderers
    /// without squircles round with this.
    pub const CORNER: f32 = 5.5;
    /// `max-w-xs`.
    pub const MAX_WIDTH: f32 = 320.0;
    /// `text-xs`.
    pub const FONT_SIZE: f32 = 12.0;
    pub const LINE_HEIGHT: f32 = 16.0;
    pub const FONT_WEIGHT: f32 = 400.0;
    pub const BORDER: f32 = 1.0;
    /// The arrow: a `size-2.5` square turned 45° with `rounded-[2px]` corners.
    pub const ARROW_SIZE: f32 = 10.0;
    pub const ARROW_RADIUS: f32 = 2.0;
}

/// shadcn's `Kbd` inside a tooltip: `h-5 min-w-5 px-1 rounded-sm text-xs font-medium`.
pub mod kbd {
    pub const HEIGHT: f32 = 20.0;
    pub const MIN_WIDTH: f32 = 20.0;
    pub const PADDING_X: f32 = 4.0;
    pub const RADIUS: f32 = 4.0;
    pub const FONT_SIZE: f32 = 12.0;
    pub const LINE_HEIGHT: f32 = 16.0;
    pub const FONT_WEIGHT: f32 = 500.0;
    /// `KbdGroup`'s `gap-1`.
    pub const GAP: f32 = 4.0;
    /// `ms-1.5` before the group.
    pub const MARGIN_START: f32 = 6.0;
}

/// The popup's paint for one theme: an inverted chip (dark in light mode, light in dark).
#[derive(Clone, Debug, PartialEq)]
pub struct TooltipSkin {
    /// `bg-linear-to-b` from `color-mix(foreground, background 10%)` to `foreground`.
    pub gradient: (Color, Color),
    /// `border-foreground/60`.
    pub border: Color,
    /// `text-background`.
    pub text: Color,
    pub shadows: Vec<Shadow>,
    /// `--tooltip-arrow`: the foreground.
    pub arrow: Color,
    /// `bg-background/20` (`/10` in dark mode).
    pub kbd_fill: Color,
    pub kbd_text: Color,
}

impl TooltipSkin {
    pub fn new(theme: &Theme) -> Self {
        Self {
            gradient: (
                theme.foreground.mix(theme.background, 0.1),
                theme.foreground,
            ),
            border: theme.foreground.fade(0.6),
            text: theme.background,
            shadows: vec![
                Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffff38)),
                Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x00000029)),
                Shadow::drop(6.0, 16.0, -4.0, Color::rgba(0x0000003d)),
            ],
            arrow: theme.foreground,
            kbd_fill: theme
                .background
                .fade(if theme.is_dark() { 0.1 } else { 0.2 }),
            kbd_text: theme.background,
        }
    }
}

/// How far the arrow's centre sits inside the popup edge facing the anchor, per side, as
/// measured on the live site (Base UI positions the popup differently per side, so the
/// classes `-bottom-2.5`, `top-1`, `-left-1` and `-right-1` land unevenly).
pub fn arrow_inset(side: Side) -> f32 {
    match side {
        Side::Top => 3.0,
        Side::Bottom => 2.0,
        Side::Right => 1.0,
        Side::Left => 2.0,
    }
}

/// The arrow's centre relative to the popup's top-left corner (its border box): on the edge
/// facing the anchor, at the anchor's centre for top and bottom popups and halfway down for
/// left and right ones (`top-1/2!`).
pub fn arrow_center(placed: &Placed) -> (f32, f32) {
    let (w, h) = (placed.rect.w, placed.rect.h);
    let inset = arrow_inset(placed.side);
    match placed.side {
        Side::Top => (placed.arrow, h - inset),
        Side::Bottom => (placed.arrow, inset),
        Side::Left => (w - inset, h / 2.0),
        Side::Right => (inset, h / 2.0),
    }
}

/// Opacity, `scale-[0.97]` and a 2 px slide from the anchor over 150 ms in; opacity and scale
/// over 100 ms out, on `cubic-bezier(0.23, 1, 0.32, 1)`.
pub const MOTION: PopupMotion = PopupMotion {
    enter: Tween::new(Duration::from_millis(150), ease::OUT),
    exit: Tween::new(Duration::from_millis(100), ease::OUT),
    scale: 0.97,
    enter_travel: 2.0,
    exit_travel: 0.0,
};

/// The shared popup's glide to a new trigger: the positioner's `top`/`left` and the arrow
/// over 200 ms.
pub const GLIDE: Tween = Tween::new(Duration::from_millis(200), ease::OUT);

/// The shared popup's width and height following new content, over 150 ms.
pub const RESIZE: Tween = Tween::new(Duration::from_millis(150), ease::OUT);

/// Base UI's hover timing for a group of tooltips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TooltipDelays {
    /// Rest on a trigger this long before it opens.
    pub open: Duration,
    /// Stay open this long after the pointer leaves.
    pub close: Duration,
    /// A tooltip opening within this long of another closing opens at once, without its
    /// enter transition.
    pub timeout: Duration,
}

impl TooltipDelays {
    /// Base UI's trigger defaults: 600 ms to open, closing at once, a 400 ms warm window.
    pub const BASE_UI: TooltipDelays = TooltipDelays {
        open: Duration::from_millis(600),
        close: Duration::ZERO,
        timeout: Duration::from_millis(400),
    };

    /// Notra's `TooltipProvider` default (`delay = 0`).
    pub const PROVIDER: TooltipDelays = TooltipDelays {
        open: Duration::ZERO,
        ..Self::BASE_UI
    };

    /// `<TooltipProvider delay={ms}>`.
    pub const fn with_open(mut self, open: Duration) -> Self {
        self.open = open;
        self
    }
}

impl Default for TooltipDelays {
    fn default() -> Self {
        Self::PROVIDER
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Pending<K> {
    Open(K, Instant),
    Close(Instant),
}

/// The open state of a group of tooltips sharing one popup (Base UI's `Tooltip.Provider` with
/// a shared handle): which trigger it points at, the open and close delays, and the warm window
/// in which the next trigger opens at once.
///
/// Hovering another trigger while the popup is open moves it there without closing (the
/// renderer glides it). Focus opens at once, Escape and a press on the trigger close at once.
/// Feed it pointer and focus events and call [`TooltipGroup::tick`] at
/// [`TooltipGroup::deadline`].
#[derive(Clone, Debug, PartialEq)]
pub struct TooltipGroup<K> {
    delays: TooltipDelays,
    active: Option<K>,
    open: bool,
    instant: bool,
    pending: Option<Pending<K>>,
    closed_at: Option<Instant>,
}

impl<K: Clone + PartialEq> Default for TooltipGroup<K> {
    fn default() -> Self {
        Self::new(TooltipDelays::default())
    }
}

impl<K: Clone + PartialEq> TooltipGroup<K> {
    pub fn new(delays: TooltipDelays) -> Self {
        Self {
            delays,
            active: None,
            open: false,
            instant: false,
            pending: None,
            closed_at: None,
        }
    }

    pub fn delays(&self) -> TooltipDelays {
        self.delays
    }

    /// The trigger the popup points at (still set while it closes).
    pub fn active(&self) -> Option<&K> {
        self.active.as_ref()
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Whether the popup is open on `key`.
    pub fn is_open_on(&self, key: &K) -> bool {
        self.open && self.active.as_ref() == Some(key)
    }

    /// Whether the last open or close skips its transition (Base UI's `data-instant`): an open
    /// inside the warm window or from focus, a close from Escape or a press.
    pub fn is_instant(&self) -> bool {
        self.instant
    }

    /// When it last closed.
    pub fn closed_at(&self) -> Option<Instant> {
        self.closed_at
    }

    /// Open, or closed less than [`TooltipDelays::timeout`] ago: the next trigger opens at once.
    pub fn is_warm(&self, now: Instant) -> bool {
        self.open
            || self
                .closed_at
                .is_some_and(|at| now.saturating_duration_since(at) < self.delays.timeout)
    }

    /// How long the pointer has to rest on a trigger before it opens, given the warm window.
    pub fn open_delay(&self, now: Instant) -> Duration {
        if self.is_warm(now) {
            Duration::ZERO
        } else {
            self.delays.open
        }
    }

    /// When the next timer fires.
    pub fn deadline(&self) -> Option<Instant> {
        match self.pending {
            Some(Pending::Open(_, at) | Pending::Close(at)) => Some(at),
            None => None,
        }
    }

    /// The pointer entered `key`. Returns whether the visible state changed.
    pub fn hover(&mut self, key: K, now: Instant) -> bool {
        let delay = self.open_delay(now);
        self.enter(key, now, delay)
    }

    /// The pointer has rested on `key` for [`TooltipGroup::open_delay`], for toolkits that run
    /// the delay themselves (Rune's `hover_delays`). Returns whether the visible state changed.
    pub fn rest(&mut self, key: K, now: Instant) -> bool {
        self.enter(key, now, Duration::ZERO)
    }

    fn enter(&mut self, key: K, now: Instant, delay: Duration) -> bool {
        if self.open {
            self.pending = None;
            if self.active.as_ref() == Some(&key) {
                return false;
            }
            self.active = Some(key);
            self.instant = false;
            return true;
        }
        if delay.is_zero() {
            let warm = self.is_warm(now);
            self.show(key, warm);
            return true;
        }
        self.pending = Some(Pending::Open(key, now + delay));
        false
    }

    /// The pointer left `key`. Returns whether the visible state changed.
    pub fn unhover(&mut self, key: &K, now: Instant) -> bool {
        let delay = self.delays.close;
        self.exit(key, now, delay)
    }

    /// The pointer has stayed off `key` for the close delay, for toolkits that run the delay
    /// themselves. Returns whether the visible state changed.
    pub fn leave(&mut self, key: &K, now: Instant) -> bool {
        self.exit(key, now, Duration::ZERO)
    }

    fn exit(&mut self, key: &K, now: Instant, delay: Duration) -> bool {
        if matches!(&self.pending, Some(Pending::Open(pending, _)) if pending == key) {
            self.pending = None;
            return false;
        }
        if !self.is_open_on(key) {
            return false;
        }
        if delay.is_zero() {
            self.hide(now, false);
            return true;
        }
        self.pending = Some(Pending::Close(now + delay));
        false
    }

    /// `key` took keyboard focus (`:focus-visible`): it opens at once, without a transition.
    pub fn focus(&mut self, key: K) -> bool {
        if self.is_open_on(&key) {
            return false;
        }
        self.show(key, true);
        true
    }

    /// `key` lost focus.
    pub fn blur(&mut self, key: &K, now: Instant) -> bool {
        if !self.is_open_on(key) {
            return false;
        }
        self.hide(now, false);
        true
    }

    /// Escape or a press on the trigger: closes at once, without a transition.
    pub fn dismiss(&mut self, now: Instant) -> bool {
        self.pending = None;
        if !self.open {
            return false;
        }
        self.hide(now, true);
        true
    }

    /// Runs the timers due at `now`. Returns whether the visible state changed.
    pub fn tick(&mut self, now: Instant) -> bool {
        match self.pending.clone() {
            Some(Pending::Open(key, at)) if now >= at => {
                self.show(key, false);
                true
            }
            Some(Pending::Close(at)) if now >= at => {
                self.hide(now, false);
                true
            }
            _ => false,
        }
    }

    fn show(&mut self, key: K, instant: bool) {
        self.pending = None;
        self.active = Some(key);
        self.open = true;
        self.instant = instant;
    }

    fn hide(&mut self, now: Instant, instant: bool) {
        self.pending = None;
        self.open = false;
        self.instant = instant;
        self.closed_at = Some(now);
    }
}

/// The duotone tooltip: a CSS-only card below its trigger with a shell header (title and an
/// aside) and a content card inside a 2 px rim.
pub mod duotone {
    use std::time::Duration;

    use crate::motion::{Tween, ease};
    use crate::theme::{Color, Shadow, Theme};

    /// `w-72`.
    pub const WIDTH: f32 = 288.0;
    /// `pt-2`: the hover bridge between the trigger and the card.
    pub const GAP: f32 = 8.0;
    /// `translate-y-1`: hidden, the card sits 4 px lower.
    pub const TRAVEL: f32 = 4.0;
    /// `0.5rem`: the card shifts horizontally to stay this far inside the viewport.
    pub const VIEWPORT_MARGIN: f32 = 8.0;
    /// `transition duration-150 ease-out` (the site's `--ease-out`, measured live).
    pub const MOTION: Tween = Tween::new(Duration::from_millis(150), ease::OUT);

    /// `rounded-2xl`.
    pub const SHELL_RADIUS: f32 = 16.0;
    /// `p-0.5 pt-0`: the rim around the card.
    pub const RIM: f32 = 2.0;
    /// Header `px-2.5 py-2 gap-3`.
    pub const HEADER_PADDING_X: f32 = 10.0;
    pub const HEADER_PADDING_Y: f32 = 8.0;
    pub const HEADER_GAP: f32 = 12.0;
    /// `rounded-[14px]`.
    pub const CARD_RADIUS: f32 = 14.0;
    /// Card `px-3 py-2.5`.
    pub const CARD_PADDING_X: f32 = 12.0;
    pub const CARD_PADDING_Y: f32 = 10.0;
    /// The paragraph inside the card keeps the docs page's 16 px prose margin, measured live.
    pub const BODY_MARGIN_Y: f32 = 16.0;

    /// The trigger: `rounded-md border px-3 py-1.5 text-sm`, `hover:bg-muted`.
    pub mod trigger {
        pub const RADIUS: f32 = 6.0;
        pub const PADDING_X: f32 = 12.0;
        pub const PADDING_Y: f32 = 6.0;
        pub const FONT_SIZE: f32 = 14.0;
        pub const LINE_HEIGHT: f32 = 20.0;
    }

    /// Resolved colours for one theme.
    #[derive(Clone, Debug, PartialEq)]
    pub struct DuotoneSkin {
        pub shell: Color,
        pub shell_border: Color,
        /// `shadow-md`.
        pub shadows: Vec<Shadow>,
        pub title: Color,
        pub aside: Color,
        pub card: Color,
        pub card_border: Color,
        pub body: Color,
        pub trigger_fill: Color,
        pub trigger_hover: Color,
        pub trigger_border: Color,
        pub trigger_text: Color,
    }

    impl DuotoneSkin {
        pub fn new(theme: &Theme) -> Self {
            Self {
                shell: theme.shell,
                shell_border: theme.shell_border,
                shadows: crate::components::overlays::popover::SHADOW_MD.to_vec(),
                title: theme.foreground,
                aside: theme.muted_foreground,
                card: theme.background,
                card_border: theme.border,
                body: theme.muted_foreground,
                trigger_fill: theme.background,
                trigger_hover: theme.muted,
                trigger_border: theme.border,
                trigger_text: theme.foreground,
            }
        }
    }

    /// The horizontal shift that keeps a card `[left, left + width]` inside `viewport_width`
    /// with [`VIEWPORT_MARGIN`] to spare, as the page's script sets `margin-left`.
    pub fn shift(left: f32, width: f32, viewport_width: f32) -> f32 {
        let m = VIEWPORT_MARGIN;
        (m - left).max(0.0) - (left + width - (viewport_width - m)).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::overlays::{Collision, Rect, place};
    use crate::theme::Appearance;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn skin_matches_the_live_tooltip() {
        let light = TooltipSkin::new(&Theme::new(Appearance::Light));
        assert_eq!(light.text, Color::rgb(0xffffff));
        assert_eq!(light.gradient.1, Color::rgb(0x171717));
        assert_eq!(light.arrow, Color::rgb(0x171717));
        let dark = TooltipSkin::new(&Theme::new(Appearance::Dark));
        assert_eq!(dark.gradient.1, Color::rgb(0xfafafa));
        assert!((dark.kbd_fill.a - 0.1).abs() < 1e-3);
    }

    #[test]
    fn arrow_sits_on_the_anchor_centre_inside_the_edge() {
        let trigger = Rect::new(371.5625, 334.0, 45.890625, 32.0);
        let p = place(
            trigger,
            (120.234375, 26.0),
            Rect::new(0.0, 0.0, 1000.0, 700.0),
            PLACEMENT,
            Collision::DEFAULT,
        );
        let (x, y) = arrow_center(&p);
        assert!((p.rect.x + x - 394.5).abs() < 0.01);
        assert!((p.rect.y + y - 327.0).abs() < 0.01);
        let right = Placed {
            side: Side::Right,
            ..p
        };
        assert_eq!(arrow_center(&right), (1.0, 13.0));
    }

    #[test]
    fn opens_after_the_delay_and_cancels_on_leave() {
        let t0 = Instant::now();
        let mut g = TooltipGroup::new(TooltipDelays::PROVIDER.with_open(ms(400)));
        assert!(!g.hover("bold", t0));
        assert_eq!(g.deadline(), Some(t0 + ms(400)));
        assert!(!g.tick(t0 + ms(399)));
        assert!(g.tick(t0 + ms(400)));
        assert!(g.is_open_on(&"bold") && !g.is_instant());

        let mut g = TooltipGroup::new(TooltipDelays::BASE_UI);
        g.hover("bold", t0);
        assert!(!g.unhover(&"bold", t0 + ms(100)));
        assert!(!g.tick(t0 + ms(700)));
        assert!(!g.is_open());
    }

    #[test]
    fn moving_to_a_neighbour_while_open_glides_without_closing() {
        let t0 = Instant::now();
        let mut g = TooltipGroup::new(TooltipDelays::PROVIDER.with_open(ms(400)));
        g.hover("bold", t0);
        g.tick(t0 + ms(400));
        assert!(g.hover("italic", t0 + ms(900)));
        assert!(g.is_open_on(&"italic") && !g.is_instant());
    }

    #[test]
    fn warm_window_opens_the_next_trigger_at_once() {
        let t0 = Instant::now();
        let mut g = TooltipGroup::new(TooltipDelays::PROVIDER.with_open(ms(400)));
        g.hover("bold", t0);
        g.tick(t0 + ms(400));
        assert!(g.unhover(&"bold", t0 + ms(500)));
        assert!(!g.is_open() && g.active() == Some(&"bold"));
        assert_eq!(g.open_delay(t0 + ms(600)), Duration::ZERO);
        assert!(g.hover("quote", t0 + ms(600)));
        assert!(g.is_open_on(&"quote") && g.is_instant());

        g.unhover(&"quote", t0 + ms(700));
        assert_eq!(g.open_delay(t0 + ms(1100)), ms(400));
        assert!(!g.hover("code", t0 + ms(1100)));
    }

    #[test]
    fn close_delay_and_dismissal() {
        let t0 = Instant::now();
        let mut g = TooltipGroup::new(TooltipDelays {
            close: ms(300),
            ..TooltipDelays::PROVIDER
        });
        assert!(g.hover(1, t0));
        assert!(!g.unhover(&1, t0 + ms(10)));
        assert_eq!(g.deadline(), Some(t0 + ms(310)));
        assert!(!g.hover(1, t0 + ms(100)));
        assert_eq!(g.deadline(), None);
        assert!(g.dismiss(t0 + ms(200)));
        assert!(!g.is_open() && g.is_instant());
        assert!(g.focus(2));
        assert!(g.is_open_on(&2) && g.is_instant());
        assert!(g.blur(&2, t0 + ms(300)));
    }

    #[test]
    fn rest_and_leave_skip_the_delays() {
        let t0 = Instant::now();
        let mut g = TooltipGroup::new(TooltipDelays {
            close: ms(300),
            ..TooltipDelays::BASE_UI
        });
        assert_eq!(g.open_delay(t0), ms(600));
        assert!(g.rest("a", t0));
        assert!(g.is_open_on(&"a") && !g.is_instant());
        assert!(g.leave(&"a", t0 + ms(10)));
        assert_eq!(g.open_delay(t0 + ms(20)), Duration::ZERO);
    }

    #[test]
    fn duotone_shifts_inside_the_viewport() {
        assert_eq!(duotone::shift(100.0, 288.0, 1000.0), 0.0);
        assert_eq!(duotone::shift(-20.0, 288.0, 1000.0), 28.0);
        assert_eq!(duotone::shift(800.0, 288.0, 1000.0), -96.0);
    }
}
