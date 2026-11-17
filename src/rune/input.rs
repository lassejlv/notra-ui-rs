//! Text entry for Rune: shadcn's [`Input`], [`Textarea`] and [`InputGroup`] around Rune's
//! `text_input`, which brings the caret, selection, IME, clipboard and undo.

use ::rune::{El, ScrollSpec, TextEdit, Tx, col, icon, row, text, text_input};

use super::{ToRune, sans, tween};
use crate::components::input::{
    AddonAlign, DISABLED_OPACITY, FieldMetrics, FieldSkin, INPUT, TEXTAREA, TRANSITION, group,
};
use crate::icons::Icon;
use crate::theme::{Color, Theme};

pub(crate) type Wire = Box<dyn FnOnce(El) -> El>;

/// A `text_input` with the field's type, colours and padding; multi-line fields grow between
/// the metrics' bounds and then scroll.
pub(crate) struct Field {
    el: El,
    multiline: bool,
    placeholder: String,
    label: Option<String>,
    disabled: bool,
    newline: Option<Wire>,
    submits: bool,
}

impl Field {
    pub(crate) fn new(edit: &TextEdit, multiline: bool) -> Self {
        Self {
            el: text_input(edit, sans(14.0, 400.0, 20.0)),
            multiline,
            placeholder: String::new(),
            label: None,
            disabled: false,
            newline: None,
            submits: false,
        }
    }

    pub(crate) fn on_edit<S: 'static>(
        mut self,
        f: impl for<'a> Fn(&'a mut S) -> &'a mut TextEdit + 'static,
    ) -> Self {
        let lens = std::rc::Rc::new(f);
        let edit = lens.clone();
        self.el = self
            .el
            .on_edit(::rune::edit_access(move |s: &mut S| edit(s)));
        if self.multiline {
            self.newline = Some(Box::new(move |e: El| {
                e.on_submit(move |s: &mut S, _| lens(s).insert("\n"))
            }));
        }
        self
    }

    pub(crate) fn on_submit<S: 'static>(mut self, f: impl Fn(&mut S, &mut Tx) + 'static) -> Self {
        self.el = self.el.on_submit(f);
        self.submits = true;
        self
    }

    pub(crate) fn placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    pub(crate) fn label(&mut self, label: impl Into<String>) {
        self.label = Some(label.into());
    }

    pub(crate) fn disable(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub(crate) fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// The field inside `padding` (top, right, bottom, left).
    pub(crate) fn build(self, theme: &Theme, m: &FieldMetrics, padding: [f32; 4]) -> El {
        let skin = FieldSkin::new(theme);
        let [top, right, bottom, left] = padding;
        let multiline = self.multiline;
        let el = match self.newline.filter(|_| !self.submits) {
            Some(newline) => newline(self.el),
            None => self.el,
        };
        let mut field = el
            .key("field")
            .font(sans(m.font_size, 400.0, m.line_height))
            .color(skin.text.rune())
            .placeholder(self.placeholder.clone(), skin.placeholder.rune())
            .caret(skin.caret.rune(), skin.selection.rune())
            .label(self.label.unwrap_or(self.placeholder))
            .disabled(self.disabled)
            .w_full()
            .pt(top)
            .pr(right)
            .pb(bottom)
            .pl(left);
        if !multiline {
            return field.single_line();
        }
        let chrome = 2.0 * m.border;
        field = field.min_h(m.min_height - chrome);
        let mut scroll = col()
            .key("scroll")
            .w_full()
            .shrink()
            .min_h(m.min_height - chrome);
        if let Some(max) = m.max_height {
            scroll = scroll.max_h(max - chrome);
        }
        scroll
            .scroll(ScrollSpec {
                stick_bottom: false,
            })
            .child(field.animate_edit_layout())
    }
}

/// The padding of a field: a single line sits in the middle of its fixed height, as browsers
/// centre an `<input>`'s text.
pub(crate) fn padding(m: &FieldMetrics, multiline: bool) -> [f32; 4] {
    let y = if multiline {
        m.padding_y
    } else {
        ((m.min_height - 2.0 * m.border - m.line_height) / 2.0).max(0.0)
    };
    [y, m.padding_x, y, m.padding_x]
}

