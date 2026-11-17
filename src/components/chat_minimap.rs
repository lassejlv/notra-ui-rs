//! The chat minimap: a rail with one line per conversation turn, a hover card that
//! previews a turn, and arrows that step a window of active turns. Shared by the GPUI
//! and Rune front ends.

use std::time::{Duration, Instant};

use crate::motion::{CubicBezier, Tween, ease, lerp};
use crate::theme::{Color, Shadow, Theme};

/// Which side of the rail the preview card opens on (`side` prop).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ChatMinimapSide {
    Left,
    /// The default: lines hang from the left edge and the card opens to the right.
    #[default]
    Right,
}

impl ChatMinimapSide {
    /// Where the card starts its entrance, in px along x (`slide-in-from-*-2`).
    pub fn enter_travel(self) -> f32 {
        match self {
            ChatMinimapSide::Left => CARD_ENTER_TRAVEL,
            ChatMinimapSide::Right => -CARD_ENTER_TRAVEL,
        }
    }
}

/// Rail width (`w-8`); every row spans it.
pub const RAIL_WIDTH: f32 = 32.0;
/// Height of a turn row (`h-2`).
pub const ITEM_HEIGHT: f32 = 8.0;
/// Height of the previous and next buttons (`h-5`).
pub const NAV_HEIGHT: f32 = 20.0;
/// Horizontal padding of a row (`px-1`).
pub const ROW_PADDING_X: f32 = 4.0;
/// The transparent border every shadcn button carries.
pub const ROW_BORDER: f32 = 1.0;
/// Thickness of a turn line (`h-0.5`).
pub const LINE_THICKNESS: f32 = 2.0;
/// The chevron's drawn size (`w-2.5` of a `0 0 10 6` view box).
pub const CHEVRON_SIZE: (f32, f32) = (10.0, 6.0);

/// Card width (`w-64`).
pub const CARD_WIDTH: f32 = 256.0;
/// Distance between the rail and the card (`sideOffset`).
pub const CARD_GAP: f32 = 8.0;
/// How far below the turn's centre the card's centre sits, as measured on the site.
pub const CARD_ALIGN_OFFSET: f32 = 4.0;
/// Card corner radius (`rounded-xl`).
pub const CARD_RADIUS: f32 = 12.0;
/// Card padding (`py-3` on the card, `px-3` on its header).
pub const CARD_PADDING: f32 = 12.0;
/// Space between the title and the description (`gap-0.5`).
pub const CARD_HEADER_GAP: f32 = 2.0;
/// Title: `text-sm font-medium`, one truncated line.
pub const TITLE_FONT: TextMetrics = TextMetrics {
    size: 14.0,
    weight: 500.0,
    line_height: 20.0,
};
/// Description: `text-xs leading-relaxed`, clamped to [`DESCRIPTION_LINES`].
pub const DESCRIPTION_FONT: TextMetrics = TextMetrics {
    size: 12.0,
    weight: 400.0,
    line_height: 19.5,
};
/// `line-clamp-2`.
pub const DESCRIPTION_LINES: usize = 2;

/// Size, weight and line height of a text style, in px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextMetrics {
    pub size: f32,
    pub weight: f32,
    pub line_height: f32,
}

/// Tailwind's default `ease-out` (not Notra's `--ease-out`).
pub const TAILWIND_EASE_OUT: CubicBezier = CubicBezier::new(0.0, 0.0, 0.2, 1.0);
/// A line's width transition (`transition-[width] duration-200 ease-out`).
pub const WIDTH_TRANSITION: Tween = Tween::new(Duration::from_millis(200), TAILWIND_EASE_OUT);
/// A line's colour transition (`transition-colors`).
pub const COLOR_TRANSITION: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);
/// The arrows' reveal (`transition-opacity duration-150`).
pub const NAV_FADE: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);
/// How long the pointer rests on a turn before its card opens.
pub const OPEN_DELAY: Duration = Duration::from_millis(80);
/// How long the card stays after the pointer leaves the turn and the card.
pub const CLOSE_DELAY: Duration = Duration::from_millis(100);
/// The card's entrance (tw-animate-css `fade-in-0 zoom-in-95 slide-in-from-*-2`).
pub const CARD_ENTER: Tween = Tween::new(Duration::from_millis(100), ease::EASE);
/// The card's exit (`fade-out-0 zoom-out-95`).
pub const CARD_EXIT: Tween = Tween::new(Duration::from_millis(100), ease::EASE);
/// The scale the card grows from and shrinks to.
pub const CARD_ENTER_SCALE: f32 = 0.95;
/// How far the card slides in, in px.
pub const CARD_ENTER_TRAVEL: f32 = 8.0;

