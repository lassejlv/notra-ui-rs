//! Notra's tooltip and the duotone tooltip for Rune.
//!
//! A tooltip is a trigger wired to a [`TooltipState`] the app owns ([`trigger`]) and a surface
//! ([`Tooltip::build`]) mounted in the overlay layer while the state is open on that trigger.
//! A shared state (Notra's `TooltipProvider`) mounts one surface for all its triggers: when the
//! pointer moves to a neighbour, the surface glides there over 200 ms while its content swaps.
//! Rune doesn't animate an anchored element when its anchor changes, so the surface stays
//! anchored to the trigger the tooltip opened on and slides by the difference between the two
//! triggers' rects, which the triggers report as they lay out.
//!
//! ```ignore
//! let bold = tooltip::trigger(button, "bold", "Bold", &s.tips, |s: &mut App| &mut s.tips);
//! if let Some(key) = s.tips.open_key() {
//!     layer.push(Tooltip::new("Bold").keys(&["⌘", "B"]).build(&theme, "toolbar-tip", &s.tips));
//! }
//! ```

use std::collections::HashMap;
use std::time::Instant;

use ::rune::{El, Key, col, el, row, text};
use ::rune_motion::{SpringPair, Transition, Vec2};

use super::{popup, spring};
use crate::components::overlays::tooltip::{
    GLIDE, MOTION, PLACEMENT, RESIZE, TooltipGroup, TooltipSkin, arrow_inset, duotone, kbd, metrics,
};
use crate::components::overlays::{Align, Placement, Side};
use crate::rune::{ToRune, sans, tween};
use crate::theme::Theme;

pub use crate::components::overlays::popover::HoverIntent;
pub use crate::components::overlays::tooltip::TooltipDelays;

/// A group of tooltips the app owns: the [`TooltipGroup`] plus what the shared popup needs to
/// glide (its triggers' rects and the trigger it opened on).
#[derive(Clone, Debug, Default)]
pub struct TooltipState {
    /// The open state, delays and warm window.
    pub group: TooltipGroup<String>,
    rects: HashMap<String, [f32; 4]>,
    origin: Option<String>,
}

impl TooltipState {
    pub fn new(delays: TooltipDelays) -> Self {
        Self {
            group: TooltipGroup::new(delays),
            ..Self::default()
        }
    }

    /// The trigger the popup is open on.
    pub fn open_key(&self) -> Option<&str> {
        self.group
            .active()
            .filter(|_| self.group.is_open())
            .map(String::as_str)
    }

    /// Opens on `key` at once, as if the pointer had rested on it (scenes, tests).
    pub fn show(&mut self, key: &str) {
        self.rest(key);
    }

    /// Opens on `key`. Returns whether it changed and whether the change may land at once: a
    /// popup still leaving (the pointer crossed the gap between two triggers) glides on to the
    /// new trigger instead.
    fn rest(&mut self, key: &str) -> (bool, bool) {
        let now = Instant::now();
        let was_open = self.group.is_open();
        let leaving = !was_open
            && self
                .group
                .closed_at()
                .is_some_and(|at| now.saturating_duration_since(at) < MOTION.exit.duration);
        let changed = self.group.rest(key.to_owned(), now);
        if !was_open && !leaving && self.group.is_open() {
            self.origin = Some(key.to_owned());
        }
        (changed, changed && !leaving && self.group.is_instant())
    }
}

/// The point of `rect` (`[x, y, w, h]`) a popup on `placement` lines up with.
fn edge_point(rect: [f32; 4], placement: Placement) -> (f32, f32) {
    let [x, y, w, h] = rect;
    let along = match placement.align {
        Align::Start => 0.0,
        Align::Center => 0.5,
        Align::End => 1.0,
    };
    match placement.side {
        Side::Top => (x + w * along, y),
        Side::Bottom => (x + w * along, y + h),
        Side::Left => (x, y + h * along),
        Side::Right => (x + w, y + h * along),
    }
}

/// Wires `trigger` (an element the view builds, keyed `key` here) to `state`: the pointer
/// resting on it for the group's delay (none inside the warm window) opens the popup on it,
/// leaving closes it, and Escape while it has focus dismisses it. `describe` is read after the
/// trigger's label (`aria-describedby`).
pub fn trigger<S: 'static>(
    trigger: El,
    key: &str,
    describe: &str,
    state: &TooltipState,
    lens: impl Fn(&mut S) -> &mut TooltipState + Clone + 'static,
) -> El {
    let now = Instant::now();
    let delays = state.group.delays();
    let (hover_key, layout_key) = (key.to_owned(), key.to_owned());
    let (hover_lens, key_lens, layout_lens) = (lens.clone(), lens.clone(), lens);
    trigger
        .key(key)
        .description(describe)
        .hover_delays(state.group.open_delay(now), delays.close)
        .on_hover_change(move |s: &mut S, inside, tx| {
            let st = hover_lens(s);
            let instant = if inside {
                st.rest(&hover_key).1
            } else {
                st.group.leave(&hover_key, Instant::now()) && st.group.is_instant()
            };
            if instant {
                tx.instant();
            }
            if !st.group.is_open() {
                tx.wake_after(delays.timeout);
            }
        })
        .on_layout_tx(move |s: &mut S, rect, tx| {
            layout_lens(s).rects.insert(layout_key.clone(), rect);
            tx.live_only();
        })
        .on_key(move |s: &mut S, ev, tx| {
            if ev.key != Key::Escape {
                return false;
            }
            let dismissed = key_lens(s).group.dismiss(Instant::now());
            if dismissed {
                tx.instant();
            }
            dismissed
        })
}