/// The bordered box shadcn fields share: the border turns `ring` and a 3 px halo shows while
/// focus is inside.
fn field_box(theme: &Theme, radius: f32, disabled: bool, child: El) -> El {
    let skin = FieldSkin::new(theme);
    let fill = if disabled {
        skin.disabled_background
    } else {
        skin.background
    };
    let inner = col()
        .key("box")
        .w_full()
        .radius(radius)
        .border(1.0, skin.border.rune())
        .fill(fill.rune())
        .focus_within_ring(1.0, -1.0, skin.focus_border.rune())
        .motion(::rune_motion::Spring::new(
            TRANSITION.duration.as_secs_f32(),
            0.0,
        ))
        .child(child);
    let outer = col()
        .w_full()
        .radius(radius)
        .focus_within_ring(skin.focus_ring.spread, 0.0, skin.focus_ring.color.rune())
        .child(inner);
    if disabled {
        outer.opacity(DISABLED_OPACITY)
    } else {
        outer
    }
}

macro_rules! field_builders {
    () => {
        pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
            self.field.placeholder(placeholder);
            self
        }

        /// The accessible name, when there is no visible label (`aria-label`).
        pub fn label(mut self, label: impl Into<String>) -> Self {
            self.field.label(label);
            self
        }

        pub fn disabled(mut self, disabled: bool) -> Self {
            self.field.disable(disabled);
            self
        }

        /// Routes edits back into the app's `TextEdit`.
        pub fn on_edit<S: 'static>(
            mut self,
            f: impl for<'a> Fn(&'a mut S) -> &'a mut TextEdit + 'static,
        ) -> Self {
            self.field = self.field.on_edit(f);
            self
        }

        /// Called on Return. Without it a multi-line field breaks the line on Return.
        pub fn on_submit<S: 'static>(
            mut self,
            f: impl Fn(&mut S, &mut ::rune::Tx) + 'static,
        ) -> Self {
            self.field = self.field.on_submit(f);
            self
        }
    };
}

pub(crate) use field_builders;

/// shadcn's `Input`: `Input::new(&s.email).placeholder("Email").on_edit(|s: &mut S| &mut s.email)`.
pub struct Input {
    field: Field,
}

impl Input {
    pub fn new(edit: &TextEdit) -> Self {
        Self {
            field: Field::new(edit, false),
        }
    }

    field_builders!();

    pub fn build(self, theme: &Theme) -> El {
        let disabled = self.field.is_disabled();
        let field = self.field.build(theme, &INPUT, padding(&INPUT, false));
        field_box(theme, INPUT.radius, disabled, field)
    }
}

/// shadcn's `Textarea`: wraps and grows with its text (`field-sizing-content`). Return breaks
/// the line; Cmd/Ctrl+Return submits.
pub struct Textarea {
    field: Field,
    metrics: FieldMetrics,
}

impl Textarea {
    pub fn new(edit: &TextEdit) -> Self {
        Self {
            field: Field::new(edit, true),
            metrics: TEXTAREA,
        }
    }

    field_builders!();

    /// Caps the growth: the field scrolls past `max_height`.
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.metrics.max_height = Some(max_height);
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let disabled = self.field.is_disabled();
        let field = self
            .field
            .build(theme, &self.metrics, padding(&self.metrics, true));
        field_box(theme, self.metrics.radius, disabled, field)
    }
}

/// A field with icons, text or buttons inside its box (shadcn's `InputGroup`).
pub struct InputGroup {
    field: Field,
    multiline: bool,
    addons: Vec<InputGroupAddon>,
}

impl InputGroup {
    /// A single-line group around `edit`.
    pub fn new(edit: &TextEdit) -> Self {
        Self {
            field: Field::new(edit, false),
            multiline: false,
            addons: Vec::new(),
        }
    }

    /// A group around a growing textarea (`InputGroupTextarea`).
    pub fn textarea(edit: &TextEdit) -> Self {
        Self {
            field: Field::new(edit, true),
            multiline: true,
            addons: Vec::new(),
        }
    }

    field_builders!();