/// How strongly a turn line is drawn, from the pointer's distance and the turn's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineEmphasis {
    /// Outside the window and away from the pointer.
    Rest,
    /// In the active window, or the turn whose card is open.
    Active,
    /// Under the pointer or keyboard focus.
    Hovered,
    /// One turn away from the hovered one.
    Near,
    /// Two turns away from the hovered one.
    Far,
}

impl LineEmphasis {
    /// The falloff around the hovered turn. It wins over `highlighted` (an active turn
    /// or the turn whose card is open), as the site's arbitrary variants do.
    pub fn of(index: usize, hovered: Option<usize>, highlighted: bool) -> Self {
        match hovered.map(|h| h.abs_diff(index)) {
            Some(0) => LineEmphasis::Hovered,
            Some(1) => LineEmphasis::Near,
            Some(2) => LineEmphasis::Far,
            _ if highlighted => LineEmphasis::Active,
            _ => LineEmphasis::Rest,
        }
    }

    /// Line width in px (`w-3`, `w-6`, `w-5`, `w-4`).
    pub fn width(self) -> f32 {
        match self {
            LineEmphasis::Rest | LineEmphasis::Active => 12.0,
            LineEmphasis::Hovered => 24.0,
            LineEmphasis::Near => 20.0,
            LineEmphasis::Far => 16.0,
        }
    }
}

/// The resolved paint of the minimap for one theme.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMinimapSkin {
    /// `text-muted-foreground/40`.
    pub line_rest: Color,
    /// `text-foreground`: hovered, focused, active and open turns.
    pub line_strong: Color,
    /// `text-foreground/75`.
    pub line_near: Color,
    /// `text-foreground/55`.
    pub line_far: Color,
    /// The arrows at rest (`text-muted-foreground`).
    pub nav: Color,
    /// The arrows under the pointer (`hover:text-foreground`).
    pub nav_hover: Color,
    /// `bg-popover`.
    pub card_fill: Color,
    /// `text-popover-foreground`, the title.
    pub card_title: Color,
    /// `text-muted-foreground`, the description.
    pub card_description: Color,
    /// `ring-1 ring-foreground/10` then `shadow-lg`, as one `box-shadow` list.
    pub card_shadows: [Shadow; 3],
    /// `focus-visible:border-ring`.
    pub focus_border: Color,
    /// `focus-visible:ring-3 ring-ring/50`.
    pub focus_ring: Color,
}

impl ChatMinimapSkin {
    pub fn new(theme: &Theme) -> Self {
        Self {
            line_rest: theme.muted_foreground.fade(0.4),
            line_strong: theme.foreground,
            line_near: theme.foreground.fade(0.75),
            line_far: theme.foreground.fade(0.55),
            nav: theme.muted_foreground,
            nav_hover: theme.foreground,
            card_fill: theme.popover,
            card_title: theme.popover_foreground,
            card_description: theme.muted_foreground,
            card_shadows: [
                Shadow::drop(0.0, 0.0, 1.0, theme.foreground.fade(0.1)),
                Shadow::drop(10.0, 15.0, -3.0, Color::rgba(0x0000001a)),
                Shadow::drop(4.0, 6.0, -4.0, Color::rgba(0x0000001a)),
            ],
            focus_border: theme.ring,
            focus_ring: theme.ring.fade(0.5),
        }
    }

    /// The colour of a line drawn with `emphasis`.
    pub fn line(&self, emphasis: LineEmphasis) -> Color {
        match emphasis {
            LineEmphasis::Rest => self.line_rest,
            LineEmphasis::Active | LineEmphasis::Hovered => self.line_strong,
            LineEmphasis::Near => self.line_near,
            LineEmphasis::Far => self.line_far,
        }
    }
}

/// One frame of the card's entrance or exit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardFrame {
    pub opacity: f32,
    pub scale: f32,
    /// Horizontal shift in px.
    pub offset_x: f32,
}

impl CardFrame {
    pub const SHOWN: CardFrame = CardFrame {
        opacity: 1.0,
        scale: 1.0,
        offset_x: 0.0,
    };

