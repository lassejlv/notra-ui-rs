//! The Notra sidebar (`notra-sidebar`): an inset or floating panel that collapses
//! to an icon rail, a rail that resizes it by drag or keyboard, menu rows with
//! collapsible sub-rows, and the timings both renderers share.

use std::time::Duration;

use crate::motion::{Tween, ease};
use crate::theme::{Color, Shadow, Theme};

/// The expanded width (`SIDEBAR_WIDTH`).
pub const WIDTH: f32 = 256.0;
pub const MIN_WIDTH: f32 = 240.0;
pub const MAX_WIDTH: f32 = 400.0;
/// The icon rail (`--sidebar-width-icon: 3rem`).
pub const ICON_WIDTH: f32 = 48.0;
/// One arrow-key resize step.
pub const RESIZE_STEP: f32 = 8.0;
/// A rail drag shorter than this is a click.
pub const DRAG_SLOP: f32 = 4.0;
/// How far past the minimum a drag must go to collapse, or past the rail to reopen.
pub const SNAP_DISTANCE: f32 = 40.0;

/// `--sidebar-duration: 300ms` with `--sidebar-ease: cubic-bezier(0.22, 1, 0.36, 1)`:
/// the width, the inset's margin, the header reflow and the sub-row fold.
pub const COLLAPSE: Tween = Tween::new(Duration::from_millis(300), ease::PROGRESS);
/// Labels and trailing parts of a row fade (`duration-150 ease-(--sidebar-ease)`).
pub const LABEL_FADE: Tween = Tween::new(Duration::from_millis(150), ease::PROGRESS);
/// Collapsed labels drift right as they fade (`translate-x-0.5`).
pub const LABEL_DRIFT: f32 = 2.0;
/// Sub-rows open and close (`duration-200 ease-(--sidebar-ease)`).
pub const SUB_PANEL: Tween = Tween::new(Duration::from_millis(200), ease::PROGRESS);
/// Tailwind's default 150 ms: the rail line, the search row height, the lg row padding.
pub const QUICK: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);

/// Where the panel sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SidebarVariant {
    /// Flush with the window edge, a border on its inner side.
    #[default]
    Sidebar,
    /// A rounded, ringed card with a gutter around it.
    Floating,
    /// Flush panel beside a rounded inset holding the page.
    Inset,
}

impl SidebarVariant {
    /// The `p-2` gutter of floating and inset panels.
    pub fn gutter(self) -> f32 {
        match self {
            SidebarVariant::Sidebar => 0.0,
            SidebarVariant::Floating | SidebarVariant::Inset => 8.0,
        }
    }

    /// The panel's outer width on the icon rail: `3rem`, or `3rem + 1rem + 2px` with a gutter.
    pub fn rail_width(self) -> f32 {
        match self {
            SidebarVariant::Sidebar => ICON_WIDTH,
            SidebarVariant::Floating | SidebarVariant::Inset => ICON_WIDTH + 16.0 + 2.0,
        }
    }

    /// Width of a row's content box on the rail (`--sidebar-rail-content`).
    pub fn rail_content(self) -> f32 {
        match self {
            SidebarVariant::Sidebar => ICON_WIDTH - 16.0,
            SidebarVariant::Floating | SidebarVariant::Inset => ICON_WIDTH - 16.0 + 2.0,
        }
    }
}

/// What a collapsed sidebar becomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Collapsible {
    /// Slides fully off screen.
    #[default]
    Offcanvas,
    /// Shrinks to the icon rail.
    Icon,
    /// Never collapses.
    None,
}

/// Open state and width, owned by the app (`SidebarProvider`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SidebarState {
    pub open: bool,
    pub width: f32,
    pub default_width: f32,
    /// A rail drag is resizing: width changes land without animating.
    pub resizing: bool,
    drag: Option<RailDrag>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RailDrag {
    start_x: f32,
    start_width: f32,
}

/// A key the focused rail handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RailKey {
    /// Arrow away from the sidebar's edge (right for a left sidebar).
    Grow,
    Shrink,
    Home,
    End,
    /// Enter or Space.
    Toggle,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self::new(true)
    }
}

impl SidebarState {
    pub fn new(open: bool) -> Self {
        Self {
            open,
            width: WIDTH,
            default_width: WIDTH,
            resizing: false,
            drag: None,
        }
    }