    pub fn addon(mut self, addon: InputGroupAddon) -> Self {
        self.addons.push(addon);
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let has = |align: AddonAlign| self.addons.iter().any(|a| a.align == align);
        let metrics = if self.multiline { TEXTAREA } else { INPUT };
        let column = self.multiline || has(AddonAlign::BlockStart) || has(AddonAlign::BlockEnd);
        let [mut top, mut right, mut bottom, mut left] = padding(&metrics, self.multiline);
        if has(AddonAlign::InlineStart) {
            left = group::INPUT_INSET_BESIDE_ADDON;
        }
        if has(AddonAlign::InlineEnd) {
            right = group::INPUT_INSET_BESIDE_ADDON;
        }
        if !self.multiline && has(AddonAlign::BlockStart) {
            top = group::BLOCK_ADDON_PADDING_Y;
        }
        if !self.multiline && has(AddonAlign::BlockEnd) {
            bottom += group::BLOCK_ADDON_PADDING_Y;
        }
        let metrics = FieldMetrics {
            min_height: if self.multiline {
                metrics.min_height
            } else {
                metrics.line_height + top + bottom + 2.0 * metrics.border
            },
            ..metrics
        };
        let disabled = self.field.is_disabled();
        let field = self
            .field
            .build(theme, &metrics, [top, right, bottom, left]);
        let (before, after): (Vec<_>, Vec<_>) = self
            .addons
            .into_iter()
            .partition(|a| matches!(a.align, AddonAlign::InlineStart | AddonAlign::BlockStart));
        let mut body = if column {
            col()
        } else {
            row().center_y().h(group::HEIGHT - 2.0)
        };
        body = body.w_full();
        for (i, addon) in before.into_iter().enumerate() {
            body = body.child(addon.build(theme).key(format!("addon-start-{i}")));
        }
        body = body.child(if column {
            field
        } else {
            row().grow(1.0).shrink().min_w(0.0).child(field)
        });
        for (i, addon) in after.into_iter().enumerate() {
            body = body.child(addon.build(theme).key(format!("addon-end-{i}")));
        }
        field_box(theme, group::RADIUS, disabled, body)
    }
}

enum AddonPart {
    Icon(Icon),
    Text(String),
    Button(InputGroupButton),
}

/// Text, icons or buttons beside the text of an [`InputGroup`].
pub struct InputGroupAddon {
    align: AddonAlign,
    parts: Vec<AddonPart>,
}

impl InputGroupAddon {
    pub fn new(align: AddonAlign) -> Self {
        Self {
            align,
            parts: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.parts.push(AddonPart::Icon(icon));
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.parts.push(AddonPart::Text(text.into()));
        self
    }

    /// An [`InputGroupButton`]; the addon tucks into the group's edge to hold it.
    pub fn button(mut self, button: InputGroupButton) -> Self {
        self.parts.push(AddonPart::Button(button));
        self
    }

    fn build(self, theme: &Theme) -> El {
        let ink = theme.muted_foreground.rune();
        let holds_button = self.parts.iter().any(|p| matches!(p, AddonPart::Button(_)));
        let pull = if holds_button {
            group::BUTTON_PULL
        } else {
            0.0
        };
        let mut e = row()
            .center_y()
            .gap(group::ADDON_GAP)
            .py(6.0)
            .cursor(::rune::Cursor::Text);
        e = match self.align {
            AddonAlign::InlineStart => e.pl(group::ADDON_INSET).ml(-pull),
            AddonAlign::InlineEnd => e.pr(group::ADDON_INSET).mr(-pull),
            AddonAlign::BlockStart => e
                .w_full()
                .px(group::BLOCK_ADDON_PADDING_X)
                .pt(group::BLOCK_ADDON_PADDING_Y),
            AddonAlign::BlockEnd => e
                .w_full()
                .px(group::BLOCK_ADDON_PADDING_X)
                .pb(group::BLOCK_ADDON_PADDING_Y),
        };
        for (i, part) in self.parts.into_iter().enumerate() {
            e = e.child(
                match part {
                    AddonPart::Icon(i) => icon(i.rune(), group::ADDON_ICON).color(ink),
                    AddonPart::Text(t) => text(t).font(sans(14.0, 500.0, 20.0)).color(ink),
                    AddonPart::Button(b) => b.build(theme),
                }
                .key(format!("part-{i}")),
            );
        }
        e
    }
}

/// A ghost button inside an [`InputGroupAddon`] (`InputGroupButton size="xs"`, or an icon).
pub struct InputGroupButton {
    button: GhostButton,
}

impl InputGroupButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            button: GhostButton::new(Self::metrics(true)).label(label),
        }
    }

    /// A square `icon-xs` button named `label` for assistive tech.
    pub fn icon_button(icon: Icon, label: impl Into<String>) -> Self {
        Self {
            button: GhostButton::new(Self::metrics(false))
                .icon(icon)
                .aria_label(label),
        }
    }

    fn metrics(text: bool) -> GhostMetrics {
        GhostMetrics {
            height: group::BUTTON_HEIGHT,
            padding_x: text.then_some(group::BUTTON_PADDING_X),
            gap: 4.0,
            radius: group::BUTTON_RADIUS,
            font_size: 14.0,
            line_height: 20.0,
            icon: group::BUTTON_ICON,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.button = self.button.icon(icon);
        self
    }

    pub fn on_press<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.button = self.button.on_press(f);
        self
    }

    fn build(self, theme: &Theme) -> El {
        let hover = if theme.is_dark() {
            theme.muted.fade(0.5)
        } else {
            theme.muted
        };
        self.button
            .colors(theme.foreground, hover, theme.foreground)
            .build(theme)
    }
}