    /// The card `elapsed` into its entrance.
    pub fn entering(elapsed: Duration, side: ChatMinimapSide) -> Self {
        let p = CARD_ENTER.progress(elapsed);
        CardFrame {
            opacity: p,
            scale: lerp(CARD_ENTER_SCALE, 1.0, p),
            offset_x: side.enter_travel() * (1.0 - p),
        }
    }

    /// The card `elapsed` into its exit; it fades and shrinks in place.
    pub fn exiting(elapsed: Duration) -> Self {
        let p = CARD_EXIT.progress(elapsed);
        CardFrame {
            opacity: 1.0 - p,
            scale: lerp(1.0, CARD_ENTER_SCALE, p),
            offset_x: 0.0,
        }
    }
}

/// A turn on the rail: what its card shows and whether it is in the active window.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChatMinimapItem {
    pub title: String,
    pub description: Option<String>,
    pub active: bool,
}

impl ChatMinimapItem {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            active: false,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

/// When the preview card opens and closes as the pointer moves over turns and the card,
/// with the site's open and close delays. A renderer without delayed hover calls
/// [`HoverIntent::tick`] every frame until [`HoverIntent::deadline`] passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HoverIntent {
    pointer: Option<usize>,
    on_card: bool,
    changed_at: Option<Instant>,
    open: Option<usize>,
}

impl HoverIntent {
    /// The turn whose card is open.
    pub fn open(&self) -> Option<usize> {
        self.open
    }

    /// The turn under the pointer, without delay.
    pub fn pointer(&self) -> Option<usize> {
        self.pointer
    }

    /// The pointer entered or left the turn at `index`.
    pub fn hover_item(&mut self, index: usize, hovered: bool, now: Instant) {
        if hovered {
            self.pointer = Some(index);
        } else if self.pointer == Some(index) {
            self.pointer = None;
        } else {
            return;
        }
        self.changed_at = Some(now);
    }

    /// The pointer entered or left the open card, which holds it open.
    pub fn hover_card(&mut self, hovered: bool, now: Instant) {
        if self.on_card != hovered {
            self.on_card = hovered;
            self.changed_at = Some(now);
        }
    }

    /// When the pending change lands, if one is pending.
    pub fn deadline(&self) -> Option<Instant> {
        let at = self.changed_at?;
        let wanted = self.wanted();
        if wanted == self.open {
            return None;
        }
        Some(
            at + if wanted.is_some() {
                OPEN_DELAY
            } else {
                CLOSE_DELAY
            },
        )
    }

    /// Lands a pending change whose time has come; returns whether the open turn changed.
    pub fn tick(&mut self, now: Instant) -> bool {
        match self.deadline() {
            Some(at) if now >= at => {
                self.open = self.wanted();
                true
            }
            _ => false,
        }
    }

    fn wanted(&self) -> Option<usize> {
        match (self.pointer, self.on_card) {
            (Some(index), _) => Some(index),
            (None, true) => self.open,
            (None, false) => None,
        }
    }
}

/// How many turns the demo's window spans (`VISIBLE_TURNS`).
pub const VISIBLE_TURNS: usize = 3;

/// The demo's window of active turns, `start..start + VISIBLE_TURNS`: the arrows step it
/// and a click on a turn centres it there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TurnWindow {
    start: usize,
    len: usize,
}

impl TurnWindow {
    /// A window over `len` turns starting at `start`, clamped to fit.
    pub fn new(len: usize, start: usize) -> Self {
        let mut window = Self { start: 0, len };
        window.start = window.clamp(start);
        window
    }

    pub fn start(&self) -> usize {
        self.start
    }

    /// How many turns there are.
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn clamp(&self, start: usize) -> usize {
        start.min(self.len.saturating_sub(VISIBLE_TURNS))
    }

    /// Whether turn `index` is in the window (`data-active`).
    pub fn is_active(&self, index: usize) -> bool {
        index < self.len && (self.start..self.start + VISIBLE_TURNS).contains(&index)
    }

    pub fn can_previous(&self) -> bool {
        self.start > 0
    }

    pub fn can_next(&self) -> bool {
        self.start < self.clamp(usize::MAX)
    }

    pub fn previous(&mut self) {
        self.start = self.start.saturating_sub(1);
    }

    pub fn next(&mut self) {
        self.start = self.clamp(self.start + 1);
    }

    /// Centres the window on turn `index`.
    pub fn select(&mut self, index: usize) {
        self.start = self.clamp(index.saturating_sub(1));
    }
}

