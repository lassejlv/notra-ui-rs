//! Rune rendering of the dualtone card family.

use ::rune::{El, col, el, icon, row, text};

use super::data_parts::{anim, shimmer, spinner};
use super::{ToRune, sans, tween};
use crate::components::card::{
    self, CardSkin, CardVariant, HINT_TRANSITION, empty, reveal, section,
};
use crate::icons::Icon;
use crate::theme::{Color, Shadow, Theme};

fn eyebrow(theme: &Theme, title: &str, hint: Option<&str>, size: f32, line_height: f32) -> El {
    let skin = CardSkin::new(theme, CardVariant::Panel);
    let mut e = row().key("eyebrow").center_y().gap(card::HINT_GAP).child(
        text(title)
            .key("eyebrow-title")
            .font(sans(size, 500.0, line_height))
            .color(skin.title.rune())
            .truncate(),
    );
    if let Some(hint) = hint {
        let fade = tween(HINT_TRANSITION);
        e = e.child(
            el().key("eyebrow-hint")
                .role("button")
                .label(card::MORE_INFO)
                .help(hint)
                .hover_group()
                .hover_motion(fade, fade)
                .child(
                    icon(Icon::HugeInformationCircle.rune(), card::HINT_ICON)
                        .color(skin.hint.rune())
                        .group_color(skin.hint_hover.rune()),
                ),
        );
    }
    e
}

fn readout(theme: &Theme, text_: &str) -> El {
    text(text_)
        .key("readout")
        .font(sans(card::READOUT_SIZE, 400.0, card::READOUT_LINE_HEIGHT))
        .color(theme.muted_foreground.rune())
        .truncate()
}

/// Notra's `InstrumentModule`: `Card::new("Mention rate").variant(CardVariant::Table).build(&theme)`.
pub struct Card {
    eyebrow: String,
    variant: CardVariant,
    bare_body: bool,
    description: Option<String>,
    hint: Option<String>,
    readout: Option<String>,
    action: Option<El>,
    children: Vec<El>,
}

impl Card {
    pub fn new(eyebrow: impl Into<String>) -> Self {
        Self {
            eyebrow: eyebrow.into(),
            variant: CardVariant::Flat,
            bare_body: false,
            description: None,
            hint: None,
            readout: None,
            action: None,
            children: Vec::new(),
        }
    }

    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Drop the shell and the lifted body: the header sits over bare content.
    pub fn bare_body(mut self, bare: bool) -> Self {
        self.bare_body = bare;
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Explains the module on an info icon after the eyebrow.
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn readout(mut self, readout: impl Into<String>) -> Self {
        self.readout = Some(readout.into());
        self
    }

    pub fn action(mut self, action: El) -> Self {
        self.action = Some(action);
        self
    }

    pub fn child(mut self, child: El) -> Self {
        self.children.push(child);
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let variant = self.variant;
        let m = variant.metrics(self.bare_body);
        let skin = CardSkin::new(theme, variant);
        let flat = !variant.is_dualtone();
        let bare = self.bare_body && !flat;
        let mut title = col()
            .key("card-title")
            .grow(1.0)
            .min_w(0.0)
            .gap(m.header_gap)
            .child(eyebrow(
                theme,
                &self.eyebrow,
                self.hint.as_deref(),
                m.title_size,
                m.title_line_height,
            ));
        if let Some(d) = &self.description {
            title = title.child(
                text(d.as_str())
                    .font(sans(14.0, 400.0, 20.0))
                    .color(skin.description.rune())
                    .wrap(),
            );
        }
        let mut head_row = row().gap(4.0).child(title);
        head_row = if variant.has_table_header() || flat {
            head_row.center_y()
        } else {
            head_row.items_start()
        };
        if self.readout.is_some() || self.action.is_some() {
            let mut actions = row().key("card-action").center_y().gap(card::ACTION_GAP);
            if let Some(r) = &self.readout {
                let r = if flat {
                    card::capitalize_first(r)
                } else {
                    r.clone()
                };
                actions = actions.child(readout(theme, &r));
            }
            if let Some(a) = self.action {
                actions = actions.child(a);
            }
            head_row = head_row.child(actions);
        }
        let mut header = col()
            .key("card-header")
            .min_h(m.header_min_height)
            .pt(m.header_padding_top)
            .pb(m.header_padding_bottom)
            .px(m.header_padding_x)
            .child(head_row);
        if m.header_centered {
            header = header.justify_center();
        }
        let mut body = col()
            .key("card-body")
            .grow(1.0)
            .min_w(0.0)
            .padding(m.body_padding)
            .px(m.body_padding_x)
            .children(self.children);
        if m.body_overlap > 0.0 {
            body = body.mt(-m.body_overlap);
        }
        if variant.is_dualtone() && !bare {
            body = body
                .radius(m.body_radius)
                .border(1.0, skin.body_border.rune())
                .fill(skin.body.rune());
            for s in &skin.body_shadows {
                body = body.box_shadow(s.rune());
            }
        }
        let mut e = col()
            .grow(1.0)
            .min_w(0.0)
            .gap(m.gap)
            .py(m.padding_y)
            .radius(m.radius)
            .child(header)
            .child(body);
        if flat {
            e = e
                .fill(skin.shell.rune())
                .box_shadow(Shadow::drop(0.0, 0.0, 1.0, skin.shell_border).rune());
        } else if !bare {
            e = e
                .padding(m.shell_padding)
                .border(1.0, skin.shell_border.rune())
                .fill(skin.shell.rune());
        }
        e
    }
}

/// Notra's `InstrumentSection`: a heading with readout and actions over bare content.
pub struct CardSection {
    title: String,
    description: Option<String>,
    hint: Option<String>,
    readout: Option<String>,
    action: Option<El>,
    children: Vec<El>,
}

impl CardSection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            hint: None,
            readout: None,
            action: None,
            children: Vec::new(),
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn readout(mut self, readout: impl Into<String>) -> Self {
        self.readout = Some(readout.into());
        self
    }

