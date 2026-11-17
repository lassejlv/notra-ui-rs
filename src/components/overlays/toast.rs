//! Notra's toasts (`notra-sonner`): Sonner's stack, timing and motion with Notra's Depth
//! toast surface and tinted status icons.
//!
//! Time is seconds on any monotonic clock the renderer keeps (`f64`), so the queue runs the
//! same under GPUI's `Instant` and Rune's frame clock.

use std::time::Duration;

use crate::icons::Icon;
use crate::motion::{CubicBezier, Tween, ease};
use crate::theme::{Color, Shadow, Theme};

/// Measured box metrics, in logical pixels.
pub mod metrics {
    /// Sonner's `--width`.
    pub const WIDTH: f32 = 356.0;
    /// `p-3`.
    pub const PADDING: f32 = 12.0;
    /// `gap-2.5` between the icon and the text.
    pub const GAP: f32 = 10.0;
    /// `rounded-[0.875rem]` squircle.
    pub const RADIUS: f32 = 14.0;
    /// The status icon's tile: `size-6 rounded-lg`, glyph `size-3.5`.
    pub const ICON_BOX: f32 = 24.0;
    pub const ICON_RADIUS: f32 = 8.0;
    pub const ICON: f32 = 14.0;
    /// `gap-0.5` between title and description.
    pub const TEXT_GAP: f32 = 2.0;
    /// `text-sm font-medium leading-snug`.
    pub const TITLE_FONT: f32 = 14.0;
    pub const TITLE_LINE: f32 = 19.25;
    /// `text-[0.8rem] leading-snug`.
    pub const DESCRIPTION_FONT: f32 = 12.8;
    pub const DESCRIPTION_LINE: f32 = 17.6;
    /// Notra's `gap={8}` between toasts.
    pub const STACK_GAP: f32 = 8.0;
    /// Sonner's default `offset` from the window edge.
    pub const OFFSET: f32 = 24.0;
    /// Each toast behind the front one shrinks this much more.
    pub const SCALE_STEP: f32 = 0.05;
    /// The close button: outline `icon-xs` resized to 20, round, `-top-2 -left-2`.
    pub const CLOSE: f32 = 20.0;
    pub const CLOSE_ICON: f32 = 10.0;
    pub const CLOSE_INSET: f32 = -8.0;
}

/// Sonner's `visibleToasts`.
pub const VISIBLE_TOASTS: usize = 3;

/// Sonner's default `duration`.
pub const DURATION: Duration = Duration::from_millis(4000);

/// Sonner waits this long after a toast starts leaving before unmounting it.
pub const TIME_BEFORE_UNMOUNT: Duration = Duration::from_millis(200);

/// `transition: transform 400ms, opacity 400ms, height 400ms` (CSS `ease`).
pub const MOVE: Tween = Tween::new(Duration::from_millis(400), ease::EASE);

/// A toast behind the front one leaving while collapsed: transform 500 ms, opacity 200 ms.
pub const LEAVE_BEHIND: Tween = Tween::new(Duration::from_millis(500), ease::EASE);
pub const LEAVE_BEHIND_FADE: Tween = Tween::new(Duration::from_millis(200), ease::EASE);

/// `animate-spin`: one turn a second, linear.
pub const SPIN: Duration = Duration::from_secs(1);
pub const SPIN_CURVE: CubicBezier = ease::LINEAR;

/// `toast()`, `toast.success()`, …
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ToastKind {
    #[default]
    Default,
    Success,
    Info,
    Warning,
    Error,
    Loading,
}

impl ToastKind {
    /// The Toaster's `icons` prop.
    pub fn icon(self) -> Option<Icon> {
        match self {
            ToastKind::Default => None,
            ToastKind::Success => Some(Icon::CircleCheck),
            ToastKind::Info => Some(Icon::Info),
            ToastKind::Warning => Some(Icon::TriangleAlert),
            ToastKind::Error => Some(Icon::OctagonX),
            ToastKind::Loading => Some(Icon::LoaderCircle),
        }
    }
}

