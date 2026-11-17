//! The Notra dashboard composer: a dualtone input card (or a round pill) with an
//! auto-growing textarea, a toolbar, the send button's states, and a tray of queued
//! messages, attachments and notices tucked under its top edge.
//!
//! Both renderers read these metrics, skins and timings, so the card measures the
//! same in GPUI and Rune.

use std::time::Duration;

use crate::components::button::{ButtonHover, ButtonSkin, ButtonVariant, DEPTH_RADIUS};
use crate::components::input::FieldMetrics;
use crate::motion::{CubicBezier, Loop, Tween, ease};
use crate::theme::{Color, Shadow, Theme};

/// Card, or a round pill for inline composers such as comment replies (`flat`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ComposerShape {
    #[default]
    Card,
    Flat,
}

/// `Sm` is the dense 36 px pill paired with the 28 px send; it only applies to `Flat`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ComposerSize {
    #[default]
    Default,
    Sm,
}

/// The card: `rounded-[10px] border bg-table-card shadow-table-card`.
pub const CARD_RADIUS: f32 = 10.0;

/// The tray under the card: `mx-1.5 rounded-t-[9px] border border-b-0 pt-1 pb-2`, and the
/// card's `-mt-2` that hides the tray's bottom padding.
pub mod tray {
    pub const INSET_X: f32 = 6.0;
    pub const RADIUS: f32 = 9.0;
    pub const PADDING_TOP: f32 = 4.0;
    pub const PADDING_BOTTOM: f32 = 8.0;
    pub const OVERLAP: f32 = 8.0;
}

/// The pill: `p-1 rounded-[29px]` tray around a `rounded-[25px] border-border/70` card.
pub mod pill {
    use super::ComposerSize;

    /// The tray's padding while it holds a nudge.
    pub const TRAY_PADDING: f32 = 4.0;
    /// The tray's top corners while it holds a nudge (`rounded-t-[18px]`).
    pub const TRAY_TOP_RADIUS: f32 = 18.0;
    /// The row inside the pill: `flex items-end gap-2 p-2`.
    pub const ROW_PADDING: f32 = 8.0;
    pub const ROW_GAP: f32 = 8.0;

    pub fn tray_radius(size: ComposerSize) -> f32 {
        match size {
            ComposerSize::Default => 29.0,
            ComposerSize::Sm => 22.0,
        }
    }

    pub fn card_radius(size: ComposerSize) -> f32 {
        match size {
            ComposerSize::Default => 25.0,
            ComposerSize::Sm => 18.0,
        }
    }
}

/// `ComposerNudge`: `flex items-center gap-2 px-2.25 pt-0.5 pb-1`; chips wrap with `gap-1.5`.
pub mod nudge {
    pub const PADDING_TOP: f32 = 2.0;
    pub const PADDING_X: f32 = 9.0;
    pub const PADDING_BOTTOM: f32 = 4.0;
    pub const GAP: f32 = 8.0;
    pub const CHIP_GAP: f32 = 6.0;
    /// The title: `text-xs font-medium`.
    pub const TITLE_SIZE: f32 = 12.0;
    pub const TITLE_LINE: f32 = 16.0;
    pub const TITLE_WEIGHT: f32 = 500.0;
    /// Free content (the limit notice): `text-sm`.
    pub const BODY_SIZE: f32 = 14.0;
    pub const BODY_LINE: f32 = 20.0;
}

/// `ComposerInput` in the card: `min-h-12 max-h-50 px-3 py-2 text-sm leading-6`.
pub const CARD_INPUT: FieldMetrics = FieldMetrics {
    min_height: 48.0,
    max_height: Some(200.0),
    padding_x: 12.0,
    padding_y: 8.0,
    radius: 0.0,
    border: 0.0,
    font_size: 14.0,
    line_height: 24.0,
};

/// `ComposerInput` in the pill: `max-h-32 min-h-7 px-2 py-1 leading-5`.
pub const FLAT_INPUT: FieldMetrics = FieldMetrics {
    min_height: 28.0,
    max_height: Some(128.0),
    padding_x: 8.0,
    padding_y: 4.0,
    radius: 0.0,
    border: 0.0,
    font_size: 14.0,
    line_height: 20.0,
};

