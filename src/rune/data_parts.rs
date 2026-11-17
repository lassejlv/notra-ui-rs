//! Rune renderings of the small parts the data surfaces share. See
//! [`crate::components::data_parts`] for the measurements.

use ::rune::{El, HelpLook, Transition, col, el, icon, row, text};
use ::rune_motion::{Anim, Beat, CubicBezier as RuneBezier, Easing, LoopSpec, Sequence};

use super::{ToRune, mono, sans, tween};
use crate::components::data_parts::{
    badge, checkbox, copy, empty, shimmer, skeleton, spinner, tabs, tooltip,
};
use crate::icons::Icon;
use crate::motion::Tween;
use crate::theme::{Color, Theme};

pub use crate::components::data_parts::badge::BadgeVariant;
pub use crate::components::data_parts::checkbox::CheckState;

/// A CSS transition with its delay, for `*_motion`.
pub fn anim(t: Tween) -> Anim {
    Anim {
        delay: t.delay.as_secs_f32(),
        ..Anim::new(tween(t))
    }
}

/// Notra's tooltip look for `El::help`. Set it once per view run:
/// `cx.help_look(notra_ui::rune::tooltip_look(&theme))`.
pub fn tooltip_look(theme: &Theme) -> HelpLook {
    let skin = tooltip::TooltipSkin::new(theme);
    HelpLook::system(theme.is_dark())
        .fill(skin.top.mix(skin.bottom, 0.5).rune())
        .ink(skin.text.rune())
        .ring(1.0, skin.border.rune())
        .radius(tooltip::RADIUS)
        .padding(tooltip::PADDING_X, tooltip::PADDING_Y)
        .font(sans(tooltip::FONT_SIZE, 400.0, tooltip::LINE_HEIGHT))
        .max_width(Some(tooltip::MAX_WIDTH + 2.0 * tooltip::PADDING_X))
        .gap(tooltip::GAP)
        .delay(0.0)
        .fade(
            tooltip::ENTER.duration.as_secs_f64(),
            tooltip::EXIT.duration.as_secs_f64(),
            tooltip::FROM_SCALE,
        )
        .shadows(tooltip::SHADOWS.iter().map(|s| s.rune()))
}

/// lucide's `loader-circle`, turning once a second.
pub fn spinner(color: Color) -> El {
    icon(Icon::LoaderCircle.rune(), spinner::SIZE)
        .color(color.rune())
        .looping(LoopSpec::rotation(spinner::TURN.period.as_secs_f64()))
}

/// `label` in `base` with Notra's shimmer band sweeping across it in `highlight`.
///
/// A window as wide as the band slides over the label while a bright copy inside it
/// slides back by the same amount, so the copy stays on the label's glyphs.
pub fn shimmer(label: &str, size: f32, line_height: f32, base: Color, highlight: Color) -> El {
    let font = sans(size, 400.0, line_height);
    let (width, _) = ::rune::text::measure(label, &[], font, None);
    let spread = shimmer::SPREAD_EM * size;
    let band = 2.0 * spread;
    let start = shimmer::center(0.0, width) - spread;
    let end = shimmer::center(1.0, width) - spread;
    let period = shimmer::SWEEP.period.as_secs_f64();
    let slide = |a: f32, b: f32| LoopSpec::slide(period, [a, 0.0], [b, 0.0]).easing(Easing::Linear);
    let lit = el()
        .key("shimmer-lit")
        .absolute(0.0, 0.0)
        .w(band)
        .h_full()
        .looping(slide(-start / band, -end / band))
        .child(
            text(label)
                .font(font)
                .color(highlight.rune())
                .absolute(0.0, 0.0),
        );
    let sheen = el()
        .key("shimmer-sheen")
        .absolute(0.0, 0.0)
        .w(band)
        .h_full()
        .clip_fade_linear(spread, 0.0, spread, 0.0)
        .looping(slide(start / band, end / band))
        .child(lit);
    row()
        .child(
            text(label)
                .key("shimmer-label")
                .font(font)
                .color(base.rune()),
        )
        .child(sheen)
}

/// The base-ui checkbox. `ink` tints the mixed state's check.
pub fn checkbox(theme: &Theme, state: CheckState, ink: Color, label: impl Into<String>) -> El {
    let skin = checkbox::CheckboxSkin::new(theme, state, ink);
    let mut e = el()
        .size(checkbox::SIZE, checkbox::SIZE)
        .center()
        .radius(checkbox::RADIUS)
        .border(1.0, skin.border.rune())
        .fill(skin.fill.rune())
        .role("checkbox")
        .label(label.into())
        .focusable(true)
        .focus_ring(3.0, 0.0, theme.ring.fade(0.5).rune())
        .cursor(::rune::Cursor::Pointer);
    e = match state {
        CheckState::On => e.checked(true),
        CheckState::Off => e.checked(false),
        CheckState::Mixed => e.checked_mixed(),
    };
    if let Some(mark) = skin.mark {
        e = e.child(icon(Icon::Check.rune(), checkbox::ICON).color(mark.rune()));
    }
    e
}

