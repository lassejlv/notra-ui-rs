//! Modal surfaces: the dialog and the sheet (shadcn's base-nova `Dialog` and `Sheet` on Base
//! UI's dialog, as Notra's blocks use them), their backdrop, and their motion.

use std::time::Duration;

use super::{PopupMotion, Side};
use crate::motion::{Tween, ease};
use crate::theme::{Color, Shadow, Theme};

/// Measured box metrics, in logical pixels.
pub mod metrics {
    /// `sm:max-w-sm`.
    pub const DIALOG_WIDTH: f32 = 384.0;
    /// `max-w-[calc(100%-2rem)]`, `max-h-[calc(100dvh-2rem)]`.
    pub const DIALOG_MARGIN: f32 = 16.0;
    pub const DIALOG_RADIUS: f32 = 12.0;
    pub const DIALOG_PADDING: f32 = 16.0;
    pub const DIALOG_GAP: f32 = 16.0;
    /// `DialogHeader`'s `gap-2`.
    pub const HEADER_GAP: f32 = 8.0;
    /// `DialogFooter`'s `gap-2` between buttons.
    pub const FOOTER_GAP: f32 = 8.0;
    /// The close button: ghost `icon-sm` at `top-2 right-2` (sheet: `top-3 right-3`).
    pub const CLOSE: f32 = 28.0;
    pub const CLOSE_ICON: f32 = 16.0;
    pub const DIALOG_CLOSE_INSET: f32 = 8.0;
    pub const SHEET_CLOSE_INSET: f32 = 12.0;
    /// `DialogTitle`: `text-base leading-none font-medium`.
    pub const TITLE_FONT: f32 = 16.0;
    pub const DIALOG_TITLE_LINE: f32 = 16.0;
    /// `SheetTitle`: `text-base font-medium` on a 24 px line.
    pub const SHEET_TITLE_LINE: f32 = 24.0;
    /// Descriptions: `text-sm text-muted-foreground`.
    pub const BODY_FONT: f32 = 14.0;
    pub const BODY_LINE: f32 = 20.0;
    /// `w-3/4 sm:max-w-sm` beside the window edge.
    pub const SHEET_WIDTH: f32 = 384.0;
    pub const SHEET_WIDTH_FRACTION: f32 = 0.75;
    /// `SheetHeader`/`SheetFooter`: `p-4`, header `gap-0.5`, footer `gap-2`.
    pub const SHEET_PADDING: f32 = 16.0;
    pub const SHEET_HEADER_GAP: f32 = 2.0;
    pub const SHEET_GAP: f32 = 16.0;
    /// `backdrop-blur-xs`.
    pub const BACKDROP_BLUR: f32 = 4.0;
}

/// `duration-100` `fade-in-0 zoom-in-95` / `fade-out-0 zoom-out-95` (tw-animate's keyframes on
/// CSS `ease`).
pub const DIALOG_MOTION: PopupMotion = PopupMotion {
    enter: Tween::new(Duration::from_millis(100), ease::EASE),
    exit: Tween::new(Duration::from_millis(100), ease::EASE),
    scale: 0.95,
    enter_travel: 0.0,
    exit_travel: 0.0,
};

/// The dialog backdrop's fade (`duration-100 fade-in-0`).
pub const DIALOG_BACKDROP: Tween = Tween::new(Duration::from_millis(100), ease::EASE);

/// The sheet: `transition duration-200 ease-in-out`, opacity and 2.5 rem of travel from its
/// edge, both ways.
pub const SHEET_MOTION: PopupMotion = PopupMotion {
    enter: Tween::new(Duration::from_millis(200), ease::IN_OUT),
    exit: Tween::new(Duration::from_millis(200), ease::IN_OUT),
    scale: 1.0,
    enter_travel: 40.0,
    exit_travel: 40.0,
};

/// The sheet backdrop's `transition-opacity duration-150`.
pub const SHEET_BACKDROP: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);