/// `ComposerToolbar`: `flex items-center gap-1 px-2 pt-1 pb-2`.
pub mod toolbar {
    pub const PADDING_TOP: f32 = 4.0;
    pub const PADDING_X: f32 = 8.0;
    pub const PADDING_BOTTOM: f32 = 8.0;
    pub const GAP: f32 = 4.0;
}

/// `ComposerToolbarButton`: a ghost `sm` button with `rounded-md px-2 text-sm`.
pub mod toolbar_button {
    pub const HEIGHT: f32 = 28.0;
    pub const PADDING_X: f32 = 8.0;
    pub const GAP: f32 = 4.0;
    pub const RADIUS: f32 = 6.0;
    pub const FONT_SIZE: f32 = 14.0;
    pub const LINE_HEIGHT: f32 = 20.0;
    pub const FONT_WEIGHT: f32 = 500.0;
    /// Icons without their own size class (`size-3.5`).
    pub const ICON: f32 = 14.0;
    /// A trailing chevron (`size-3`).
    pub const CHEVRON: f32 = 12.0;
    /// `size="icon-sm"`: square, with a 16 px icon.
    pub const ICON_ONLY_ICON: f32 = 16.0;
}

/// `ComposerSend`.
pub mod send {
    use super::ComposerSize;

    /// The button's side (`size-8`, `size-7`).
    pub fn size(size: ComposerSize) -> f32 {
        match size {
            ComposerSize::Default => 32.0,
            ComposerSize::Sm => 28.0,
        }
    }

    /// The arrow (`size-4`, `size-3.5`).
    pub fn icon(size: ComposerSize) -> f32 {
        match size {
            ComposerSize::Default => 16.0,
            ComposerSize::Sm => 14.0,
        }
    }

    /// The stop square (`size-3`).
    pub const STOP_ICON: f32 = 12.0;

    /// The spinner while busy (`size-3.5`, `size-3`).
    pub fn spinner(size: ComposerSize) -> f32 {
        match size {
            ComposerSize::Default => 14.0,
            ComposerSize::Sm => 12.0,
        }
    }
}

/// `ComposerChip`: `inline-flex gap-1.5 rounded-md border border-dashed py-1 pr-1 pl-1.5 text-xs`.
pub mod chip {
    pub const PADDING_LEFT: f32 = 6.0;
    pub const PADDING_RIGHT: f32 = 4.0;
    pub const PADDING_Y: f32 = 4.0;
    pub const GAP: f32 = 6.0;
    pub const RADIUS: f32 = 6.0;
    pub const FONT_SIZE: f32 = 12.0;
    pub const LINE_HEIGHT: f32 = 16.0;
    /// `max-w-48 truncate`.
    pub const LABEL_MAX_WIDTH: f32 = 192.0;
    /// A leading file icon (`size-3.5`) or spinner (`size-3`).
    pub const ICON: f32 = 14.0;
    pub const SPINNER: f32 = 12.0;
    /// Steer, edit and remove: `size-4 rounded` ghost buttons holding a `size-3` icon.
    pub const ACTION: f32 = 16.0;
    pub const ACTION_RADIUS: f32 = 4.0;
    pub const ACTION_ICON: f32 = 12.0;
    /// Chrome's dash for a 1 px `border-dashed`.
    pub const DASH: f32 = 3.0;
    pub const DASH_GAP: f32 = 3.0;
}

/// The tinted icon tile notices lead with: `size-6 rounded-lg [corner-shape:squircle]`.
pub mod badge {
    pub const SIZE: f32 = 24.0;
    pub const RADIUS: f32 = 8.0;
    pub const ICON: f32 = 14.0;
    pub const GAP: f32 = 8.0;
    /// The tile is the tone at 12 %.
    pub const TINT: f32 = 0.12;
}

/// A notice's tone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tone {
    Info,
    Warning,
}