/// One toast's content.
#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    /// Pushing a toast with an id already shown updates it in place (`toast.success(.., { id })`).
    pub id: String,
    pub kind: ToastKind,
    pub title: String,
    pub description: Option<String>,
    /// `action`: a primary `xs` button; its id comes back when pressed.
    pub action: Option<String>,
    /// `cancel`: a ghost `xs` button that dismisses the toast.
    pub cancel: Option<String>,
    /// `None` never expires (loading toasts).
    pub duration: Option<Duration>,
    /// `closeButton`.
    pub close_button: bool,
}

impl Toast {
    pub fn new(id: impl Into<String>, kind: ToastKind, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind,
            title: title.into(),
            description: None,
            action: None,
            cancel: None,
            duration: (kind != ToastKind::Loading).then_some(DURATION),
            close_button: false,
        }
    }

    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.action = Some(label.into());
        self
    }

    pub fn cancel(mut self, label: impl Into<String>) -> Self {
        self.cancel = Some(label.into());
        self
    }

    pub fn duration(mut self, duration: Option<Duration>) -> Self {
        self.duration = duration;
        self
    }

    pub fn close_button(mut self, on: bool) -> Self {
        self.close_button = on;
        self
    }

    /// The toast's height for one line of title and description: Sonner stacks toasts by
    /// their measured heights, which for single-line toasts this reproduces.
    pub fn height(&self) -> f32 {
        let text = metrics::TITLE_LINE
            + self
                .description
                .as_ref()
                .map_or(0.0, |_| metrics::TEXT_GAP + metrics::DESCRIPTION_LINE);
        let buttons = if self.action.is_some() || self.cancel.is_some() {
            crate::components::button::ButtonSize::Xs.metrics().height
        } else {
            0.0
        };
        let icon = if self.kind.icon().is_some() {
            metrics::ICON_BOX
        } else {
            0.0
        };
        2.0 + 2.0 * metrics::PADDING + text.max(icon).max(buttons)
    }
}

/// The resolved paint of toasts for one theme.
#[derive(Clone, Debug, PartialEq)]
pub struct ToastSkin {
    pub fill: Color,
    /// `bg-linear-to-b` stops, top to bottom.
    pub gradient: (Color, Color),
    pub border: Color,
    pub shadows: Vec<Shadow>,
    pub text: Color,
    pub description: Color,
    pub status: StatusColors,
}

/// The icon tile's fill and glyph colour for each kind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatusColors {
    pub neutral: (Color, Color),
    pub success: (Color, Color),
    pub info: (Color, Color),
    pub warning: (Color, Color),
    pub error: (Color, Color),
}

impl StatusColors {
    pub fn of(&self, kind: ToastKind) -> (Color, Color) {
        match kind {
            ToastKind::Default | ToastKind::Loading => self.neutral,
            ToastKind::Success => self.success,
            ToastKind::Info => self.info,
            ToastKind::Warning => self.warning,
            ToastKind::Error => self.error,
        }
    }
}

impl ToastSkin {
    pub fn new(theme: &Theme) -> Self {
        let tint = |c: Color| (c.fade(0.12), c);
        let status = StatusColors {
            neutral: (theme.muted, theme.muted_foreground),
            success: tint(theme.success),
            info: tint(theme.info),
            warning: tint(theme.warning),
            error: tint(theme.destructive),
        };
        if theme.is_dark() {
            Self {
                fill: theme.muted,
                gradient: (theme.input, theme.muted),
                border: theme.input,
                shadows: vec![
                    Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffff0f)),
                    Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x00000066)),
                    Shadow::drop(8.0, 24.0, -8.0, Color::rgba(0x00000099)),
                ],
                text: theme.popover_foreground,
                description: theme.muted_foreground,
                status,
            }
        } else {
            Self {
                fill: theme.background,
                gradient: (theme.background, theme.muted.fade(0.6)),
                border: theme.border,
                shadows: vec![
                    Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffffb3)),
                    Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x0000000f)),
                    Shadow::drop(8.0, 24.0, -8.0, Color::rgba(0x00000024)),
                ],
                text: theme.popover_foreground,
                description: theme.muted_foreground,
                status,
            }
        }
    }
}

/// Where a toast sits in the stack and how it looks there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    /// Distance its bottom edge is lifted above the stack's bottom.
    pub lift: f32,
    pub scale: f32,
    /// Its drawn height (collapsed toasts take the front one's).
    pub height: f32,
    /// `data-visible`: only the first [`VISIBLE_TOASTS`] show.
    pub opacity: f32,
    /// Its content's opacity: hidden behind the front toast while collapsed.
    pub content: f32,
}

