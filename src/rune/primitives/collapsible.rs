use ::rune::{El, col, icon, row};
use ::rune_motion::{Spring, SpringPair, Transition};

use crate::components::primitives::collapsible::{CHEVRON, PANEL};
use crate::icons::Icon;
use crate::rune::{ToRune, tween};
use crate::theme::Color;

type Press = Box<dyn FnOnce(El) -> El>;

/// A disclosure whose panel grows open and fades in:
/// `Collapsible::new(s.open).trigger(..).content(..).on_toggle(|s: &mut S| s.open ^= true)`.
pub struct Collapsible {
    open: bool,
    disabled: bool,
    trigger: Option<El>,
    content: Option<El>,
    press: Option<Press>,
}

impl Collapsible {
    pub fn new(open: bool) -> Self {
        Self {
            open,
            disabled: false,
            trigger: None,
            content: None,
            press: None,
        }
    }

    pub fn trigger(mut self, trigger: El) -> Self {
        self.trigger = Some(trigger);
        self
    }

    /// The panel's content, mounted while open.
    pub fn content(mut self, content: El) -> Self {
        self.content = Some(content);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_toggle<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(Box::new(move |e: El| e.on_press(f)));
        self
    }

    pub fn build(self) -> El {
        let mut trigger = row()
            .key("trigger")
            .role("button")
            .expanded(self.open)
            .disabled(self.disabled)
            .children(self.trigger);
        if !self.disabled
            && let Some(press) = self.press
        {
            trigger = press(trigger.focusable(true).cursor(::rune::Cursor::Pointer));
        }
        let spring = Spring::new(PANEL.duration.as_secs_f32(), 0.0);
        let panel = self.open.then(|| {
            col()
                .key("panel")
                .clip_rounded(0.0)
                .transition(
                    Transition::opacity()
                        .hold_layout()
                        .springs(SpringPair::new(spring, spring)),
                )
                .children(self.content)
        });
        col()
            .motion(spring)
            .clip_rounded(0.0)
            .child(trigger)
            .children(panel)
    }
}

/// A chevron that turns half a turn while its collapsible is open.
pub struct CollapsibleChevron {
    open: bool,
    icon: Icon,
    size: f32,
    color: Option<Color>,
}

impl CollapsibleChevron {
    pub fn new(open: bool) -> Self {
        Self {
            open,
            icon: Icon::ChevronDown,
            size: 16.0,
            color: None,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn build(self, ink: Color) -> El {
        icon(self.icon.rune(), self.size)
            .color(self.color.unwrap_or(ink).rune())
            .rotate(if self.open { std::f32::consts::PI } else { 0.0 })
            .rotate_motion(tween(CHEVRON))
    }
}
