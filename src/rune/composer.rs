//! The composer for Rune: [`ComposerFrame`] and its parts, matching `composer.tsx`.

use ::rune::{Align, El, Side, TextEdit, Tx, col, icon, row, text};
use ::rune_motion::{AnimExt, LoopSpec, Spring, SpringPair, Transition};

use super::input::{Field, GhostButton, GhostMetrics, Wire, field_builders, padding};
use super::{ToRune, brightness_overlay, sans, tween};
use crate::components::button::ButtonHover;
use crate::components::composer::{
    CARD_INPUT, CARD_RADIUS, ComposerShape, ComposerSize, ComposerSkin, FLAT_INPUT, HOVER, NUDGE,
    PillTray, SendState, Tone, badge, chip, nudge, pill, send, tone_color, toolbar, toolbar_button,
    tooltip, tray,
};
use crate::components::input::DISABLED_OPACITY;
use crate::icons::Icon;
use crate::theme::Theme;

fn nudge_spring() -> Spring {
    Spring::new(NUDGE.duration.as_secs_f32(), 0.0)
}

/// The composer: a dualtone card (or a pill with `flat`) holding a [`ComposerInput`] and a
/// [`ComposerToolbar`], with an optional tray of queued messages, attachments or a notice
/// tucked under its top edge. The tray fades in as the card slides down to make room.
pub struct ComposerFrame {
    shape: ComposerShape,
    size: ComposerSize,
    connected_top: bool,
    nudge: Option<El>,
    children: Vec<El>,
}

impl Default for ComposerFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl ComposerFrame {
    pub fn new() -> Self {
        Self {
            shape: ComposerShape::Card,
            size: ComposerSize::Default,
            connected_top: false,
            nudge: None,
            children: Vec::new(),
        }
    }

    /// A round pill with a tray only while there is a nudge (React's `flat`).
    pub fn flat(mut self) -> Self {
        self.shape = ComposerShape::Flat;
        self
    }

    /// The dense pill; pair it with a `ComposerSize::Sm` send.
    pub fn size(mut self, size: ComposerSize) -> Self {
        self.size = size;
        self
    }

    /// Drops the top radius so the frame sits flush under another surface.
    pub fn connected_top(mut self, connected: bool) -> Self {
        self.connected_top = connected;
        self
    }

    /// The tray's contents, usually a built [`ComposerNudge`].
    pub fn nudge(mut self, nudge: Option<El>) -> Self {
        self.nudge = nudge;
        self
    }

    pub fn child(mut self, child: El) -> Self {
        self.children.push(child);
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let skin = ComposerSkin::new(theme);
        let has_nudge = self.nudge.is_some();
        let connected = self.connected_top;
        let presence =
            Transition::opacity().springs(SpringPair::new(nudge_spring(), nudge_spring()));
        let frame = col()
            .w_full()
            .min_w(0.0)
            .role("group")
            .motion(nudge_spring());
        let mut card = col().key("card").w_full().min_w(0.0).border_layout();
        for (i, child) in self.children.into_iter().enumerate() {
            card = card.child(child.key(format!("part-{i}")));
        }

        if self.shape == ComposerShape::Flat {
            let t = PillTray::at(self.size, if has_nudge { 1.0 } else { 0.0 });
            let radius = pill::card_radius(self.size);
            let top = if connected { 0.0 } else { t.top_radius };
            let card_top = if connected && !has_nudge { 0.0 } else { radius };
            let card = card
                .border(1.0, skin.pill_border.rune())
                .fill(skin.pill.rune())
                .corners([card_top, card_top, radius, radius])
                .clip_corners([card_top, card_top, radius, radius]);
            return frame
                .fill(skin.pill_tray.fade(t.fill).rune())
                .padding(t.padding)
                .corners([top, top, t.bottom_radius, t.bottom_radius])
                .children(
                    self.nudge
                        .map(|n| col().key("nudge").w_full().transition(presence).child(n)),
                )
                .child(card);
        }

        let tray_el = self.nudge.map(|n| {
            let mut t = col()
                .key("nudge")
                .border_sides(
                    [if connected { 0.0 } else { 1.0 }, 1.0, 0.0, 1.0],
                    skin.tray_border.rune(),
                )
                .border_layout()
                .fill(skin.tray.rune())
                .pt(tray::PADDING_TOP)
                .pb(tray::PADDING_BOTTOM)
                .transition(presence)
                .child(n);
            if !connected {
                t = t
                    .mx(tray::INSET_X)
                    .corners([tray::RADIUS, tray::RADIUS, 0.0, 0.0]);
            }
            t
        });
        let top = if connected && !has_nudge {
            0.0
        } else {
            CARD_RADIUS
        };
        let mut card = card
            .border_sides(
                [if top == 0.0 { 0.0 } else { 1.0 }, 1.0, 1.0, 1.0],
                skin.card_border.rune(),
            )
            .fill(skin.card.rune())
            .corners([top, top, CARD_RADIUS, CARD_RADIUS])
            .clip_corners([top, top, CARD_RADIUS, CARD_RADIUS]);
        for s in skin.card_shadow {
            card = card.box_shadow(s.rune());
        }
        if has_nudge {
            card = card.mt(-tray::OVERLAP);
        }
        frame.children(tray_el).child(card)
    }
}

