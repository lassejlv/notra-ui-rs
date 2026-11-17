use ::rune::{El, Tx, el, icon, row, text};

use super::{spin, swap_transition};
use crate::components::button::loading::SWAP_TRAVEL_EM;
use crate::components::primitives::focus_ring;
use crate::components::primitives::marketing_button::{
    DISABLED_OPACITY, GAP, MarketingSize, MarketingSkin, MarketingVariant, TRACKING_EM, TRANSITION,
    WEIGHT, loading,
};
use crate::icons::Icon;
use crate::rune::{ToRune, sans, tween};
use crate::theme::{Color, Theme};

type Press = Box<dyn FnOnce(El) -> El>;

/// Notra's marketing call to action:
/// `MarketingButton::new("Start for free").icon_end(Icon::ArrowRight).build(&theme)`.
pub struct MarketingButton {
    label: String,
    icon: Option<Icon>,
    icon_end: Option<Icon>,
    variant: MarketingVariant,
    size: MarketingSize,
    disabled: bool,
    loading: bool,
    press: Option<Press>,
}

impl MarketingButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            icon_end: None,
            variant: MarketingVariant::Primary,
            size: MarketingSize::Default,
            disabled: false,
            loading: false,
            press: None,
        }
    }

    pub fn variant(mut self, variant: MarketingVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: MarketingSize) -> Self {
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

    /// Swaps the label for a spinner; the button keeps its width.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
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
        let skin = MarketingSkin::new(self.variant);
        let ink = skin.text.rune();
        let font = sans(m.font_size, WEIGHT, m.line_height).tracking(TRACKING_EM * m.font_size);
        let travel = SWAP_TRAVEL_EM * m.font_size;
        let interactive = !self.disabled && !self.loading;
        let fade = tween(TRANSITION);
        let pin = |e: El| e.pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0));

        let label = |key: &'static str| {
            let glyph = |i: Icon| icon(i.rune(), m.icon_size).color(ink);
            let mut row = row().key(key).center_y().justify_center().gap(GAP);
            if let Some(i) = self.icon {
                row = row.child(glyph(i));
            }
            row = row.child(text(self.label.clone()).font(font).color(ink));
            if let Some(i) = self.icon_end {
                row = row.child(glyph(i));
            }
            row
        };

        let radius = m.height / 2.0;
        let (ring_width, ring) = focus_ring(theme);
        let mut e = row()
            .h(m.height)
            .px(m.padding_x)
            .center_y()
            .justify_center()
            .radius(radius)
            .gradient(skin.rest.from.rune(), skin.rest.to.rune())
            .role("button")
            .label(self.label.clone())
            .busy(self.loading)
            .focusable(interactive)
            .focus_ring(ring_width, 0.0, ring.rune());
        for shadow in &skin.shadows {
            e = e.box_shadow(shadow.rune());
        }
        if interactive {
            e = e
                .child(
                    pin(el())
                        .key("hover")
                        .radius(radius)
                        .gradient(skin.hover.from.rune(), skin.hover.to.rune())
                        .group_opacity(0.0, 1.0),
                )
                .hover_group()
                .hover_motion(fade, fade)
                .press(press_overlay(&skin).rune())
                .cursor(::rune::Cursor::Pointer);
            if let Some(press) = self.press {
                e = press(e);
            }
        } else if self.disabled {
            e = e.opacity(DISABLED_OPACITY);
        }

        e = e.child(label("label-size").opacity(0.0).a11y_hidden());
        if !self.loading {
            e = e.child(
                pin(label("label"))
                    .enter_delay(loading::ENTER_DELAY.as_secs_f32())
                    .transition(swap_transition(travel)),
            );
        } else {
            let size = loading::SPINNER_EM * m.font_size;
            e = e.child(
                pin(row())
                    .key("spinner")
                    .center()
                    .enter_delay(loading::ENTER_DELAY.as_secs_f32())
                    .transition(swap_transition(travel))
                    .child(
                        el().size(size, size)
                            .child(
                                icon(Icon::SpinnerTrack.rune(), size)
                                    .color(skin.text.fade(loading::TRACK_OPACITY).rune())
                                    .absolute(0.0, 0.0),
                            )
                            .child(
                                icon(Icon::SpinnerArc.rune(), size)
                                    .color(ink)
                                    .absolute(0.0, 0.0)
                                    .looping(spin()),
                            ),
                    ),
            );
        }
        e
    }
}

/// `:active` darkens both stops; Rune's press is a flat overlay, so this is the black
/// that takes the resting gradient's midpoint to the pressed one's.
fn press_overlay(skin: &MarketingSkin) -> Color {
    let luma = |c: Color| 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
    let rest = luma(skin.rest.from.mix(skin.rest.to, 0.5));
    let active = luma(skin.active.from.mix(skin.active.to, 0.5));
    Color::BLACK.alpha((1.0 - active / rest.max(1e-3)).clamp(0.0, 1.0))
}