/// Sonner's stack for toasts of `heights`, newest (front) first. Collapsed, each toast
/// behind is lifted `gap` more and scaled 5 % smaller at the front one's height; expanded
/// (hovered), they stack at their own heights `gap` apart.
pub fn stack(heights: &[f32], expanded: bool) -> Vec<Slot> {
    let front = heights.first().copied().unwrap_or(0.0);
    let mut offset = 0.0;
    heights
        .iter()
        .enumerate()
        .map(|(i, &h)| {
            let visible = if i < VISIBLE_TOASTS { 1.0 } else { 0.0 };
            let slot = if expanded {
                Slot {
                    lift: offset,
                    scale: 1.0,
                    height: h,
                    opacity: visible,
                    content: 1.0,
                }
            } else {
                Slot {
                    lift: i as f32 * metrics::STACK_GAP,
                    scale: 1.0 - i as f32 * metrics::SCALE_STEP,
                    height: if i == 0 { h } else { front },
                    opacity: visible,
                    content: if i == 0 { 1.0 } else { 0.0 },
                }
            };
            offset += h + metrics::STACK_GAP;
            slot
        })
        .collect()
}

/// A toast in the queue with its timing.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub toast: Toast,
    /// When it first showed, or was last updated in place.
    pub shown_at: f64,
    /// Time it has spent counting down (paused while the stack is hovered).
    pub elapsed: f64,
    /// When it started leaving.
    pub removed_at: Option<f64>,
}

impl Entry {
    pub fn is_removed(&self) -> bool {
        self.removed_at.is_some()
    }
}

/// The Toaster's queue: newest first, timers that pause while the stack is hovered, removal
/// that leaves the toast mounted for its exit.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToastQueue {
    entries: Vec<Entry>,
    /// The stack is hovered: it expands and timers pause.
    pub expanded: bool,
    last_tick: Option<f64>,
}

impl ToastQueue {
    /// Every toast still mounted, newest first, leaving ones included.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Toasts that aren't leaving, newest first: the ones the stack lays out.
    pub fn live(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|e| !e.is_removed())
    }

    /// Shows `toast` at `now`, or updates the toast with its id in place (restarting its
    /// timer, as Sonner does).
    pub fn push(&mut self, toast: Toast, now: f64) {
        if let Some(e) = self
            .entries
            .iter_mut()
            .find(|e| e.toast.id == toast.id && !e.is_removed())
        {
            e.toast = toast;
            e.elapsed = 0.0;
            return;
        }
        self.entries.insert(
            0,
            Entry {
                toast,
                shown_at: now,
                elapsed: 0.0,
                removed_at: None,
            },
        );
    }

    /// Starts `id` leaving.
    pub fn dismiss(&mut self, id: &str, now: f64) {
        for e in self.entries.iter_mut().filter(|e| e.toast.id == id) {
            e.removed_at.get_or_insert(now);
        }
    }

    /// Advances timers to `now`: expires toasts whose time ran out and unmounts those that
    /// finished leaving. Returns whether anything changed.
    pub fn tick(&mut self, now: f64) -> bool {
        let dt = self.last_tick.map_or(0.0, |t| (now - t).max(0.0));
        self.last_tick = Some(now);
        let mut changed = false;
        let paused = self.expanded;
        for e in self.entries.iter_mut().filter(|e| !e.is_removed()) {
            if !paused {
                e.elapsed += dt;
            }
            if let Some(d) = e.toast.duration
                && e.elapsed >= d.as_secs_f64()
            {
                e.removed_at = Some(now);
                changed = true;
            }
        }
        let leave = (MOVE.duration + TIME_BEFORE_UNMOUNT).as_secs_f64();
        let before = self.entries.len();
        self.entries
            .retain(|e| e.removed_at.is_none_or(|at| now - at < leave));
        changed || self.entries.len() != before
    }

    /// Seconds until the next toast expires or unmounts, for a timer; `None` when nothing is
    /// pending.
    pub fn next_deadline(&self, now: f64) -> Option<f64> {
        let leave = (MOVE.duration + TIME_BEFORE_UNMOUNT).as_secs_f64();
        self.entries
            .iter()
            .filter_map(|e| match (e.removed_at, e.toast.duration) {
                (Some(at), _) => Some(at + leave - now),
                (None, Some(d)) if !self.expanded => Some(d.as_secs_f64() - e.elapsed),
                _ => None,
            })
            .map(|s| s.max(0.0))
            .min_by(|a, b| a.total_cmp(b))
    }

    /// The live toasts' stack slots, newest first.
    pub fn slots(&self) -> Vec<Slot> {
        let heights: Vec<f32> = self.live().map(|e| e.toast.height()).collect();
        stack(&heights, self.expanded)
    }
}