/// The tray's row: a title, chips, or free content, with an action at the end.
#[derive(Default)]
pub struct ComposerNudge {
    title: Option<El>,
    action: Option<El>,
    children: Vec<El>,
}

impl ComposerNudge {
    pub fn new() -> Self {
        Self::default()
    }

    /// A text title, shown when there are no children (`text-xs font-medium`).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(text(title.into()).font(sans(
            nudge::TITLE_SIZE,
            nudge::TITLE_WEIGHT,
            nudge::TITLE_LINE,
        )));
        self
    }

    /// Any title, such as a built [`ComposerNotice`].
    pub fn title_el(mut self, title: El) -> Self {
        self.title = Some(title);
        self
    }

    /// Sits at the end of the row (an Upgrade button, a close button).
    pub fn action(mut self, action: El) -> Self {
        self.action = Some(action);
        self
    }

    pub fn child(mut self, child: El) -> Self {
        self.children.push(child);
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let has_children = !self.children.is_empty();
        let has_title = self.title.is_some();
        let mut e = row()
            .w_full()
            .min_w(0.0)
            .center_y()
            .gap(nudge::GAP)
            .px(nudge::PADDING_X)
            .pt(nudge::PADDING_TOP)
            .pb(nudge::PADDING_BOTTOM);
        if has_children {
            e = e.flex_wrap();
            let mut chips = row()
                .key("chips")
                .grow(1.0)
                .shrink()
                .min_w(0.0)
                .flex_wrap()
                .center_y()
                .gap(nudge::CHIP_GAP);
            for (i, child) in self.children.into_iter().enumerate() {
                chips = chips.child(child.key(format!("chip-{i}")));
            }
            e = e.child(chips);
        } else if let Some(title) = self.title {
            e = e.child(
                title
                    .key("title")
                    .color(theme.foreground.rune())
                    .grow(1.0)
                    .shrink()
                    .min_w(0.0),
            );
        }
        if let Some(action) = self.action {
            if !has_children && !has_title {
                e = e.child(::rune::el().key("spacer").grow(1.0));
            }
            e = e.child(row().key("action").center_y().child(action));
        }
        e
    }
}

/// A notice for the tray: a tinted icon tile and its text.
pub struct ComposerNotice {
    tone: Tone,
    icon: Icon,
    text: String,
    body: bool,
}

impl ComposerNotice {
    pub fn new(tone: Tone, icon: Icon, text: impl Into<String>) -> Self {
        Self {
            tone,
            icon,
            text: text.into(),
            body: false,
        }
    }

    /// Body text (`text-sm`, truncated) for a notice placed as the nudge's content rather than
    /// its title.
    pub fn body(mut self) -> Self {
        self.body = true;
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let tone = tone_color(theme, self.tone);
        let font = if self.body {
            sans(nudge::BODY_SIZE, 400.0, nudge::BODY_LINE)
        } else {
            sans(nudge::TITLE_SIZE, nudge::TITLE_WEIGHT, nudge::TITLE_LINE)
        };
        row()
            .min_w(0.0)
            .center_y()
            .gap(badge::GAP)
            .child(
                row()
                    .key("badge")
                    .size(badge::SIZE, badge::SIZE)
                    .center_y()
                    .justify_center()
                    .radius(badge::RADIUS)
                    .continuous_corners()
                    .fill(tone.fade(badge::TINT).rune())
                    .child(icon(self.icon.rune(), badge::ICON).color(tone.rune())),
            )
            .child(
                text(self.text)
                    .key("text")
                    .font(font)
                    .color(theme.foreground.rune())
                    .shrink()
                    .min_w(0.0)
                    .truncate(),
            )
    }
}

/// What leads a chip's label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipLead {
    /// A muted `size-3.5` icon (a file).
    Icon(Icon),
    /// A `size-3` spinner, for an upload in progress.
    Spinner,
}