/// A turn's place in a scrolling transcript, in content px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TurnSpan {
    pub top: f32,
    pub height: f32,
}

impl TurnSpan {
    pub fn bottom(&self) -> f32 {
        self.top + self.height
    }
}

/// The scroll spy (`useChatMinimapScrollSpy`): the turns that intersect a viewport
/// scrolled to `scroll_top` and `viewport_height` tall, in order.
pub fn visible_turns(turns: &[TurnSpan], scroll_top: f32, viewport_height: f32) -> Vec<usize> {
    let bottom = scroll_top + viewport_height;
    turns
        .iter()
        .enumerate()
        .filter(|(_, t)| t.top < bottom && t.bottom() > scroll_top)
        .map(|(i, _)| i)
        .collect()
}

/// The scroll offset that brings `turn` to the top of the viewport (`scrollToTurn`),
/// limited to how far the transcript scrolls.
pub fn scroll_offset_for(turn: TurnSpan, content_height: f32, viewport_height: f32) -> f32 {
    turn.top
        .clamp(0.0, (content_height - viewport_height).max(0.0))
}

/// A demo turn's title and description.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DemoTurn {
    pub title: &'static str,
    pub description: &'static str,
}

/// The site's seven demo turns.
pub const DEMO_TURNS: [DemoTurn; 7] = [
    DemoTurn {
        title: "Add a conversation minimap",
        description: "Add a rail that shows how long the conversation is, with one line per turn.",
    },
    DemoTurn {
        title: "Make the lines grow on hover",
        description: "Lines grow on hover and each one opens a preview card beside the rail.",
    },
    DemoTurn {
        title: "Hide the arrows until hover",
        description: "The up and down buttons only show while they are hovered.",
    },
    DemoTurn {
        title: "The background dither is messed up, fix pls",
        description: "The dither sphere was cut off on a straight line at its bottom and left edge.",
    },
    DemoTurn {
        title: "Let the card open on either side",
        description: "Keep the card on the right by default and allow the left.",
    },
    DemoTurn {
        title: "Build it from shadcn primitives",
        description: "Use shadcn Button, Card, HoverCard and Tooltip underneath.",
    },
    DemoTurn {
        title: "Add it to the registry",
        description: "Register the component and write a docs page for it.",
    },
];

/// The default example's first active turn (`start` state).
pub const DEMO_START: usize = 2;
/// The long example's turn count.
pub const LONG_TURNS: usize = 50;
/// The long example's first active turn (`initialStart`).
pub const LONG_START: usize = 20;

/// `count` demo turns, cycling [`DEMO_TURNS`] as the long example does.
pub fn demo_turns(count: usize) -> impl Iterator<Item = DemoTurn> {
    DEMO_TURNS.iter().copied().cycle().take(count)
}