/// The live `sonner-types` and `sonner-action` examples' toasts.
pub mod demo {
    use super::{Toast, ToastKind};

    pub fn default() -> Toast {
        Toast::new("default", ToastKind::Default, "Workspace synced")
    }
    pub fn success() -> Toast {
        Toast::new("success", ToastKind::Success, "Deployment complete")
    }
    pub fn info() -> Toast {
        Toast::new("info", ToastKind::Info, "New scan scheduled")
            .description("Runs every Monday at 09:00.")
    }
    pub fn warning() -> Toast {
        Toast::new("warning", ToastKind::Warning, "Seat limit almost reached")
            .description("9 of 10 seats are in use.")
    }
    pub fn error() -> Toast {
        Toast::new("error", ToastKind::Error, "Something went wrong")
    }
    pub fn loading() -> Toast {
        Toast::new("publish", ToastKind::Loading, "Publishing changelog...")
    }
    /// What the loading toast becomes after two seconds.
    pub fn published() -> Toast {
        Toast::new("publish", ToastKind::Success, "Changelog published")
    }
    pub fn with_action() -> Toast {
        Toast::new("trash", ToastKind::Default, "Post moved to trash").action("Undo")
    }
    pub fn restored() -> Toast {
        Toast::new("restored", ToastKind::Success, "Post restored")
    }
    pub fn with_cancel() -> Toast {
        Toast::new("role", ToastKind::Error, "Couldn't change role")
            .description("Your plan includes 3 team members.")
            .cancel("Dismiss")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heights_match_the_live_toasts() {
        assert!((demo::info().height() - 64.85).abs() < 0.01);
        assert_eq!(demo::default().height(), 45.25);
        assert_eq!(demo::error().height(), 50.0);
    }

    #[test]
    fn collapsed_stack_lifts_and_shrinks_the_older_toasts() {
        let s = stack(&[50.0, 64.85, 45.25, 50.0], false);
        assert_eq!(s[1].lift, 8.0);
        assert!((s[2].scale - 0.9).abs() < 1e-6);
        assert_eq!(s[2].height, 50.0);
        assert_eq!(s[1].content, 0.0);
        assert_eq!(s[3].opacity, 0.0);
    }

    #[test]
    fn expanded_stack_uses_each_height() {
        let s = stack(&[50.0, 64.85, 45.25], true);
        assert_eq!(s[1].lift, 58.0);
        assert!((s[2].lift - 130.85).abs() < 1e-3);
        assert!(s.iter().all(|s| s.scale == 1.0 && s.content == 1.0));
    }

    #[test]
    fn timers_pause_while_hovered_and_toasts_unmount_after_leaving() {
        let mut q = ToastQueue::default();
        q.push(demo::success(), 0.0);
        q.tick(0.0);
        q.tick(3.0);
        q.expanded = true;
        q.tick(10.0);
        assert!(!q.entries()[0].is_removed());
        q.expanded = false;
        q.tick(10.5);
        assert!(q.tick(11.1));
        assert!(q.entries()[0].is_removed());
        assert!(q.tick(11.8));
        assert!(q.entries().is_empty());
    }

    #[test]
    fn pushing_an_existing_id_updates_in_place() {
        let mut q = ToastQueue::default();
        q.push(demo::loading(), 0.0);
        q.push(demo::default(), 0.1);
        q.push(demo::published(), 2.0);
        assert_eq!(q.entries().len(), 2);
        assert_eq!(q.entries()[1].toast.kind, ToastKind::Success);
        assert_eq!(q.next_deadline(2.0), Some(4.0));
    }
}