/// A queued message or an attachment in the tray: a dashed chip with steer, edit and remove
/// actions.
pub struct ComposerChip {
    label: String,
    lead: Option<ChipLead>,
    pending: bool,
    preview: Option<Wire>,
    steer: Option<Wire>,
    edit: Option<Wire>,
    remove: Option<Wire>,
}

fn press<S: 'static>(f: impl Fn(&mut S) + 'static) -> Option<Wire> {
    Some(Box::new(move |e: El| e.on_press(f)))
}

impl ComposerChip {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            lead: None,
            pending: false,
            preview: None,
            steer: None,
            edit: None,
            remove: None,
        }
    }

    pub fn lead(mut self, lead: ChipLead) -> Self {
        self.lead = Some(lead);
        self
    }

    /// Still uploading: fainter, without steer or edit.
    pub fn pending(mut self, pending: bool) -> Self {
        self.pending = pending;
        self
    }

    /// Makes the label a button (a preview).
    pub fn on_click<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.preview = press(f);
        self
    }

    /// Sends the queued message now.
    pub fn on_steer<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.steer = press(f);
        self
    }

    /// Moves the message back into the input.
    pub fn on_edit<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.edit = press(f);
        self
    }

    pub fn on_remove<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.remove = press(f);
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let skin = ComposerSkin::new(theme);
        let ink = if self.pending { skin.muted } else { skin.text };
        let mut content = row().key("content").min_w(0.0).center_y().gap(chip::GAP);
        if let Some(lead) = self.lead {
            content = content.child(
                match lead {
                    ChipLead::Icon(i) => icon(i.rune(), chip::ICON).color(skin.muted.rune()),
                    ChipLead::Spinner => spinner(chip::SPINNER, ink),
                }
                .key("lead"),
            );
        }
        content = content.child(
            text(self.label.clone())
                .key("label")
                .font(sans(chip::FONT_SIZE, 400.0, chip::LINE_HEIGHT))
                .color(ink.rune())
                .max_w(chip::LABEL_MAX_WIDTH)
                .truncate(),
        );
        if let Some(preview) = self.preview {
            content = preview(
                content
                    .role("button")
                    .label(format!("Preview {}", self.label))
                    .focusable(true)
                    .radius(chip::ACTION_RADIUS)
                    .focus_ring(3.0, 0.0, theme.ring.fade(0.5).rune())
                    .cursor(::rune::Cursor::Pointer),
            );
        }
        let metrics = GhostMetrics {
            height: chip::ACTION,
            padding_x: None,
            gap: 0.0,
            radius: chip::ACTION_RADIUS,
            font_size: 14.0,
            line_height: 20.0,
            icon: chip::ACTION_ICON,
        };
        let action = |key: &str, icon: Icon, label: String, wire: Option<Wire>| {
            wire.map(|wire| {
                GhostButton::new(metrics)
                    .icon(icon)
                    .aria_label(label)
                    .colors(skin.muted, skin.chip_action_hover, skin.hover_text)
                    .wire(wire)
                    .build(theme)
                    .key(key.to_owned())
            })
        };
        let pending = self.pending;
        let label = self.label;
        row()
            .center_y()
            .gap(chip::GAP)
            .shrink()
            .min_w(0.0)
            .radius(chip::RADIUS)
            .border(
                1.0,
                if pending {
                    skin.chip_pending_border
                } else {
                    skin.chip_border
                }
                .rune(),
            )
            .border_dash(chip::DASH, chip::DASH_GAP)
            .border_layout()
            .fill(skin.chip.rune())
            .pl(chip::PADDING_LEFT)
            .pr(chip::PADDING_RIGHT)
            .py(chip::PADDING_Y)
            .busy(pending)
            .child(content)
            .children(action(
                "steer",
                Icon::ArrowUp,
                format!("Send {label} now"),
                self.steer.filter(|_| !pending),
            ))
            .children(action(
                "edit",
                Icon::Pencil,
                format!("Edit {label}"),
                self.edit.filter(|_| !pending),
            ))
            .children(action(
                "remove",
                Icon::X,
                format!("Remove {label}"),
                self.remove,
            ))
    }
}

/// The composer's textarea: no box of its own, `text-sm leading-6` in the card and
/// `leading-5` in the pill, growing to 200 px (128 px) before it scrolls. Return breaks the
/// line unless `on_submit` is set.
pub struct ComposerInput {
    field: Field,
    shape: ComposerShape,
}

impl ComposerInput {
    pub fn new(edit: &TextEdit) -> Self {
        Self {
            field: Field::new(edit, true),
            shape: ComposerShape::Card,
        }
    }