/// The resolved paint of dialogs and sheets for one theme.
#[derive(Clone, Debug, PartialEq)]
pub struct ModalSkin {
    /// `bg-black/10`.
    pub backdrop: Color,
    pub surface: Color,
    pub text: Color,
    pub description: Color,
    /// The dialog's `ring-1 ring-foreground/10`.
    pub ring: Color,
    /// The sheet's inner edge (`border-l` and friends).
    pub border: Color,
    /// `DialogFooter`'s `bg-muted/50` and `border-t`.
    pub footer: Color,
    pub footer_border: Color,
    /// The sheet's `shadow-lg`.
    pub sheet_shadows: Vec<Shadow>,
}

impl ModalSkin {
    pub fn new(theme: &Theme) -> Self {
        Self {
            backdrop: Color::BLACK.alpha(0.1),
            surface: theme.popover,
            text: theme.popover_foreground,
            description: theme.muted_foreground,
            ring: theme.foreground.fade(0.1),
            border: theme.border,
            footer: theme.muted.fade(0.5),
            footer_border: theme.border,
            sheet_shadows: vec![
                Shadow::drop(10.0, 15.0, -3.0, Color::BLACK.alpha(0.1)),
                Shadow::drop(4.0, 6.0, -4.0, Color::BLACK.alpha(0.1)),
            ],
        }
    }
}

/// Where a sheet slides in from (`side`), and its rectangle in a window of `size`.
pub fn sheet_rect(side: Side, window: (f32, f32), content_height: f32) -> super::Rect {
    let (w, h) = window;
    let width = (w * metrics::SHEET_WIDTH_FRACTION).min(metrics::SHEET_WIDTH);
    match side {
        Side::Right => super::Rect::new(w - width, 0.0, width, h),
        Side::Left => super::Rect::new(0.0, 0.0, width, h),
        Side::Top => super::Rect::new(0.0, 0.0, w, content_height.min(h)),
        Side::Bottom => {
            let ch = content_height.min(h);
            super::Rect::new(0.0, h - ch, w, ch)
        }
    }
}

/// The sheet's frame at `shown` (0 gone, 1 at rest): it travels in from `side`.
pub fn sheet_offset(side: Side, shown: f32) -> (f32, f32) {
    let travel = SHEET_MOTION.enter_travel * (1.0 - shown.clamp(0.0, 1.0));
    let (dx, dy) = side.direction();
    (dx * travel, dy * travel)
}

/// The dialog's width in a window `window_width` wide.
pub fn dialog_width(window_width: f32, max: f32) -> f32 {
    max.min(window_width - 2.0 * metrics::DIALOG_MARGIN)
        .max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sheets_hug_their_edge() {
        let r = sheet_rect(Side::Right, (1000.0, 700.0), 0.0);
        assert_eq!((r.x, r.w, r.h), (616.0, 384.0, 700.0));
        let narrow = sheet_rect(Side::Left, (400.0, 700.0), 0.0);
        assert_eq!(narrow.w, 300.0);
        let bottom = sheet_rect(Side::Bottom, (1000.0, 700.0), 200.0);
        assert_eq!(bottom.y, 500.0);
    }

    #[test]
    fn sheets_start_outside_by_two_and_a_half_rem() {
        assert_eq!(sheet_offset(Side::Right, 0.0), (40.0, 0.0));
        assert_eq!(sheet_offset(Side::Top, 0.0), (0.0, -40.0));
        assert_eq!(sheet_offset(Side::Left, 1.0), (0.0, 0.0));
    }

    #[test]
    fn dialogs_keep_a_margin_on_narrow_windows() {
        assert_eq!(dialog_width(1000.0, metrics::DIALOG_WIDTH), 384.0);
        assert_eq!(dialog_width(300.0, metrics::DIALOG_WIDTH), 268.0);
    }
}