/// shadcn's badge.
pub fn badge(theme: &Theme, label: impl Into<String>, variant: BadgeVariant) -> El {
    let skin = badge::BadgeSkin::new(theme, variant);
    row()
        .h(badge::HEIGHT)
        .px(badge::PADDING_X - 1.0)
        .center_y()
        .radius(badge::RADIUS)
        .border(1.0, skin.border.rune())
        .fill(skin.fill.rune())
        .child(
            text(label.into())
                .font(sans(badge::FONT_SIZE, 500.0, badge::LINE_HEIGHT))
                .color(skin.text.rune()),
        )
}

/// shadcn's skeleton bar, pulsing.
pub fn skeleton(theme: &Theme, w: Option<f32>, h: f32) -> El {
    let mut e = el()
        .h(h)
        .radius(6.0)
        .fill(theme.muted.rune())
        .looping(LoopSpec::opacity(
            skeleton::PULSE.period.as_secs_f64(),
            0.0,
            0.5,
            1.0,
        ));
    if let Some(w) = w {
        e = e.w(w);
    }
    e
}

const SWAP_IN: Easing = Easing::CubicBezier(RuneBezier::new(0.4, 0.0, 0.2, 1.0));
/// The copy icon: out over 150 ms, back after the feedback.
const COPY_AWAY: &[Beat] = &[
    Beat::new(0.0),
    Beat::new(0.15).opacity(0.0).scale(0.5).easing(SWAP_IN),
    Beat::new(1.5).opacity(0.0).scale(0.5),
    Beat::new(1.65).easing(SWAP_IN),
];
/// The tick (or alert): in over 150 ms, out after the feedback.
const FEEDBACK_SHOW: &[Beat] = &[
    Beat::new(0.0).opacity(0.0).scale(0.5),
    Beat::new(0.15).easing(SWAP_IN),
    Beat::new(1.5),
    Beat::new(1.65).opacity(0.0).scale(0.5).easing(SWAP_IN),
];

/// A copy button that writes `value` to the clipboard and shows a tick for 1.5 s.
///
/// The app counts copies (`copies`) and bumps the count in `on_copy`: each new
/// count replays the swap.
pub struct CopyButton {
    value: String,
    metrics: copy::CopyMetrics,
    label: String,
    copies: u32,
    press: Option<Box<dyn FnOnce(El) -> El>>,
}

impl CopyButton {
    /// `label` is the accessible name (`Copy snippet`, `Copy command`).
    pub fn new(
        value: impl Into<String>,
        metrics: copy::CopyMetrics,
        label: impl Into<String>,
    ) -> Self {
        Self {
            value: value.into(),
            metrics,
            label: label.into(),
            copies: 0,
            press: None,
        }
    }

    pub fn copies(mut self, copies: u32) -> Self {
        self.copies = copies;
        self
    }

    pub fn on_copy<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        let value = self.value.clone();
        self.press = Some(Box::new(move |e: El| {
            e.on_press_tx(move |s: &mut S, tx| {
                tx.copy(value.clone());
                f(s);
            })
        }));
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let m = self.metrics;
        let fade = tween(copy::HOVER);
        let glyph = |i: Icon, color: Color| {
            icon(i.rune(), m.icon)
                .color(color.rune())
                .group_color(theme.foreground.rune())
        };
        let replay = self.copies;
        let layer = |key: &str, child: El, beats: Option<&'static [Beat]>| {
            let mut e = el()
                .key(format!("{key}-{replay}"))
                .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .center()
                .transition(Transition::none())
                .child(child);
            if let Some(beats) = beats {
                e = e.sequence(Sequence::new(beats));
            }
            e
        };
        let copy_icon = layer(
            "copy",
            glyph(Icon::HugeCopy01, theme.muted_foreground),
            (replay > 0).then_some(COPY_AWAY),
        );
        let mut e = el()
            .size(m.size, m.size)
            .radius(m.radius)
            .role("button")
            .label(self.label)
            .focusable(true)
            .focus_ring(3.0, 0.0, theme.ring.fade(0.5).rune())
            .hover(theme.muted.rune())
            .hover_group()
            .hover_motion(fade, fade)
            .cursor(::rune::Cursor::Pointer)
            .child(copy_icon);
        if replay > 0 {
            e = e.child(
                layer(
                    "tick",
                    glyph(Icon::HugeTick02, theme.muted_foreground),
                    Some(FEEDBACK_SHOW),
                )
                .opacity(1.0),
            );
        }
        if let Some(press) = self.press {
            e = press(e);
        }
        e
    }
}

/// One tab of [`Tabs`].
#[derive(Clone, Debug)]
pub struct TabOption {
    pub label: String,
    pub icon: Option<Icon>,
}

impl TabOption {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}

type TabSelect = Box<dyn Fn(usize) -> Box<dyn FnOnce(El) -> El>>;

/// base-ui tabs with the sliding underline.
pub struct Tabs {
    options: Vec<TabOption>,
    selected: usize,
    mono: bool,
    label: String,
    select: Option<TabSelect>,
}