    /// The pill's metrics.
    pub fn flat(mut self) -> Self {
        self.shape = ComposerShape::Flat;
        self
    }

    field_builders!();

    pub fn build(self, theme: &Theme) -> El {
        let metrics = match self.shape {
            ComposerShape::Card => CARD_INPUT,
            ComposerShape::Flat => FLAT_INPUT,
        };
        let disabled = self.field.is_disabled();
        let mut e = self.field.build(theme, &metrics, padding(&metrics, true));
        if self.shape == ComposerShape::Flat {
            e = e.grow(1.0).shrink().min_w(0.0);
        }
        if disabled {
            e.opacity(DISABLED_OPACITY)
        } else {
            e
        }
    }
}

/// The row under the input: `flex items-center gap-1 px-2 pt-1 pb-2`, with leading children
/// and a trailing group pushed to the end (`ml-auto`).
#[derive(Default)]
pub struct ComposerToolbar {
    leading: Vec<El>,
    trailing: Vec<El>,
}

impl ComposerToolbar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child(mut self, child: El) -> Self {
        self.leading.push(child);
        self
    }

    /// A control in the group at the row's end.
    pub fn trailing(mut self, child: El) -> Self {
        self.trailing.push(child);
        self
    }

    pub fn build(self) -> El {
        let mut e = row()
            .w_full()
            .center_y()
            .gap(toolbar::GAP)
            .px(toolbar::PADDING_X)
            .pt(toolbar::PADDING_TOP)
            .pb(toolbar::PADDING_BOTTOM);
        for (i, child) in self.leading.into_iter().enumerate() {
            e = e.child(child.key(format!("lead-{i}")));
        }
        e = e.child(::rune::el().key("spacer").grow(1.0));
        let mut end = row().key("end").center_y().gap(toolbar::GAP);
        for (i, child) in self.trailing.into_iter().enumerate() {
            end = end.child(child.key(format!("end-{i}")));
        }
        e.child(end)
    }
}

/// A muted ghost button in the toolbar: a model or effort picker, or an icon action.
pub struct ComposerToolbarButton {
    button: GhostButton,
}

impl ComposerToolbarButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            button: GhostButton::new(Self::metrics(true)).label(label),
        }
    }

    /// A square `icon-sm` button named `label` for assistive tech.
    pub fn icon_button(icon: Icon, label: impl Into<String>) -> Self {
        Self {
            button: GhostButton::new(Self::metrics(false))
                .icon(icon)
                .aria_label(label),
        }
    }

    fn metrics(text: bool) -> GhostMetrics {
        GhostMetrics {
            height: toolbar_button::HEIGHT,
            padding_x: text.then_some(toolbar_button::PADDING_X),
            gap: toolbar_button::GAP,
            radius: toolbar_button::RADIUS,
            font_size: toolbar_button::FONT_SIZE,
            line_height: toolbar_button::LINE_HEIGHT,
            icon: if text {
                toolbar_button::ICON
            } else {
                toolbar_button::ICON_ONLY_ICON
            },
        }
    }

    /// An icon before the label.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.button = self.button.icon(icon);
        self
    }

    /// A `size-3` chevron after the label, for pickers.
    pub fn chevron(mut self) -> Self {
        self.button = self
            .button
            .icon_end(Icon::ChevronDown, toolbar_button::CHEVRON);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.button = self.button.disabled(disabled);
        self
    }

    pub fn on_press<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.button = self.button.on_press(f);
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let skin = ComposerSkin::new(theme);
        self.button
            .colors(skin.muted, skin.hover_fill, skin.hover_text)
            .build(theme)
    }
}

/// The send button: violet when there is something to send, the dark Depth button when
/// there is not, a spinner while busy. Stop with `Icon::SquareSolid`.
pub struct ComposerSend {
    key: String,
    label: String,
    icon: Icon,
    busy: bool,
    disabled: bool,
    size: ComposerSize,
    round: bool,
    tooltip: Option<String>,
    press: Option<Wire>,
}

