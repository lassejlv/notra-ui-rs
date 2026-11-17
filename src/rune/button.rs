use ::rune::{El, Tx, icon, row, text};
use ::rune_motion::{Beat, Easing, LoopSpec, Sequence, Transition};

use super::{ToRune, brightness_overlay, sans, tween};
use crate::components::button::{
    ButtonHover, ButtonSize, ButtonSkin, ButtonVariant, DISABLED_OPACITY, TRANSITION, loading,
};
use crate::icons::Icon;
use crate::theme::Theme;

type Press = Box<dyn FnOnce(El) -> El>;

/// The Depth button: `Button::new("Save").variant(ButtonVariant::Primary).build(&theme)`.
pub struct Button {
    label: Option<String>,
    icon: Option<Icon>,
    icon_end: Option<Icon>,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    loading: bool,
    progress: Option<f32>,
    press: Option<Press>,
}

/// The dot bounce, one period: rest, up by its own height at 30 %, back by 60 %.
const DOT_RISE: Easing = Easing::Piecewise(&[
    [0.0, 0.0],
    [0.075, 0.156],
    [0.15, 0.5],
    [0.225, 0.844],
    [0.3, 1.0],
    [0.375, 0.844],
    [0.45, 0.5],
    [0.525, 0.156],
    [0.6, 0.0],
    [1.0, 0.0],
]);