/// A key cap inside a tooltip (shadcn's `Kbd`).
pub fn kbd(label: &str, theme: &Theme) -> El {
    let skin = TooltipSkin::new(theme);
    row()
        .h(kbd::HEIGHT)
        .min_w(kbd::MIN_WIDTH)
        .px(kbd::PADDING_X)
        .center_y()
        .justify_center()
        .radius(kbd::RADIUS)
        .fill(skin.kbd_fill.rune())
        .role("none")
        .child(
            text(label)
                .font(sans(kbd::FONT_SIZE, kbd::FONT_WEIGHT, kbd::LINE_HEIGHT))
                .color(skin.kbd_text.rune()),
        )
}

/// The tooltip surface: `Tooltip::new("Bold").keys(&["⌘", "B"]).build(&theme, key, anchor)`.
pub struct Tooltip {
    label: String,
    keys: Vec<String>,
    content: Option<El>,
    placement: Placement,
}

impl Tooltip {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            keys: Vec::new(),
            content: None,
            placement: PLACEMENT,
        }
    }

    /// Key caps after the label (a `KbdGroup`).
    pub fn keys(mut self, keys: &[&str]) -> Self {
        self.keys = keys.iter().map(|k| (*k).to_owned()).collect();
        self
    }

    /// Custom content in place of the label.
    pub fn content(mut self, content: El) -> Self {
        self.content = Some(content);
        self
    }

    /// The side of the trigger it opens on (`top` by default).
    pub fn side(mut self, side: Side) -> Self {
        self.placement.side = side;
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// The surface keyed `key`, on the trigger `state` is open on. Mount it while
    /// [`TooltipState::open_key`] is set; a shared state mounts one surface under one key.
    pub fn build(self, theme: &Theme, key: &str, state: &TooltipState) -> El {
        let active = state.open_key().unwrap_or_default();
        let anchor = state.origin.as_deref().unwrap_or(active);
        let (dx, dy) = match (state.rects.get(active), state.rects.get(anchor)) {
            (Some(&to), Some(&from)) => {
                let (tx, ty) = edge_point(to, self.placement);
                let (fx, fy) = edge_point(from, self.placement);
                (tx - fx, ty - fy)
            }
            _ => (0.0, 0.0),
        };
        let shift = self.placement.align_offset;
        let (sx, sy) = if self.placement.side.is_vertical() {
            (shift, 0.0)
        } else {
            (0.0, shift)
        };
        let skin = TooltipSkin::new(theme);
        let has_keys = !self.keys.is_empty();
        let font = sans(
            metrics::FONT_SIZE,
            metrics::FONT_WEIGHT,
            metrics::LINE_HEIGHT,
        );
        let content = self.content.unwrap_or_else(|| {
            let mut line = row().key(format!("{key}-content")).center_y().child(
                text(self.label.clone())
                    .wrap()
                    .font(font)
                    .color(skin.text.rune()),
            );
            if has_keys {
                line = line.child(
                    row()
                        .ml(kbd::MARGIN_START)
                        .gap(kbd::GAP)
                        .center_y()
                        .children(self.keys.iter().map(|k| kbd(k, theme))),
                );
            }
            line
        });
        let surface = col()
            .key(key)
            .role("tooltip")
            .label(self.label)
            .max_w(metrics::MAX_WIDTH)
            .pl(metrics::PADDING_X)
            .pr(if has_keys {
                metrics::PADDING_END_WITH_KBD
            } else {
                metrics::PADDING_X
            })
            .py(metrics::PADDING_Y)
            .radius(metrics::CORNER)
            .border(metrics::BORDER, skin.border.rune())
            .gradient(skin.gradient.0.rune(), skin.gradient.1.rune())
            .child(arrow(self.placement.side, &skin))
            .child(content)
            .motion(spring(RESIZE));
        let surface = skin
            .shadows
            .iter()
            .fold(surface, |s, shadow| s.box_shadow(shadow.rune()));
        let placement = Placement {
            align_offset: 0.0,
            ..self.placement
        };
        popup(surface, anchor, placement, &MOTION)
            .offset(dx + sx, dy + sy)
            .offset_motion(tween(GLIDE))
    }
}