/// Box metrics of a ghost button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GhostMetrics {
    pub height: f32,
    pub padding_x: Option<f32>,
    pub gap: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub icon: f32,
}

/// A ghost button sized by its caller, as the composer's toolbar buttons, chip actions and
/// input-group buttons restyle the ghost `Button` with classes.
pub(crate) struct GhostButton {
    metrics: GhostMetrics,
    icon: Option<Icon>,
    label: Option<String>,
    icon_end: Option<(Icon, f32)>,
    aria_label: Option<String>,
    text: Color,
    hover_fill: Color,
    hover_text: Color,
    disabled: bool,
    press: Option<Wire>,
}

impl GhostButton {
    pub(crate) fn new(metrics: GhostMetrics) -> Self {
        Self {
            metrics,
            icon: None,
            label: None,
            icon_end: None,
            aria_label: None,
            text: Color::BLACK,
            hover_fill: Color::TRANSPARENT,
            hover_text: Color::BLACK,
            disabled: false,
            press: None,
        }
    }

    pub(crate) fn colors(mut self, text: Color, hover_fill: Color, hover_text: Color) -> Self {
        self.text = text;
        self.hover_fill = hover_fill;
        self.hover_text = hover_text;
        self
    }

    pub(crate) fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub(crate) fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub(crate) fn icon_end(mut self, icon: Icon, size: f32) -> Self {
        self.icon_end = Some((icon, size));
        self
    }

    pub(crate) fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    pub(crate) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub(crate) fn on_press<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(Box::new(move |e: El| e.on_press(f)));
        self
    }

    /// Attaches a press handler wired elsewhere.
    pub(crate) fn wire(mut self, wire: Wire) -> Self {
        self.press = Some(wire);
        self
    }

    pub(crate) fn build(self, theme: &Theme) -> El {
        let m = self.metrics;
        let interactive = !self.disabled;
        let ink = self.text.rune();
        let hot = self.hover_text.rune();
        let glyph = |i: Icon, size: f32| {
            let g = icon(i.rune(), size).color(ink);
            if interactive { g.group_color(hot) } else { g }
        };
        let mut e = row()
            .h(m.height)
            .center_y()
            .justify_center()
            .gap(m.gap)
            .radius(m.radius)
            .border(1.0, ::rune::Color::TRANSPARENT)
            .role("button")
            .label(
                self.aria_label
                    .clone()
                    .or_else(|| self.label.clone())
                    .unwrap_or_default(),
            )
            .disabled(self.disabled)
            .focusable(interactive)
            .focus_ring(3.0, 0.0, theme.ring.fade(0.5).rune());
        e = match m.padding_x {
            Some(p) => e.px(p),
            None => e.w(m.height),
        };
        if let Some(i) = self.icon {
            e = e.child(glyph(i, m.icon).key("icon"));
        }
        if let Some(label) = self.label {
            let mut t = text(label)
                .key("label")
                .font(sans(m.font_size, 500.0, m.line_height))
                .color(ink);
            if interactive {
                t = t.group_color(hot);
            }
            e = e.child(t);
        }
        if let Some((i, size)) = self.icon_end {
            e = e.child(glyph(i, size).key("icon-end"));
        }
        if !interactive {
            return e.opacity(DISABLED_OPACITY);
        }
        let fade = tween(TRANSITION);
        e = e
            .hover(self.hover_fill.rune())
            .hover_motion(fade, fade)
            .hover_group()
            .cursor(::rune::Cursor::Pointer);
        match self.press {
            Some(press) => press(e),
            None => e,
        }
    }
}
