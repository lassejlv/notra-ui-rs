//! Icon tabs: a segmented control whose active tab shows its icon, sliding in
//! beside the label, under an indicator that glides to the active tab.

use std::time::Duration;

use crate::motion::{Tween, ease};
use crate::theme::{Color, Shadow, Theme};

/// Box metrics measured on ui.usenotra.com, in logical pixels.
pub mod metrics {
    /// `h-8`.
    pub const LIST_HEIGHT: f32 = 32.0;
    /// `p-0.5` around the tabs.
    pub const LIST_PADDING: f32 = 2.0;
    /// `gap-0.5` between tabs.
    pub const LIST_GAP: f32 = 2.0;
    /// `rounded-lg`.
    pub const LIST_RADIUS: f32 = 8.0;
    /// `rounded-md`, shared by the tabs and the indicator.
    pub const TAB_RADIUS: f32 = 6.0;
    /// `px-2.5`.
    pub const TAB_PADDING_X: f32 = 10.0;
    /// `gap-1.5` between the icon slot and the label.
    pub const TAB_GAP: f32 = 6.0;
    pub const FONT_SIZE: f32 = 14.0;
    pub const LINE_HEIGHT: f32 = 20.0;
    pub const FONT_WEIGHT: f32 = 500.0;
    /// Width of the icon slot on the active tab (`w-4`).
    pub const ICON_SLOT: f32 = 16.0;
    /// The icon glyph (`size={15}` in the example).
    pub const ICON_SIZE: f32 = 15.0;
    /// End margin of the collapsed slot (`-me-1.5`), which cancels the tab gap.
    pub const ICON_TUCK: f32 = -6.0;
    /// Scale of the glyph while its slot is collapsed (`scale-50`).
    pub const ICON_HIDDEN_SCALE: f32 = 0.5;
    /// The focus outline (`outline-2`).
    pub const FOCUS_OUTLINE: f32 = 2.0;
}

/// The indicator's slide (`ICON_TABS_INDICATOR_MS`), eased by [`ease_out_quint`].
pub const INDICATOR_DURATION: Duration = Duration::from_millis(300);
/// Width, margin, opacity and scale of the slide-in icon.
pub const ICON_REVEAL: Tween = Tween::new(Duration::from_millis(200), ease::PROGRESS);
/// `transition-colors duration-200` on the tab label.
pub const COLOR: Tween = Tween::new(Duration::from_millis(200), ease::IN_OUT);

/// `easeOutQuint` from the registry: `1 - (1 - t)^5`.
pub fn ease_out_quint(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(5)
}

/// Colours for one theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconTabsSkin {
    /// `bg-muted`.
    pub list: Color,
    /// `bg-background`, `dark:bg-foreground/10`.
    pub indicator: Color,
    pub indicator_shadow: Shadow,
    /// `text-muted-foreground/75`.
    pub text: Color,
    /// `hover:text-muted-foreground`.
    pub text_hover: Color,
    /// `data-active:text-foreground`.
    pub text_active: Color,
    /// `focus-visible:outline-ring`.
    pub ring: Color,
}

impl IconTabsSkin {
    pub fn new(theme: &Theme) -> Self {
        Self {
            list: theme.muted,
            indicator: if theme.is_dark() {
                theme.foreground.fade(0.1)
            } else {
                theme.background
            },
            indicator_shadow: Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x0000_0014)),
            text: theme.muted_foreground.fade(0.75),
            text_hover: theme.muted_foreground,
            text_active: theme.foreground,
            ring: theme.ring,
        }
    }
}

/// Which tab a key moves to, as Base UI's tab list does with `loopFocus`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabKey {
    Next,
    Previous,
    First,
    Last,
}

impl TabKey {
    /// The tab `self` lands on from `index` among `count` tabs.
    pub fn apply(self, index: usize, count: usize) -> usize {
        if count == 0 {
            return 0;
        }
        match self {
            TabKey::Next => (index + 1) % count,
            TabKey::Previous => (index + count - 1) % count,
            TabKey::First => 0,
            TabKey::Last => count - 1,
        }
    }
}

/// The indicator's horizontal box, `[left, width]`, gliding from where it is
/// shown to the active tab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IndicatorSlide {
    from: [f32; 2],
    to: [f32; 2],
    elapsed_at: Option<f64>,
}

impl IndicatorSlide {
    /// An indicator resting at `target`.
    pub fn new(target: [f32; 2]) -> Self {
        Self {
            from: target,
            to: target,
            elapsed_at: None,
        }
    }

    pub fn target(&self) -> [f32; 2] {
        self.to
    }

    /// Slides toward `target` from the box shown at `now` (seconds).
    pub fn slide_to(&mut self, target: [f32; 2], now: f64) {
        if target == self.to {
            return;
        }
        self.from = self.value(now);
        self.to = target;
        self.elapsed_at = Some(now);
    }

    /// Moves without animating: a resize, or reduced motion.
    pub fn snap(&mut self, target: [f32; 2]) {
        *self = Self::new(target);
    }

    pub fn value(&self, now: f64) -> [f32; 2] {
        let Some(start) = self.elapsed_at else {
            return self.to;
        };
        let t = ((now - start) / INDICATOR_DURATION.as_secs_f64()) as f32;
        let p = ease_out_quint(t);
        [
            self.from[0] + (self.to[0] - self.from[0]) * p,
            self.from[1] + (self.to[1] - self.from[1]) * p,
        ]
    }

    pub fn is_animating(&self, now: f64) -> bool {
        self.elapsed_at
            .is_some_and(|start| now - start < INDICATOR_DURATION.as_secs_f64())
    }
}

/// One tab of the demo: its label and the icon that slides in when active.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DemoTab {
    pub value: &'static str,
    pub label: &'static str,
    pub icon: crate::Icon,
}

/// The registry example: webhook views in a `w-64` list.
pub mod demo {
    use super::DemoTab;
    use crate::Icon;

    pub const TABS: [DemoTab; 2] = [
        DemoTab {
            value: "deliveries",
            label: "Deliveries",
            icon: Icon::Send,
        },
        DemoTab {
            value: "endpoints",
            label: "Endpoints",
            icon: Icon::Link,
        },
    ];
    /// `className="w-64"` on the list.
    pub const LIST_WIDTH: f32 = 256.0;
    pub const LABEL: &str = "Webhook views";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quint_hits_its_ends_and_front_loads() {
        assert_eq!(ease_out_quint(0.0), 0.0);
        assert_eq!(ease_out_quint(1.0), 1.0);
        assert!(ease_out_quint(0.25) > 0.75);
    }

    #[test]
    fn slide_retargets_from_the_shown_box() {
        let mut s = IndicatorSlide::new([0.0, 100.0]);
        s.slide_to([100.0, 120.0], 0.0);
        let mid = s.value(0.15);
        assert!(mid[0] > 50.0 && mid[0] < 100.0);
        s.slide_to([0.0, 100.0], 0.15);
        assert_eq!(s.value(0.15), mid);
        assert_eq!(s.value(1.0), [0.0, 100.0]);
        assert!(!s.is_animating(1.0));
    }

    #[test]
    fn keys_loop_around_the_list() {
        assert_eq!(TabKey::Next.apply(1, 2), 0);
        assert_eq!(TabKey::Previous.apply(0, 2), 1);
        assert_eq!(TabKey::Last.apply(0, 3), 2);
    }

    #[test]
    fn dark_indicator_is_a_translucent_foreground() {
        let skin = IconTabsSkin::new(&Theme::dark());
        assert!((skin.indicator.a - 0.1).abs() < 1e-6);
    }
}
