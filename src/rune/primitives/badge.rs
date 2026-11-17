use ::rune::{El, icon, row, text};

use crate::components::primitives::COLOR_TRANSITION;
use crate::components::primitives::badge::{self, BadgeSkin, BadgeVariant};
use crate::icons::Icon;
use crate::rune::{ToRune, sans, tween};
use crate::theme::Theme;

type Press = Box<dyn FnOnce(El) -> El>;

/// shadcn's badge: `Badge::new("New").variant(BadgeVariant::Secondary).build(&theme)`.
pub struct Badge {
    label: String,
    variant: BadgeVariant,
    icon: Option<Icon>,
    icon_end: Option<Icon>,
    press: Option<Press>,
}

impl Badge {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::Default,
            icon: None,
            icon_end: None,
            press: None,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
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

    /// Makes the badge a link, which turns on its hover style.
    pub fn on_press<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(Box::new(move |e: El| e.on_press(f)));
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let skin = BadgeSkin::new(theme, self.variant);
        let link = self.press.is_some();
        let hover = skin.hover_for(link);
        let ink = skin.text.rune();
        let glyph = |i: Icon| {
            let g = icon(i.rune(), badge::ICON_SIZE).color(ink);
            match hover {
                Some((_, text)) => g.group_color(text.rune()),
                None => g,
            }
        };
        let mut label = text(self.label.clone())
            .font(sans(badge::FONT_SIZE, badge::WEIGHT, badge::LINE_HEIGHT))
            .color(ink);
        if let Some((_, text)) = hover {
            label = label.group_color(text.rune());
        }
        let side = |has: bool| {
            if has {
                badge::PADDING_X_ICON
            } else {
                badge::PADDING_X
            }
        };
        let mut e = row()
            .h(badge::HEIGHT)
            .pl(side(self.icon.is_some()))
            .pr(side(self.icon_end.is_some()))
            .gap(badge::GAP)
            .center_y()
            .justify_center()
            .radius(badge::RADIUS.min(badge::HEIGHT / 2.0))
            .border(1.0, skin.border.rune())
            .fill(skin.fill.rune())
            .label(self.label);
        if let Some(i) = self.icon {
            e = e.child(glyph(i));
        }
        e = e.child(label);
        if let Some(i) = self.icon_end {
            e = e.child(glyph(i));
        }
        if let Some((fill, _)) = hover {
            let fade = tween(COLOR_TRANSITION);
            let overlay = if fill.a < 1.0 && skin.fill.a >= 1.0 {
                theme.background.fade(1.0 - fill.a)
            } else {
                fill
            };
            e = e
                .hover(overlay.rune())
                .hover_motion(fade, fade)
                .hover_group();
        }
        match self.press {
            Some(press) => press(
                e.role("link")
                    .cursor(::rune::Cursor::Pointer)
                    .focusable(true),
            ),
            None => e,
        }
    }
}