/// Rail items for the demo turns over `window`, its active ones marked.
pub fn demo_items(window: TurnWindow) -> Vec<ChatMinimapItem> {
    demo_turns(window.len())
        .enumerate()
        .map(|(i, t)| {
            ChatMinimapItem::new(t.title)
                .description(t.description)
                .active(window.is_active(i))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falloff_wins_over_active() {
        let hovered = Some(2);
        assert_eq!(LineEmphasis::of(2, hovered, false), LineEmphasis::Hovered);
        assert_eq!(LineEmphasis::of(3, hovered, true), LineEmphasis::Near);
        assert_eq!(LineEmphasis::of(0, hovered, false), LineEmphasis::Far);
        assert_eq!(LineEmphasis::of(4, hovered, true), LineEmphasis::Far);
        assert_eq!(LineEmphasis::of(5, hovered, true), LineEmphasis::Active);
        assert_eq!(LineEmphasis::of(6, hovered, false), LineEmphasis::Rest);
        assert_eq!(LineEmphasis::of(1, None, false), LineEmphasis::Rest);
    }

    #[test]
    fn widths_match_the_site() {
        let widths = [0, 1, 2, 3, 4, 5].map(|i| LineEmphasis::of(i, Some(2), false).width());
        assert_eq!(widths, [16.0, 20.0, 24.0, 20.0, 16.0, 12.0]);
    }

    #[test]
    fn skin_matches_the_measured_colours() {
        let skin = ChatMinimapSkin::new(&Theme::light());
        assert_eq!(skin.line(LineEmphasis::Rest).a, 0.4);
        assert_eq!(skin.line(LineEmphasis::Near).a, 0.75);
        assert_eq!(skin.line(LineEmphasis::Far).a, 0.55);
        assert_eq!(skin.line(LineEmphasis::Active), Theme::light().foreground);
        assert_eq!(
            ChatMinimapSkin::new(&Theme::dark()).card_fill,
            Color::rgb(0x141518)
        );
    }

    #[test]
    fn window_clamps_and_steps() {
        let mut w = TurnWindow::new(7, DEMO_START);
        assert!(w.is_active(2) && w.is_active(4) && !w.is_active(5));
        w.select(6);
        assert_eq!(w.start(), 4);
        assert!(!w.can_next() && w.can_previous());
        w.next();
        assert_eq!(w.start(), 4);
        w.select(0);
        assert_eq!(w.start(), 0);
        assert!(!w.can_previous());
        w.previous();
        assert_eq!(w.start(), 0);
        assert_eq!(TurnWindow::new(2, 5).start(), 0);
        assert!(!TurnWindow::new(2, 0).is_active(2));
        assert_eq!(TurnWindow::new(LONG_TURNS, LONG_START).start(), 20);
    }

    #[test]
    fn hover_intent_waits_before_opening_and_closing() {
        let t0 = Instant::now();
        let mut h = HoverIntent::default();
        h.hover_item(2, true, t0);
        assert_eq!(h.pointer(), Some(2));
        assert!(!h.tick(t0 + Duration::from_millis(40)));
        assert!(h.tick(t0 + OPEN_DELAY));
        assert_eq!(h.open(), Some(2));

        let t1 = t0 + Duration::from_millis(200);
        h.hover_item(2, false, t1);
        h.hover_card(true, t1 + Duration::from_millis(50));
        assert!(!h.tick(t1 + Duration::from_millis(300)));
        assert_eq!(h.open(), Some(2));

        let t2 = t1 + Duration::from_millis(400);
        h.hover_card(false, t2);
        assert!(!h.tick(t2 + Duration::from_millis(50)));
        assert!(h.tick(t2 + CLOSE_DELAY));
        assert_eq!(h.open(), None);
    }

    #[test]
    fn moving_between_turns_switches_the_card() {
        let t0 = Instant::now();
        let mut h = HoverIntent::default();
        h.hover_item(1, true, t0);
        h.tick(t0 + OPEN_DELAY);
        h.hover_item(2, true, t0 + Duration::from_millis(100));
        h.hover_item(1, false, t0 + Duration::from_millis(100));
        assert_eq!(h.pointer(), Some(2));
        assert!(h.tick(t0 + Duration::from_millis(180)));
        assert_eq!(h.open(), Some(2));
    }

    #[test]
    fn card_frames_land_and_travel_from_the_rail() {
        assert_eq!(
            CardFrame::entering(CARD_ENTER.total(), ChatMinimapSide::Right),
            CardFrame::SHOWN
        );
        let start = CardFrame::entering(Duration::ZERO, ChatMinimapSide::Right);
        assert_eq!(
            (start.opacity, start.scale, start.offset_x),
            (0.0, 0.95, -8.0)
        );
        assert_eq!(
            CardFrame::entering(Duration::ZERO, ChatMinimapSide::Left).offset_x,
            8.0
        );
        assert_eq!(CardFrame::exiting(CARD_EXIT.total()).opacity, 0.0);
    }

    #[test]
    fn scroll_spy_reports_intersecting_turns() {
        let turns: Vec<TurnSpan> = (0..5)
            .map(|i| TurnSpan {
                top: i as f32 * 100.0,
                height: 100.0,
            })
            .collect();
        assert_eq!(visible_turns(&turns, 150.0, 200.0), vec![1, 2, 3]);
        assert_eq!(visible_turns(&turns, 100.0, 100.0), vec![1]);
        assert_eq!(scroll_offset_for(turns[3], 500.0, 200.0), 300.0);
        assert_eq!(scroll_offset_for(turns[4], 500.0, 200.0), 300.0);
    }

    #[test]
    fn long_demo_cycles_the_turns() {
        let items = demo_items(TurnWindow::new(LONG_TURNS, LONG_START));
        assert_eq!(items.len(), 50);
        assert_eq!(items[7].title, DEMO_TURNS[0].title);
        assert!(items[20].active && items[22].active && !items[23].active);
    }
}