/// The notice colours: the examples fall back to their own OKLCH values because
/// the docs page defines no `--info` or `--warning`.
pub fn tone_color(theme: &Theme, tone: Tone) -> Color {
    match (tone, theme.is_dark()) {
        (Tone::Info, false) => Color::oklch(0.55, 0.13, 250.0),
        (Tone::Info, true) => Color::oklch(0.7, 0.12, 250.0),
        (Tone::Warning, false) => Color::oklch(0.57, 0.113, 55.0),
        (Tone::Warning, true) => Color::oklch(0.7, 0.125, 55.0),
    }
}

/// Every colour the composer paints for one theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComposerSkin {
    pub card: Color,
    pub card_border: Color,
    pub card_shadow: &'static [Shadow],
    /// `bg-muted/50` under the card; `bg-muted` around the pill.
    pub tray: Color,
    pub pill_tray: Color,
    pub tray_border: Color,
    /// The pill: `bg-background border-border/70`.
    pub pill: Color,
    pub pill_border: Color,
    pub text: Color,
    pub muted: Color,
    /// Ghost hover: `hover:bg-muted` (`dark:hover:bg-muted/50`) with `hover:text-foreground`.
    pub hover_fill: Color,
    pub hover_text: Color,
    pub chip: Color,
    pub chip_border: Color,
    pub chip_pending_border: Color,
    /// `hover:bg-accent` (dark keeps the ghost's `muted/50`).
    pub chip_action_hover: Color,
}

impl ComposerSkin {
    pub fn new(theme: &Theme) -> Self {
        let dark = theme.is_dark();
        let hover_fill = if dark {
            theme.muted.fade(0.5)
        } else {
            theme.muted
        };
        Self {
            card: theme.table_card,
            card_border: theme.table_card_border,
            card_shadow: theme.table_card_shadow,
            tray: theme.muted.fade(0.5),
            pill_tray: theme.muted,
            tray_border: theme.table_card_border,
            pill: theme.background,
            pill_border: theme.border.fade(0.7),
            text: theme.foreground,
            muted: theme.muted_foreground,
            hover_fill,
            hover_text: theme.foreground,
            chip: theme.background,
            chip_border: theme.foreground.fade(0.25),
            chip_pending_border: theme.foreground.fade(0.15),
            chip_action_hover: if dark { hover_fill } else { theme.accent },
        }
    }
}

/// What the send button shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SendState {
    /// Violet and pressable.
    Ready,
    /// Nothing to send: the dark Depth button with a dimmed glyph, no hover.
    Inert,
    /// A spinner; still pressable unless also disabled, then dimmed.
    Busy { disabled: bool },
}

impl SendState {
    /// `ComposerSend`'s `busy` and `disabled` props.
    pub fn new(busy: bool, disabled: bool) -> Self {
        match (busy, disabled) {
            (true, disabled) => SendState::Busy { disabled },
            (false, true) => SendState::Inert,
            (false, false) => SendState::Ready,
        }
    }

    pub fn is_pressable(self) -> bool {
        matches!(self, SendState::Ready | SendState::Busy { disabled: false })
    }

    /// Only a busy, disabled send dims: `data-disabled:not-aria-busy:opacity-100`.
    pub fn opacity(self) -> f32 {
        match self {
            SendState::Busy { disabled: true } => crate::components::button::DISABLED_OPACITY,
            _ => 1.0,
        }
    }

    /// The button's paint; an inert send drops its hover and dims its glyph to
    /// `text-muted-foreground/70`.
    pub fn skin(self, theme: &Theme) -> ButtonSkin {
        match self {
            SendState::Inert => ButtonSkin {
                text: theme.muted_foreground.fade(0.7),
                hover: ButtonHover::Brightness(1.0),
                ..ButtonSkin::new(theme, ButtonVariant::Secondary)
            },
            _ => ButtonSkin::new(theme, ButtonVariant::Primary),
        }
    }

    /// The card's squircle, or a circle inside the pill.
    pub fn radius(shape: ComposerShape, size: ComposerSize) -> f32 {
        match shape {
            ComposerShape::Card => DEPTH_RADIUS,
            ComposerShape::Flat => send::size(size) / 2.0,
        }
    }
}

