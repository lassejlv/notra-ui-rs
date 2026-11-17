use ::rune::{El, el, icon};

use crate::components::primitives::checkbox::{self, CheckState, CheckboxSkin};
use crate::components::primitives::focus_ring;
use crate::rune::ToRune;
use crate::theme::Theme;

type Press = Box<dyn FnOnce(El, CheckState) -> El>;

/// A checkbox: `Checkbox::new(s.agreed.into()).on_change(|s: &mut S, next| ..).build(&theme)`.
pub struct Checkbox {
    state: CheckState,
    disabled: bool,
    label: Option<String>,
    press: Option<Press>,
}

impl Checkbox {
    pub fn new(state: CheckState) -> Self {
        Self {
            state,
            disabled: false,
            label: None,
            press: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The accessible name (`aria-label`).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Called with the next state when pressed.
    pub fn on_change<S: 'static>(mut self, f: impl Fn(&mut S, CheckState) + 'static) -> Self {
        self.press = Some(Box::new(move |e: El, next| {
            e.on_press(move |s: &mut S| f(s, next))
        }));
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let skin = CheckboxSkin::new(theme, self.state);
        let (ring_width, ring) = focus_ring(theme);
        let mut e = el()
            .size(checkbox::SIZE, checkbox::SIZE)
            .radius(checkbox::RADIUS)
            .border(1.0, skin.border.rune())
            .fill(skin.fill.rune())
            .center()
            .role("checkbox")
            .disabled(self.disabled)
            .focusable(!self.disabled)
            .focus_ring(ring_width, 0.0, ring.rune());
        e = match self.state {
            CheckState::Indeterminate => e.checked_mixed(),
            state => e.checked(state.is_on()),
        };
        if let Some(label) = self.label {
            e = e.label(label);
        }
        if let Some(glyph) = self.state.icon() {
            e = e.child(icon(glyph.rune(), checkbox::ICON_SIZE).color(skin.mark.rune()));
        }
        if self.disabled {
            return e.opacity(checkbox::DISABLED_OPACITY);
        }
        let next = self.state.toggled();
        match self.press {
            Some(press) => press(e.cursor(::rune::Cursor::Pointer), next),
            None => e,
        }
    }
}