const DOT_PULSE: &[Beat] = &[
    Beat::new(0.0).opacity(0.45),
    Beat::new(0.3).opacity(1.0),
    Beat::new(0.6).opacity(0.45),
    Beat::new(1.0).opacity(0.45),
];

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            ..Self::bare(None)
        }
    }

    /// A square button showing only `icon`.
    pub fn icon_button(icon: Icon) -> Self {
        Self::bare(Some(icon)).size(ButtonSize::Icon)
    }

    fn bare(icon: Option<Icon>) -> Self {
        Self {
            label: None,
            icon,
            icon_end: None,
            variant: ButtonVariant::Primary,
            size: ButtonSize::Default,
            disabled: false,
            loading: false,
            progress: None,
            press: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn icon_end(mut self, icon: Icon) -> Self {
        self.icon_end = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Swaps the label for bouncing dots while an action runs.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Fills the button from the left, `0.0..=100.0`. `None` when done.
    pub fn progress(mut self, progress: Option<f32>) -> Self {
        self.progress = progress;
        self
    }

    pub fn on_press<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(Box::new(move |e: El| e.on_press(f)));
        self
    }

    pub fn on_press_tx<S: 'static>(mut self, f: impl Fn(&mut S, &mut Tx) + 'static) -> Self {
        self.press = Some(Box::new(move |e: El| e.on_press_tx(f)));
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let m = self.size.metrics();
        let skin = ButtonSkin::new(theme, self.variant);
        let em = m.font_size;
        let ink = skin.text.rune();
        let busy = self.loading || self.progress.is_some();
        let interactive = !self.disabled && !busy;
        let font = sans(m.font_size, 500.0, m.line_height);
        let fade = tween(TRANSITION);

        let glyph = |i: Icon| {
            let mut g = icon(i.rune(), m.icon_size).color(ink);
            if let ButtonHover::Fill { text, .. } = skin.hover {
                g = g.group_color(text.rune());
            }
            g
        };
        let mut label = row().key("label").center_y().gap(m.gap);
        if let Some(i) = self.icon {
            label = label.child(glyph(i).key("icon"));
        }
        if let Some(t) = self.label.clone() {
            let mut t = text(t).key("text").font(font).color(ink);
            if let ButtonHover::Fill { text, .. } = skin.hover {
                t = t.group_color(text.rune());
            }
            label = label.child(t);
        }
        if let Some(i) = self.icon_end {
            label = label.child(glyph(i).key("icon-end"));
        }
        let travel = loading::SWAP_TRAVEL_EM * em;
        let label = if self.loading {
            label
                .opacity(0.0)
                .offset(0.0, -travel)
                .blur(loading::SWAP_BLUR * 0.5)
        } else {
            label.opacity(1.0).offset(0.0, 0.0).blur(0.0)
        }
        .opacity_motion(tween(loading::SWAP_OUT))
        .offset_motion(tween(loading::SWAP_OUT))
        .blur_motion(tween(loading::SWAP_OUT));

        let mut e = row()
            .h(m.height)
            .center_y()
            .justify_center()
            .gap(m.gap)
            .radius(skin.radius.unwrap_or(m.radius))
            .border(1.0, skin.border.rune())
            .fill(skin.fill.rune())
            .role("button")
            .label(
                self.label
                    .clone()
                    .or_else(|| self.icon.map(|i| i.name().replace('-', " ")))
                    .unwrap_or_default(),
            )
            .busy(busy)
            .focusable(interactive)
            .focus_ring(3.0, 0.0, theme.ring.fade(0.5).rune());
        e = match m.padding_x {
            Some(p) => e.px(p),
            None => e.w(m.height),
        };
        if skin.radius.is_some() {
            e = e.continuous_corners();
        }
        if let Some((top, bottom)) = skin.gradient {
            e = e.gradient(top.rune(), bottom.rune());
        }
        for s in &skin.shadows {
            e = e.box_shadow(s.rune());
        }
        if interactive {
            let overlay = match skin.hover {
                ButtonHover::Brightness(f) => {
                    let base = skin.gradient.map_or(skin.fill, |(t, b)| t.mix(b, 0.5));
                    Some(brightness_overlay(base, f))
                }
                ButtonHover::Gradient { .. } => Some(theme.secondary.fade(0.15)),
                ButtonHover::Fill { fill, .. } => Some(fill),
                ButtonHover::Underline => None,
            };
            if let Some(o) = overlay {
                e = e.hover(o.rune()).hover_motion(fade, fade);
            }
            e = e.hover_group().cursor(::rune::Cursor::Pointer);
            if let Some(press) = self.press {
                e = press(e);
            }
        } else if self.disabled {
            e = e.opacity(DISABLED_OPACITY);
        }

        if let Some(value) = self.progress {
            let p = (value / 100.0).clamp(0.0, 1.0);
            let fill = skin.text.fade(loading::PROGRESS_ALPHA).rune();
            e = e
                .child(
                    row()
                        .key("progress")
                        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                        .transition(Transition::opacity())
                        .child(
                            el_fill()
                                .key("progress-fill")
                                .h_full()
                                .grow(p.max(0.0001))
                                .fill(fill)
                                .motion(::rune_motion::Spring::new(0.3, 0.0)),
                        )
                        .child(el_fill().key("progress-rest").grow((1.0 - p).max(0.0001))),
                )
                .clip_rounded(skin.radius.unwrap_or(m.radius));
        }

        e = e.child(label);

        if self.loading {
            let size = loading::DOT_SIZE_EM * em;
            let dots = (0..3).map(|i| {
                let bounce = LoopSpec::slide(1.0, [0.0, 0.0], [0.0, -1.0]).easing(DOT_RISE);
                let bounce = LoopSpec {
                    delay: loading::DOT_BOUNCE_DELAYS[i].as_secs_f64(),
                    ..bounce
                };
                let pulse = LoopSpec::sequence(
                    Sequence::new(DOT_PULSE).delay(loading::DOT_BOUNCE_DELAYS[i].as_secs_f64()),
                );
                el_fill()
                    .key(format!("dot-{i}"))
                    .enter_delay(loading::DOT_ENTER_DELAYS[i].as_secs_f32())
                    .transition(swap_transition(travel))
                    .child(
                        el_fill().size(size, size).looping(bounce).child(
                            el_fill()
                                .size(size, size)
                                .radius(size / 2.0)
                                .fill(ink)
                                .looping(pulse),
                        ),
                    )
            });
            e = e.child(
                row()
                    .key("loader")
                    .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                    .center_y()
                    .justify_center()
                    .gap(loading::DOT_GAP_EM * em)
                    .transition(swap_transition(travel))
                    .children(dots),
            );
        }
        e
    }
}

fn el_fill() -> El {
    ::rune::el()
}

/// In from below, out through the top, blurring at the far ends.
fn swap_transition(travel: f32) -> Transition {
    let blur = loading::SWAP_BLUR * 0.5;
    Transition::asymmetric(
        Transition::offset(0.0, travel).combined(Transition::blur(blur)),
        Transition::offset(0.0, -travel).combined(Transition::blur(blur)),
    )
}