    pub fn collapsed(&self) -> bool {
        !self.open
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Sets the expanded width, rounded and clamped to the allowed range.
    pub fn set_width(&mut self, width: f32) {
        self.width = width.round().clamp(MIN_WIDTH, MAX_WIDTH);
    }

    /// The rail was pressed at window `x`.
    pub fn rail_press(&mut self, x: f32) {
        self.drag = Some(RailDrag {
            start_x: x,
            start_width: self.width,
        });
    }

    /// The pointer moved to window `x` while the rail is held: resizes, collapses
    /// past the minimum, or reopens a collapsed sidebar dragged far enough out.
    pub fn rail_move(&mut self, x: f32) {
        let Some(drag) = self.drag else {
            return;
        };
        let delta = x - drag.start_x;
        if delta.abs() < DRAG_SLOP {
            return;
        }
        if self.collapsed() {
            if delta >= SNAP_DISTANCE {
                self.rail_release();
                self.open = true;
            }
            return;
        }
        self.resizing = true;
        let dragged = drag.start_width + delta;
        self.set_width(dragged);
        if dragged < MIN_WIDTH - SNAP_DISTANCE {
            self.rail_release();
            self.open = false;
        }
    }

    pub fn rail_release(&mut self) {
        self.drag = None;
        self.resizing = false;
    }

    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// A double click restores the default width.
    pub fn rail_reset(&mut self) {
        self.open = true;
        self.width = self.default_width;
    }

    /// Returns whether the key was handled.
    pub fn rail_key(&mut self, key: RailKey) -> bool {
        let next = match key {
            RailKey::Toggle => {
                self.toggle();
                return true;
            }
            RailKey::Shrink if self.collapsed() => return false,
            RailKey::Shrink if self.width <= MIN_WIDTH => {
                self.open = false;
                return true;
            }
            RailKey::Shrink => self.width - RESIZE_STEP,
            RailKey::Grow if self.collapsed() => self.width,
            RailKey::Grow => self.width + RESIZE_STEP,
            RailKey::Home => MIN_WIDTH,
            RailKey::End => MAX_WIDTH,
        };
        self.open = true;
        self.set_width(next);
        true
    }

    /// The panel's outer width for `variant` when collapsed to `collapsible`.
    pub fn outer_width(&self, variant: SidebarVariant, collapsible: Collapsible) -> f32 {
        match (self.open, collapsible) {
            (true, _) | (false, Collapsible::None) => self.width,
            (false, Collapsible::Icon) => variant.rail_width(),
            (false, Collapsible::Offcanvas) => 0.0,
        }
    }

    /// `aria-valuetext` of the rail.
    pub fn rail_value_text(&self) -> String {
        if self.collapsed() {
            "Collapsed".to_owned()
        } else {
            format!("{} pixels", self.width)
        }
    }
}

/// Row sizes (`sidebarMenuButtonVariants`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum MenuButtonSize {
    Sm,
    #[default]
    Default,
    Lg,
}

impl MenuButtonSize {
    pub fn height(self) -> f32 {
        match self {
            MenuButtonSize::Sm => 28.0,
            MenuButtonSize::Default => 32.0,
            MenuButtonSize::Lg => 48.0,
        }
    }

    pub fn font_size(self) -> f32 {
        match self {
            MenuButtonSize::Sm => 12.0,
            MenuButtonSize::Default | MenuButtonSize::Lg => 14.0,
        }
    }
}

/// Box metrics of the sidebar's parts, in logical pixels.
pub mod metrics {
    /// Header, footer and group padding (`p-2`).
    pub const SECTION_PADDING: f32 = 8.0;
    /// Gap between rows of a menu (`gap-1`).
    pub const MENU_GAP: f32 = 4.0;
    /// Row padding, gap and radius (`p-2 gap-2 rounded-md`).
    pub const ROW_PADDING: f32 = 8.0;
    pub const ROW_GAP: f32 = 8.0;
    pub const ROW_RADIUS: f32 = 6.0;
    pub const ICON: f32 = 16.0;
    pub const FONT_SIZE: f32 = 14.0;
    pub const LINE_HEIGHT: f32 = 20.0;
    /// Group labels (`h-8 px-2 text-xs font-medium`).
    pub const GROUP_LABEL_HEIGHT: f32 = 32.0;
    /// Sub-row list (`mx-3.5 px-2.5 py-0.5 gap-1 border-l`).
    pub const SUB_MARGIN_X: f32 = 14.0;
    pub const SUB_PADDING_X: f32 = 10.0;
    pub const SUB_PADDING_Y: f32 = 2.0;
    /// Sub-rows (`h-7 px-2`).
    pub const SUB_ROW_HEIGHT: f32 = 28.0;
    pub const SUB_ROW_PADDING_X: f32 = 8.0;
    /// The rail's hit area (`w-4`) and line (`w-[2px]`, `inset-y-2`).
    pub const RAIL_HIT: f32 = 16.0;
    pub const RAIL_LINE: f32 = 2.0;
    pub const RAIL_INSET_Y: f32 = 8.0;
    /// Inset page (`m-2 rounded-xl`).
    pub const INSET_MARGIN: f32 = 8.0;
    pub const INSET_RADIUS: f32 = 12.0;
    /// Floating panel (`rounded-xl`).
    pub const FLOATING_RADIUS: f32 = 12.0;
    /// The trigger (`size="icon-sm"`, a 16 px glyph).
    pub const TRIGGER: f32 = 28.0;
    pub const TRIGGER_RADIUS: f32 = 6.0;
}