/// The 10 px square turned 45°, centred on the edge facing the anchor. Rune places the surface
/// itself, so the arrow sits at the middle of that edge (exact for a centred, unshifted popup).
fn arrow(side: Side, skin: &TooltipSkin) -> El {
    let size = metrics::ARROW_SIZE;
    let half = size / 2.0;
    let diamond = el()
        .size(size, size)
        .radius(metrics::ARROW_RADIUS)
        .fill(skin.arrow.rune())
        .rotate(std::f32::consts::FRAC_PI_4);
    let outside = arrow_inset(side) - half - metrics::BORDER;
    let lane = match side {
        Side::Top => row().pin(Some(0.0), None, Some(0.0), Some(outside)),
        Side::Bottom => row().pin(Some(0.0), Some(outside), Some(0.0), None),
        Side::Left => col().pin(None, Some(0.0), Some(outside), Some(0.0)),
        Side::Right => col().pin(Some(outside), Some(0.0), None, Some(0.0)),
    };
    let lane = if side.is_vertical() {
        lane.h(size).justify_center()
    } else {
        lane.w(size).center_y()
    };
    lane.role("none").child(diamond)
}

/// The duotone tooltip's trigger (`rounded-md border bg-background px-3 py-1.5 text-sm`).
pub fn duotone_trigger(label: &str, theme: &Theme) -> El {
    use duotone::trigger as m;
    let skin = duotone::DuotoneSkin::new(theme);
    row()
        .px(m::PADDING_X)
        .py(m::PADDING_Y)
        .radius(m::RADIUS)
        .border(1.0, skin.trigger_border.rune())
        .fill(skin.trigger_fill.rune())
        .hover(skin.trigger_hover.rune())
        .hover_motion(
            crate::rune::tween(duotone::MOTION),
            crate::rune::tween(duotone::MOTION),
        )
        .role("button")
        .label(label)
        .focusable(true)
        .focus_ring(3.0, 0.0, theme.ring.fade(0.5).rune())
        .child(
            text(label)
                .font(sans(m::FONT_SIZE, 400.0, m::LINE_HEIGHT))
                .color(skin.trigger_text.rune()),
        )
}

/// The duotone tooltip card: a shell header (title and an optional aside) over a content card
/// in a 2 px rim. Mount it, anchored below its trigger, while a [`HoverIntent`] (no delays)
/// is open; it stays open while the pointer is over it.
pub struct DuotoneTooltip {
    title: String,
    aside: Option<String>,
    body: String,
}

impl DuotoneTooltip {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            aside: None,
            body: body.into(),
        }
    }

    pub fn aside(mut self, aside: impl Into<String>) -> Self {
        self.aside = Some(aside.into());
        self
    }

    /// The card keyed `key` below the trigger keyed `anchor`, reporting its hover to `lens`.
    pub fn build<S: 'static>(
        self,
        theme: &Theme,
        key: &str,
        anchor: &str,
        lens: impl Fn(&mut S) -> &mut HoverIntent + 'static,
    ) -> El {
        use duotone as d;
        let skin = d::DuotoneSkin::new(theme);
        let mut header = row()
            .px(d::HEADER_PADDING_X)
            .py(d::HEADER_PADDING_Y)
            .gap(d::HEADER_GAP)
            .center_y()
            .child(
                text(self.title.clone())
                    .truncate()
                    .font(sans(14.0, 600.0, 20.0))
                    .color(skin.title.rune())
                    .shrink(),
            )
            .child(el().grow(1.0));
        if let Some(aside) = self.aside {
            header = header.child(
                text(aside)
                    .font(sans(12.0, 400.0, 16.0))
                    .color(skin.aside.rune()),
            );
        }
        let card = col()
            .px(d::CARD_PADDING_X)
            .py(d::CARD_PADDING_Y + d::BODY_MARGIN_Y)
            .radius(d::CARD_RADIUS)
            .border(1.0, skin.card_border.rune())
            .fill(skin.card.rune())
            .child(
                text(self.body)
                    .wrap()
                    .font(sans(12.0, 400.0, 16.0))
                    .color(skin.body.rune()),
            );
        let shell = col()
            .w(d::WIDTH)
            .px(d::RIM)
            .pb(d::RIM)
            .radius(d::SHELL_RADIUS)
            .border(1.0, skin.shell_border.rune())
            .fill(skin.shell.rune())
            .child(header)
            .child(card);
        let shell = skin
            .shadows
            .iter()
            .fold(shell, |s, shadow| s.box_shadow(shadow.rune()));
        let motion = spring(d::MOTION);
        let hidden = ::rune_motion::Effect {
            opacity: 0.0,
            offset: Vec2::new(0.0, d::TRAVEL),
            ..::rune_motion::Effect::IDENTITY
        };
        col()
            .key(key)
            .role("tooltip")
            .label(self.title)
            .pt(d::GAP)
            .child(shell)
            .anchored(anchor, ::rune::Side::Bottom, ::rune::Align::Center, 0.0)
            .transition(Transition {
                insertion: hidden,
                removal: hidden,
                pop_layout: true,
                springs: SpringPair::new(motion, motion),
            })
            .on_hover_change(move |s: &mut S, inside, _| {
                lens(s).set_card(inside, Instant::now());
            })
    }
}