impl Tabs {
    pub fn new(options: Vec<TabOption>, selected: usize) -> Self {
        Self {
            options,
            selected,
            mono: false,
            label: String::new(),
            select: None,
        }
    }

    /// Set labels in the mono family (the package command).
    pub fn mono(mut self, mono: bool) -> Self {
        self.mono = mono;
        self
    }

    /// The tab list's accessible name.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn on_select<S: 'static>(mut self, f: impl Fn(&mut S, usize) + Clone + 'static) -> Self {
        self.select = Some(Box::new(move |i| {
            let f = f.clone();
            Box::new(move |e: El| e.on_press(move |s: &mut S| f(s, i)))
        }));
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let font = if self.mono {
            mono(tabs::FONT_SIZE, 400.0, tabs::LINE_HEIGHT)
        } else {
            sans(tabs::FONT_SIZE, 400.0, tabs::LINE_HEIGHT)
        };
        let widths: Vec<f32> = self
            .options
            .iter()
            .map(|o| {
                let icon = o.icon.map_or(0.0, |_| tabs::ICON + tabs::ICON_GAP);
                ::rune::text::measure(&o.label, &[], font, None).0 + icon
            })
            .collect();
        let selected = self.selected.min(self.options.len().saturating_sub(1));
        let left: f32 = widths[..selected].iter().map(|w| w + tabs::GAP).sum();
        let width = widths.get(selected).copied().unwrap_or(0.0);
        let fade = tween(tabs::COLOR);
        let mut list = row()
            .key("tabs")
            .h(tabs::HEIGHT)
            .gap(tabs::GAP)
            .role("tablist")
            .label(self.label)
            .focus_group();
        for (i, option) in self.options.into_iter().enumerate() {
            let active = i == selected;
            let ink = if active {
                theme.foreground
            } else {
                theme.muted_foreground
            };
            let mut tab = row()
                .key(format!("tab-{i}"))
                .h_full()
                .center_y()
                .gap(tabs::ICON_GAP)
                .radius(4.0)
                .role("tab")
                .label(option.label.clone())
                .checked(active)
                .focusable(true)
                .focus_selected(active)
                .focus_ring(2.0, -2.0, theme.ring.rune())
                .hover_group()
                .hover_motion(fade, fade)
                .cursor(::rune::Cursor::Pointer);
            if let Some(i) = option.icon {
                tab = tab.child(
                    icon(i.rune(), tabs::ICON)
                        .color(ink.rune())
                        .group_color(theme.foreground.rune()),
                );
            }
            tab = tab.child(
                text(option.label)
                    .font(font)
                    .color(ink.rune())
                    .group_color(theme.foreground.rune()),
            );
            if let Some(select) = &self.select {
                tab = select(i)(tab);
            }
            list = list.child(tab);
        }
        list.child(
            el().key("tabs-indicator")
                .absolute(
                    0.0,
                    tabs::HEIGHT - tabs::INDICATOR_BOTTOM - tabs::INDICATOR_HEIGHT,
                )
                .offset(left, 0.0)
                .offset_motion(anim(tabs::SLIDE))
                .w(width)
                .h(tabs::INDICATOR_HEIGHT)
                .radius(tabs::INDICATOR_HEIGHT / 2.0)
                .fill(theme.foreground.rune())
                .motion(::rune_motion::Spring::new(
                    tabs::SLIDE.duration.as_secs_f32(),
                    0.0,
                )),
        )
    }
}

/// shadcn's `Empty`: an icon tile, a title, a description and actions under it.
pub fn empty_state(
    theme: &Theme,
    glyph: Icon,
    title: &str,
    description: Option<&str>,
    actions: Vec<El>,
) -> El {
    let mut header = col()
        .center()
        .gap(empty::HEADER_GAP)
        .max_w(empty::DESCRIPTION_MAX_WIDTH)
        .child(
            el().size(empty::MEDIA_SIZE, empty::MEDIA_SIZE)
                .mb(empty::MEDIA_MARGIN)
                .center()
                .radius(empty::MEDIA_RADIUS)
                .fill(theme.muted.rune())
                .child(icon(glyph.rune(), empty::MEDIA_ICON).color(theme.foreground.rune())),
        )
        .child(
            text(title)
                .font(sans(14.0, 500.0, 20.0).tracking(empty::TITLE_TRACKING))
                .color(theme.foreground.rune()),
        );
    if let Some(description) = description {
        header = header.child(
            text(description)
                .font(sans(14.0, 400.0, empty::DESCRIPTION_LINE_HEIGHT))
                .color(theme.muted_foreground.rune())
                .align(::rune::Align::Center)
                .wrap(),
        );
    }
    let mut e = col()
        .center()
        .gap(empty::GAP)
        .padding(empty::PADDING)
        .child(header);
    if !actions.is_empty() {
        e = e.child(col().center().gap(empty::CONTENT_GAP).children(actions));
    }
    e
}