/// The tray's `height` and `opacity` as a nudge comes and goes, and the pill's tray
/// (`background-color, padding, border-radius`) on the same curve.
pub const NUDGE: Tween = Tween::new(Duration::from_millis(300), ease::EMPHASIZED_IN_OUT);

/// Hover colours on the toolbar, chips and send: `transition-all` at 150 ms.
pub const HOVER: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);

/// `animate-spin`: one turn a second.
pub const SPIN: Loop = Loop::new(Duration::from_secs(1), CubicBezier::new(0.0, 0.0, 1.0, 1.0));

/// The pill tray between no nudge (`0`) and a nudge (`1`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PillTray {
    pub padding: f32,
    pub top_radius: f32,
    pub bottom_radius: f32,
    /// How much of `bg-muted` shows.
    pub fill: f32,
}

impl PillTray {
    pub fn at(size: ComposerSize, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let radius = pill::tray_radius(size);
        Self {
            padding: pill::TRAY_PADDING * t,
            top_radius: crate::motion::lerp(radius, pill::TRAY_TOP_RADIUS, t),
            bottom_radius: radius,
            fill: t,
        }
    }
}

/// The send tooltip (Notra's tooltip): a dark Depth chip above the button with an arrow.
pub mod tooltip {
    use std::time::Duration;

    use crate::motion::{Tween, ease};
    use crate::theme::{Color, Shadow, Theme};

    /// Base UI's hover delay before a tooltip opens.
    pub const DELAY: Duration = Duration::from_millis(600);
    /// `transition-[opacity,scale,translate] duration-150 ease-out` on the way in…
    pub const ENTER: Tween = Tween::new(Duration::from_millis(150), ease::OUT);
    /// …and `data-ending-style:duration-100` on the way out.
    pub const EXIT: Tween = Tween::new(Duration::from_millis(100), ease::OUT);
    /// Starting and ending style: `scale-[0.97] opacity-0`, 2 px towards the trigger.
    pub const FROM_SCALE: f32 = 0.97;
    pub const FROM_OFFSET: f32 = 2.0;
    /// `sideOffset={4}`.
    pub const SIDE_OFFSET: f32 = 4.0;
    pub const PADDING_X: f32 = 10.0;
    pub const PADDING_Y: f32 = 4.0;
    pub const RADIUS: f32 = 10.0;
    pub const FONT_SIZE: f32 = 12.0;
    pub const LINE_HEIGHT: f32 = 16.0;
    /// The arrow: a `size-2.5 rotate-45 rounded-[2px]` square behind the chip, its centre
    /// 2 px inside the chip's edge.
    pub const ARROW: f32 = 10.0;
    pub const ARROW_RADIUS: f32 = 2.0;
    pub const ARROW_INSET: f32 = 2.0;

    /// The tooltip's paint for one theme.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct TooltipSkin {
        /// `bg-linear-to-b from-[color-mix(foreground, background 10%)] to-foreground`.
        pub top: Color,
        pub bottom: Color,
        pub border: Color,
        pub text: Color,
        pub arrow: Color,
        pub shadows: [Shadow; 3],
    }

    impl TooltipSkin {
        pub fn new(theme: &Theme) -> Self {
            Self {
                top: theme.foreground.mix(theme.background, 0.1),
                bottom: theme.foreground,
                border: theme.foreground.fade(0.6),
                text: theme.background,
                arrow: theme.foreground,
                shadows: [
                    Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffff38)),
                    Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x00000029)),
                    Shadow::drop(6.0, 16.0, -4.0, Color::rgba(0x0000003d)),
                ],
            }
        }
    }
}

/// A queued message waiting for the reply in progress.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueuedMessage {
    pub id: u64,
    pub text: String,
}

/// The queue the composer shows as chips: send now (steer), edit back into the
/// input, or drop.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Queue {
    messages: Vec<QueuedMessage>,
    next_id: u64,
}