/// The sidebar's colours for one theme.
#[derive(Clone, Debug, PartialEq)]
pub struct SidebarSkin {
    pub background: Color,
    pub foreground: Color,
    /// Row hover and active fill.
    pub accent: Color,
    pub accent_foreground: Color,
    pub border: Color,
    pub ring: Color,
    pub muted_foreground: Color,
    /// Bold text of an active group row (`text-foreground`).
    pub strong: Color,
    /// The page beside an inset sidebar.
    pub inset: Color,
    pub inset_shadow: Vec<Shadow>,
    /// The trigger glyph (`text-foreground/70`, `hover:text-foreground`).
    pub trigger: Color,
    pub trigger_hover: Color,
    /// Floating panels: `shadow-sm` and a ring.
    pub floating_shadow: Vec<Shadow>,
}

impl SidebarSkin {
    pub fn new(theme: &Theme) -> Self {
        let dark = theme.is_dark();
        Self {
            background: theme.sidebar,
            foreground: theme.sidebar_foreground,
            accent: theme.sidebar_accent,
            accent_foreground: theme.sidebar_accent_foreground,
            border: theme.sidebar_border,
            ring: theme.sidebar_ring,
            muted_foreground: theme.muted_foreground,
            strong: theme.foreground,
            inset: theme.background,
            inset_shadow: if dark {
                Vec::new()
            } else {
                vec![Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x0000_000a))]
            },
            trigger: theme.foreground.fade(0.7),
            trigger_hover: theme.foreground,
            floating_shadow: vec![
                Shadow::drop(1.0, 3.0, 0.0, Color::rgba(0x0000_001a)),
                Shadow::drop(1.0, 2.0, -1.0, Color::rgba(0x0000_001a)),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragging_resizes_within_bounds() {
        let mut s = SidebarState::default();
        s.rail_press(256.0);
        s.rail_move(258.0);
        assert!(!s.resizing, "inside the slop");
        s.rail_move(300.0);
        assert!(s.resizing);
        assert_eq!(s.width, 300.0);
        s.rail_move(900.0);
        assert_eq!(s.width, MAX_WIDTH);
        s.rail_release();
        assert!(!s.resizing && s.open);
    }

    #[test]
    fn dragging_far_past_the_minimum_collapses() {
        let mut s = SidebarState::default();
        s.rail_press(256.0);
        s.rail_move(256.0 - (WIDTH - MIN_WIDTH) - SNAP_DISTANCE - 1.0);
        assert!(s.collapsed());
        assert!(!s.is_dragging());
        assert_eq!(s.width, MIN_WIDTH);
    }

    #[test]
    fn dragging_a_collapsed_rail_out_reopens() {
        let mut s = SidebarState::new(false);
        s.rail_press(66.0);
        s.rail_move(66.0 + SNAP_DISTANCE - 1.0);
        assert!(s.collapsed());
        s.rail_move(66.0 + SNAP_DISTANCE);
        assert!(s.open);
    }

    #[test]
    fn keys_resize_and_collapse_at_the_minimum() {
        let mut s = SidebarState::default();
        assert!(s.rail_key(RailKey::Home));
        assert_eq!(s.width, MIN_WIDTH);
        assert!(s.rail_key(RailKey::Shrink));
        assert!(s.collapsed());
        assert!(!s.rail_key(RailKey::Shrink));
        assert!(s.rail_key(RailKey::Grow));
        assert!(s.open);
        assert_eq!(s.width, MIN_WIDTH);
        s.rail_key(RailKey::End);
        assert_eq!(s.width, MAX_WIDTH);
    }

    #[test]
    fn inset_rail_is_three_rem_plus_its_gutter() {
        let s = SidebarState::new(false);
        assert_eq!(
            s.outer_width(SidebarVariant::Inset, Collapsible::Icon),
            66.0
        );
        assert_eq!(SidebarVariant::Inset.rail_content(), 34.0);
        assert_eq!(s.rail_value_text(), "Collapsed");
    }
}