    pub fn action(mut self, action: El) -> Self {
        self.action = Some(action);
        self
    }

    pub fn child(mut self, child: El) -> Self {
        self.children.push(child);
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let has_action = self.readout.is_some() || self.action.is_some();
        let described = self.description.is_some();
        let tight = !described && has_action;
        let mut heading = col()
            .key("section-heading")
            .min_w(0.0)
            .gap(section::TITLE_GAP)
            .child(eyebrow(
                theme,
                &self.title,
                self.hint.as_deref(),
                14.0,
                if tight { 14.0 } else { 20.0 },
            ));
        if tight {
            heading = heading.h(section::ROW_HEIGHT).justify_center();
        }
        if let Some(d) = &self.description {
            heading = heading.child(
                text(d.as_str())
                    .font(sans(14.0, 400.0, 20.0))
                    .color(theme.muted_foreground.rune())
                    .wrap(),
            );
        }
        let mut top = row()
            .key("section-top")
            .flex_wrap()
            .justify_between()
            .gap(section::HEADER_GAP)
            .min_w(0.0)
            .child(heading);
        top = if described {
            top.items_start()
        } else {
            top.center_y()
        };
        if has_action {
            let mut actions = row()
                .key("section-actions")
                .flex_wrap()
                .center_y()
                .gap(card::ACTION_GAP);
            if let Some(r) = &self.readout {
                actions = actions.child(readout(theme, &card::capitalize_first(r)));
            }
            if let Some(a) = self.action {
                actions = actions.child(a);
            }
            top = top.child(actions);
        }
        col().gap(section::GAP).min_w(0.0).child(top).child(
            col()
                .key("section-body")
                .grow(1.0)
                .min_w(0.0)
                .children(self.children),
        )
    }
}

/// Notra's `InstrumentEmpty`: a centred message, a spinner and shimmer while busy, an
/// optional action and an optional faded preview behind.
pub struct CardEmpty {
    message: String,
    busy: bool,
    action: Option<El>,
    preview: Option<El>,
}

impl CardEmpty {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            busy: false,
            action: None,
            preview: None,
        }
    }

    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn action(mut self, action: El) -> Self {
        self.action = Some(action);
        self
    }

    pub fn preview(mut self, preview: El) -> Self {
        self.preview = Some(preview);
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let mut e = col()
            .key("empty")
            .grow(1.0)
            .min_h(empty::MIN_HEIGHT)
            .center()
            .gap(empty::GAP)
            .role("status")
            .busy(self.busy);
        let has_preview = self.preview.is_some();
        if let Some(preview) = self.preview {
            let surface = theme.table_card;
            let fade = |from: Color, to: Color| el().w_full().gradient(from.rune(), to.rune());
            let veil = col()
                .key("empty-veil")
                .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .child(fade(surface, surface.fade(0.0)).grow(24.0))
                .child(el().grow(46.0))
                .child(fade(surface.fade(0.0), surface).grow(30.0));
            e = e.clip_rounded(0.0).child(
                el().key("empty-preview")
                    .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                    .opacity(empty::PREVIEW_OPACITY)
                    .inert()
                    .a11y_hidden()
                    .child(preview)
                    .child(veil),
            );
        }
        if !self.message.is_empty() || self.busy {
            let message = card::capitalize_first(&self.message);
            let mut line = row()
                .key("empty-message")
                .center_y()
                .gap(empty::MESSAGE_GAP);
            if self.busy {
                line = line.child(spinner(theme.muted_foreground));
            }
            if !message.is_empty() {
                line = line.child(if self.busy {
                    shimmer(
                        &message,
                        14.0,
                        20.0,
                        theme.muted_foreground,
                        theme.foreground,
                    )
                } else {
                    text(message)
                        .font(sans(14.0, 400.0, 20.0))
                        .color(theme.muted_foreground.rune())
                });
            }
            if has_preview {
                line = line
                    .px(empty::PILL_PADDING_X)
                    .py(empty::PILL_PADDING_Y)
                    .radius(999.0)
                    .fill(theme.background.fade(empty::PILL_ALPHA).rune())
                    .box_shadow(Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x0000000d)).rune());
            }
            e = e.child(line);
        }
        if let Some(action) = self.action.filter(|_| !self.busy) {
            e = e.child(action);
        }
        e
    }
}

/// Notra's `InstrumentReveal`: content rises 6 px and fades in when `active` turns on,
/// `order` × 45 ms after its siblings.
pub fn card_reveal(active: bool, order: u32, child: El) -> El {
    let motion = anim(reveal::tween(order));
    let (opacity, offset) = reveal::frame(order, active, None);
    col()
        .min_w(0.0)
        .opacity(opacity)
        .offset(0.0, offset)
        .opacity_motion(motion)
        .offset_motion(motion)
        .child(child)
}

/// Notra's `InstrumentGrid`: equal columns 12 px apart.
pub fn card_grid(columns: usize, children: Vec<El>) -> El {
    let columns = columns.max(1);
    let mut rows = col().gap(card::GRID_GAP).w_full();
    let mut iter = children.into_iter().peekable();
    let mut index = 0;
    while iter.peek().is_some() {
        let mut line = row()
            .key(format!("grid-row-{index}"))
            .gap(card::GRID_GAP)
            .items_stretch();
        for _ in 0..columns {
            line = line.child(match iter.next() {
                Some(child) => col().grow(1.0).min_w(0.0).child(child),
                None => el().grow(1.0),
            });
        }
        rows = rows.child(line);
        index += 1;
    }
    rows
}