impl ComposerSend {
    /// `key` keys the button (its tooltip anchors to it); `label` names the action.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            icon: Icon::ArrowUp,
            busy: false,
            disabled: false,
            size: ComposerSize::Default,
            round: false,
            tooltip: None,
            press: None,
        }
    }

    /// `Icon::ArrowUp` (the default) or `Icon::SquareSolid` to stop.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }

    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: ComposerSize) -> Self {
        self.size = size;
        self
    }

    /// A circle, as inside a `flat` frame.
    pub fn round(mut self, round: bool) -> Self {
        self.round = round;
        self
    }

    /// A tooltip above, shown after a hover of 600 ms.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn on_press<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = press(f);
        self
    }

    pub fn on_press_tx<S: 'static>(mut self, f: impl Fn(&mut S, &mut Tx) + 'static) -> Self {
        self.press = Some(Box::new(move |e: El| e.on_press_tx(f)));
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let state = SendState::new(self.busy, self.disabled);
        let skin = state.skin(theme);
        let side = send::size(self.size);
        let shape = if self.round {
            ComposerShape::Flat
        } else {
            ComposerShape::Card
        };
        let ink = skin.text;
        let glyph = if self.busy {
            spinner(send::spinner(self.size), ink)
        } else {
            let size = if self.icon == Icon::SquareSolid {
                send::STOP_ICON
            } else {
                send::icon(self.size)
            };
            icon(self.icon.rune(), size).color(ink.rune())
        };
        let button_key = format!("{}-button", self.key);
        let mut e = row()
            .key(button_key.clone())
            .size(side, side)
            .center_y()
            .justify_center()
            .radius(SendState::radius(shape, self.size))
            .border(1.0, skin.border.rune())
            .fill(skin.fill.rune())
            .role("button")
            .label(self.label)
            .busy(self.busy)
            .focusable(true)
            .focus_ring(3.0, 0.0, theme.ring.fade(0.5).rune())
            .opacity(state.opacity())
            .child(glyph.key("glyph"));
        if shape == ComposerShape::Card {
            e = e.continuous_corners();
        }
        if let Some((top, bottom)) = skin.gradient {
            e = e.gradient(top.rune(), bottom.rune());
        }
        for s in &skin.shadows {
            e = e.box_shadow(s.rune());
        }
        if state.is_pressable() {
            if let ButtonHover::Brightness(f) = skin.hover
                && f != 1.0
            {
                let base = skin.gradient.map_or(skin.fill, |(t, b)| t.mix(b, 0.5));
                let fade = tween(HOVER);
                e = e
                    .hover(brightness_overlay(base, f).rune())
                    .hover_motion(fade, fade);
            }
            e = e.cursor(::rune::Cursor::Pointer);
            if let Some(press) = self.press {
                e = press(e);
            }
        }
        row().key(self.key).hover_group().child(e).children(
            self.tooltip
                .map(|label| tooltip_bubble(theme, &button_key, label)),
        )
    }
}

/// The dark tooltip above a keyed trigger: it fades in 600 ms into a hover, as Base UI's does.
fn tooltip_bubble(theme: &Theme, anchor: &str, label: String) -> El {
    let skin = tooltip::TooltipSkin::new(theme);
    let enter = tween(tooltip::ENTER).delay(tooltip::DELAY.as_secs_f32());
    let exit = tween(tooltip::EXIT);
    let mut chip = row()
        .key("chip")
        .px(tooltip::PADDING_X)
        .py(tooltip::PADDING_Y)
        .radius(tooltip::RADIUS)
        .continuous_corners()
        .border(1.0, skin.border.rune())
        .border_layout()
        .gradient(skin.top.rune(), skin.bottom.rune())
        .child(
            text(label)
                .font(sans(tooltip::FONT_SIZE, 400.0, tooltip::LINE_HEIGHT))
                .color(skin.text.rune()),
        );
    for s in &skin.shadows {
        chip = chip.box_shadow(s.rune());
    }
    let arrow = ::rune::el()
        .key("arrow")
        .size(tooltip::ARROW, tooltip::ARROW)
        .radius(tooltip::ARROW_RADIUS)
        .fill(skin.arrow.rune())
        .rotate(std::f32::consts::FRAC_PI_4)
        .mt(-(tooltip::ARROW / 2.0 + tooltip::ARROW_INSET))
        .paint_order(-1);
    let below_chip = tooltip::ARROW / 2.0 - tooltip::ARROW_INSET;
    col()
        .key("tooltip")
        .anchored(
            anchor.to_owned(),
            Side::Top,
            Align::Center,
            tooltip::SIDE_OFFSET - below_chip,
        )
        .items_center()
        .a11y_hidden()
        .group_opacity(0.0, 1.0)
        .hover_motion(enter, exit)
        .child(chip)
        .child(arrow)
}

/// A spinning `loader-circle` (`animate-spin`).
fn spinner(size: f32, color: crate::theme::Color) -> El {
    icon(Icon::LoaderCircle.rune(), size)
        .color(color.rune())
        .looping(LoopSpec::rotation(1.0))
}
