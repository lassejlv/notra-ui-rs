use std::rc::Rc;

use ::rune::{El, col, el, row, text};
use ::rune_motion::{Effect, SpringPair, Transition, Vec2};

use super::{Side, spring};
use crate::components::button::{ButtonSize, ButtonVariant};
use crate::components::overlays::dialog::{
    DIALOG_BACKDROP, DIALOG_MOTION, ModalSkin, SHEET_BACKDROP, SHEET_MOTION, metrics,
};
use crate::icons::Icon;
use crate::motion::Tween;
use crate::rune::{Button, ToRune, sans};
use crate::theme::{Shadow, Theme};

type Close<S> = Rc<dyn Fn(&mut S)>;

/// The content shared by dialogs and sheets.
struct Content {
    key: String,
    title: Option<String>,
    description: Option<String>,
    body: Vec<El>,
    footer: Vec<El>,
    close_button: bool,
}

impl Content {
    fn new(key: String) -> Self {
        Self {
            key,
            title: None,
            description: None,
            body: Vec::new(),
            footer: Vec::new(),
            close_button: true,
        }
    }
}

fn backdrop(key: &str, skin: &ModalSkin, fade: Tween) -> El {
    el().key(format!("{key}-backdrop"))
        .absolute(0.0, 0.0)
        .w_full()
        .h_full()
        .fill(skin.backdrop.rune())
        .backdrop_blur(metrics::BACKDROP_BLUR / 2.0)
        .a11y_hidden()
        .transition(Transition::opacity().springs(SpringPair::new(spring(fade), spring(fade))))
}

fn close_button<S: 'static>(theme: &Theme, close: &Close<S>, inset: f32) -> El {
    let close = close.clone();
    el().key("close")
        .pin(None, Some(inset), Some(inset), None)
        .child(
            Button::icon_button(Icon::X)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .on_press(move |s: &mut S| close(s))
                .build(theme)
                .label("Close"),
        )
}

fn title(text_: &str, line: f32, theme: &Theme) -> El {
    text(text_.to_string())
        .key("title")
        .font(sans(metrics::TITLE_FONT, 500.0, line))
        .color(theme.foreground.rune())
}

fn description(text_: &str, skin: &ModalSkin) -> El {
    text(text_.to_string())
        .key("description")
        .font(sans(metrics::BODY_FONT, 400.0, metrics::BODY_LINE))
        .color(skin.description.rune())
        .wrap()
}

macro_rules! content_builders {
    ($ty:ident) => {
        impl $ty {
            pub fn title(mut self, title: impl Into<String>) -> Self {
                self.content.title = Some(title.into());
                self
            }

            pub fn description(mut self, description: impl Into<String>) -> Self {
                self.content.description = Some(description.into());
                self
            }

            /// Body content, between the header and the footer.
            pub fn child(mut self, child: El) -> Self {
                self.content.body.push(child);
                self
            }

            /// A button or other element in the footer.
            pub fn footer(mut self, child: El) -> Self {
                self.content.footer.push(child);
                self
            }

            /// The X in the corner (`showCloseButton`), on by default.
            pub fn close_button(mut self, on: bool) -> Self {
                self.content.close_button = on;
                self
            }
        }
    };
}

/// A modal dialog over a dimmed, blurred backdrop. Mount the result over the window (an
/// overlay layer) while the dialog is open: it traps focus, Escape, the close button and a
/// press outside call `on_close`, and focus returns to the opener when it unmounts.
///
/// ```ignore
/// if s.renaming {
///     layer.push(Dialog::new("rename").title("Rename workflow").child(field)
///         .footer(Button::new("Save").build(t)).build(t, |s: &mut App| s.renaming = false));
/// }
/// ```
pub struct Dialog {
    content: Content,
    max_width: f32,
}

content_builders!(Dialog);