impl Queue {
    pub fn new<'a>(texts: impl IntoIterator<Item = &'a str>) -> Self {
        let mut queue = Self::default();
        for text in texts {
            queue.push(text);
        }
        queue
    }

    pub fn messages(&self) -> &[QueuedMessage] {
        &self.messages
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Queues `text`, trimmed; blank text is ignored. Returns the new message's id.
    pub fn push(&mut self, text: &str) -> Option<u64> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.messages.push(QueuedMessage {
            id,
            text: text.to_owned(),
        });
        Some(id)
    }

    /// Takes a message out (remove, steer, or edit back into the input).
    pub fn take(&mut self, id: u64) -> Option<QueuedMessage> {
        let index = self.messages.iter().position(|m| m.id == id)?;
        Some(self.messages.remove(index))
    }
}

/// The copy and data the docs examples use.
pub mod demo {
    pub const PLACEHOLDER: &str = "Send a message...";
    pub const QUEUE_PLACEHOLDER: &str = "Queue a message...";
    pub const QUEUE_ANOTHER_PLACEHOLDER: &str = "Queue another message...";
    pub const REPLY_PLACEHOLDER: &str = "Write a reply...";
    pub const QUEUE: [&str; 2] = [
        "Add a comparison table for the pricing page",
        "Keep the intro under 80 words",
    ];
    pub const QUEUE_FILE: &str = "linear-sync-changelog.pdf";
    pub const READY_DRAFT: &str = "Draft release notes for the October launch";
    pub const QUEUED_DRAFT: &str = "Add a comparison table too";
    pub const WAITING_DRAFT: &str = "Now shorten it to a tweet";
    pub const UPLOAD_DRAFT: &str = "Summarize these files";
    pub const NOTICE_DRAFT: &str = "Write a changelog entry";
    pub const UPLOADED_FILE: &str = "pricing-notes.pdf";
    pub const UPLOADING_FILE: &str = "launch-screenshots.zip";
    pub const NOTICE: &str = "2 chat messages left this month";
    pub const LIMIT: &str = "You used all chat messages";
    pub const REPLY_TITLE: &str = "Replying to Dominik";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_states_follow_the_props() {
        assert_eq!(SendState::new(false, false), SendState::Ready);
        assert_eq!(SendState::new(false, true), SendState::Inert);
        assert!(SendState::new(true, false).is_pressable());
        assert!(!SendState::Inert.is_pressable());
        assert_eq!(SendState::new(true, true).opacity(), 0.5);
        assert_eq!(SendState::Inert.opacity(), 1.0);
    }

    #[test]
    fn inert_send_is_the_dark_button_with_a_dim_glyph() {
        let theme = Theme::light();
        let skin = SendState::Inert.skin(&theme);
        assert_eq!(skin.fill, theme.secondary);
        assert!((skin.text.a - 0.7).abs() < 1e-6);
        assert_eq!(skin.hovered().fill, skin.fill);
    }

    #[test]
    fn pill_send_is_round() {
        assert_eq!(
            SendState::radius(ComposerShape::Flat, ComposerSize::Sm),
            14.0
        );
        assert_eq!(
            SendState::radius(ComposerShape::Card, ComposerSize::Default),
            DEPTH_RADIUS
        );
    }

    #[test]
    fn pill_tray_eases_its_top_corners() {
        let open = PillTray::at(ComposerSize::Default, 1.0);
        assert_eq!((open.padding, open.top_radius), (4.0, 18.0));
        let closed = PillTray::at(ComposerSize::Default, 0.0);
        assert_eq!((closed.padding, closed.top_radius), (0.0, 29.0));
    }

    #[test]
    fn queue_takes_messages_by_id() {
        let mut q = Queue::new(demo::QUEUE);
        assert_eq!(q.push("   "), None);
        let id = q.push(" Shorter please ");
        assert_eq!(q.messages().len(), 3);
        assert_eq!(
            q.take(id.unwrap_or(0)).map(|m| m.text),
            Some("Shorter please".into())
        );
        assert_eq!(q.take(99), None);
    }

    #[test]
    fn card_input_grows_from_one_to_eight_lines() {
        assert_eq!(CARD_INPUT.content_height(1), 32.0);
        assert_eq!(CARD_INPUT.content_height(3), 72.0);
        assert_eq!(CARD_INPUT.content_height(30), 184.0);
    }
}