impl Dialog {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            content: Content::new(key.into()),
            max_width: metrics::DIALOG_WIDTH,
        }
    }

    /// The widest the dialog gets (`sm:max-w-sm`, 384, by default).
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }

    pub fn build<S: 'static>(self, theme: &Theme, on_close: impl Fn(&mut S) + 'static) -> El {
        let skin = ModalSkin::new(theme);
        let close: Close<S> = Rc::new(on_close);
        let c = self.content;
        let mut surface = col()
            .key(format!("{}-dialog", c.key))
            .role("dialog")
            .label(c.title.clone().unwrap_or_default())
            .w(self.max_width)
            .gap(metrics::DIALOG_GAP)
            .padding(metrics::DIALOG_PADDING)
            .radius(metrics::DIALOG_RADIUS)
            .fill(skin.surface.rune())
            .box_shadow(Shadow::drop(0.0, 0.0, 1.0, skin.ring).rune());
        if c.title.is_some() || c.description.is_some() {
            let mut header = col().key("header").gap(metrics::HEADER_GAP);
            if let Some(t) = &c.title {
                header = header.child(title(t, metrics::DIALOG_TITLE_LINE, theme).pr(32.0));
            }
            if let Some(d) = &c.description {
                header = header.child(description(d, &skin));
            }
            surface = surface.child(header);
        }
        surface = surface.children(c.body);
        if !c.footer.is_empty() {
            surface = surface.child(
                row()
                    .key("footer")
                    .mx(-metrics::DIALOG_PADDING)
                    .mb(-metrics::DIALOG_PADDING)
                    .padding(metrics::DIALOG_PADDING)
                    .gap(metrics::FOOTER_GAP)
                    .justify_end()
                    .fill(skin.footer.rune())
                    .border_sides([1.0, 0.0, 0.0, 0.0], skin.footer_border.rune())
                    .children(c.footer),
            );
        }
        if c.close_button {
            surface = surface.child(close_button(theme, &close, metrics::DIALOG_CLOSE_INSET));
        }
        let dismiss = close.clone();
        let surface = surface
            .clip_rounded(metrics::DIALOG_RADIUS)
            .modal(true, move |s: &mut S| dismiss(s))
            .transition(Transition {
                insertion: Effect {
                    opacity: 0.0,
                    scale: DIALOG_MOTION.scale,
                    ..Effect::IDENTITY
                },
                removal: Effect {
                    opacity: 0.0,
                    scale: DIALOG_MOTION.scale,
                    ..Effect::IDENTITY
                },
                pop_layout: true,
                springs: SpringPair::new(spring(DIALOG_MOTION.enter), spring(DIALOG_MOTION.exit)),
            });
        el().key(c.key.clone())
            .absolute(0.0, 0.0)
            .w_full()
            .h_full()
            .child(backdrop(&c.key, &skin, DIALOG_BACKDROP))
            .child(
                col()
                    .key("center")
                    .absolute(0.0, 0.0)
                    .w_full()
                    .h_full()
                    .padding(metrics::DIALOG_MARGIN)
                    .center_y()
                    .items_center()
                    .justify_center()
                    .child(surface),
            )
    }
}

/// A modal side panel sliding in from an edge over the dimmed backdrop, with a header, body
/// and a footer pinned to its end. Mount it like [`Dialog`].
pub struct Sheet {
    content: Content,
    side: Side,
}

content_builders!(Sheet);

impl Sheet {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            content: Content::new(key.into()),
            side: Side::Right,
        }
    }

    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    pub fn build<S: 'static>(self, theme: &Theme, on_close: impl Fn(&mut S) + 'static) -> El {
        let skin = ModalSkin::new(theme);
        let close: Close<S> = Rc::new(on_close);
        let c = self.content;
        let vertical = self.side.is_vertical();
        let mut panel = col()
            .key(format!("{}-sheet", c.key))
            .role("dialog")
            .label(c.title.clone().unwrap_or_default())
            .gap(metrics::SHEET_GAP)
            .fill(skin.surface.rune());
        panel = if vertical {
            panel.w_full()
        } else {
            panel.h_full().w(metrics::SHEET_WIDTH)
        };
        let border = match self.side {
            Side::Right => [0.0, 0.0, 0.0, 1.0],
            Side::Left => [0.0, 1.0, 0.0, 0.0],
            Side::Top => [0.0, 0.0, 1.0, 0.0],
            Side::Bottom => [1.0, 0.0, 0.0, 0.0],
        };
        panel = panel.border_sides(border, skin.border.rune());
        for s in &skin.sheet_shadows {
            panel = panel.box_shadow(s.rune());
        }
        if c.title.is_some() || c.description.is_some() {
            let mut header = col()
                .key("header")
                .gap(metrics::SHEET_HEADER_GAP)
                .padding(metrics::SHEET_PADDING);
            if let Some(t) = &c.title {
                header = header.child(title(t, metrics::SHEET_TITLE_LINE, theme));
            }
            if let Some(d) = &c.description {
                header = header.child(description(d, &skin));
            }
            panel = panel.child(header);
        }
        panel = panel.children(c.body);
        if !c.footer.is_empty() {
            panel = panel.child(el().key("spacer").grow(1.0)).child(
                col()
                    .key("footer")
                    .gap(metrics::FOOTER_GAP)
                    .padding(metrics::SHEET_PADDING)
                    .items_stretch()
                    .children(c.footer),
            );
        }
        if c.close_button {
            panel = panel.child(close_button(theme, &close, metrics::SHEET_CLOSE_INSET));
        }
        let (dx, dy) = self.side.direction();
        let travel = SHEET_MOTION.enter_travel;
        let effect = Effect {
            opacity: 0.0,
            offset: Vec2::new(dx * travel, dy * travel),
            ..Effect::IDENTITY
        };
        let dismiss = close.clone();
        let panel = panel
            .modal(true, move |s: &mut S| dismiss(s))
            .transition(Transition {
                insertion: effect,
                removal: effect,
                pop_layout: true,
                springs: SpringPair::new(spring(SHEET_MOTION.enter), spring(SHEET_MOTION.exit)),
            });
        let place = match self.side {
            Side::Right => row().justify_end(),
            Side::Left => row(),
            Side::Top => col(),
            Side::Bottom => col().justify_end(),
        }
        .key("edge")
        .absolute(0.0, 0.0)
        .w_full()
        .h_full();
        el().key(c.key.clone())
            .absolute(0.0, 0.0)
            .w_full()
            .h_full()
            .child(backdrop(&c.key, &skin, SHEET_BACKDROP))
            .child(place.child(panel))
    }
}
