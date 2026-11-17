use std::rc::Rc;

use ::rune::{El, Tx, col, el, icon, row, text};

use super::menu::depth_surface;
use super::popup;
use crate::components::button::ButtonHover;
use crate::components::overlays::menu::metrics as rows;
use crate::components::overlays::select::{
    MOTION, OPTION_PADDING_RIGHT, PLACEMENT, SelectEntry, SelectOption, SelectSize, SelectSkin,
    SelectVariant, TRANSITION, TriggerSkin, label_of, metrics,
};
use crate::icons::Icon;
use crate::rune::{ToRune, brightness_overlay, sans, tween};
use crate::theme::Theme;

/// Whether a select's list is open, owned by the app.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectState {
    pub open: bool,
}

/// Picks the select's state out of the app's.
pub type Lens<S> = fn(&mut S) -> &mut SelectState;

type Change<S> = Rc<dyn Fn(&mut S, &str)>;

/// Notra's select: a Depth (or ghost) button showing the value and a listbox anchored below it.
///
/// ```ignore
/// let select = Select::new("frequency", &entries).value(Some(&s.frequency)).width(192.0);
/// let trigger = select.trigger(&s.frequency_select, t, |s: &mut App| &mut s.frequency_select);
/// layer.extend(select.popup(&s.frequency_select, t, |s: &mut App| &mut s.frequency_select, |s: &mut App, v| s.frequency = v.into()));
/// ```
///
/// The list is modal while open: it focuses the selected option, arrows and type-ahead move,
/// Enter, Space or a press picks and closes, Escape or a press outside closes, and focus
/// returns to the trigger.
pub struct Select<'a> {
    key: String,
    entries: &'a [SelectEntry],
    value: Option<&'a str>,
    placeholder: String,
    label: Option<String>,
    variant: SelectVariant,
    size: SelectSize,
    width: Option<f32>,
}

impl<'a> Select<'a> {
    pub fn new(key: impl Into<String>, entries: &'a [SelectEntry]) -> Self {
        Self {
            key: key.into(),
            entries,
            value: None,
            placeholder: String::new(),
            label: None,
            variant: SelectVariant::Default,
            size: SelectSize::Default,
            width: None,
        }
    }

    pub fn value(mut self, value: Option<&'a str>) -> Self {
        self.value = value;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// The accessible name (the visible `<label>` text).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn variant(mut self, variant: SelectVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: SelectSize) -> Self {
        self.size = size;
        self
    }

    /// The trigger's width (`w-48`); by default it fits its value.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    fn trigger_key(&self) -> String {
        format!("{}-trigger", self.key)
    }

    fn option_key(&self, value: &str) -> String {
        format!("{}-option-{value}", self.key)
    }

    /// The trigger: pressing it (or Space, ArrowUp or ArrowDown while it has focus) opens the
    /// list.
    pub fn trigger<S: 'static>(&self, state: &SelectState, theme: &Theme, lens: Lens<S>) -> El {
        let skin = TriggerSkin::new(theme, self.variant);
        let paint = if state.open {
            &skin.expanded
        } else {
            &skin.button
        };
        let m = self.size.metrics();
        let shown = self.value.and_then(|v| label_of(self.entries, v));
        let ink = if shown.is_some() {
            paint.text
        } else {
            skin.placeholder
        };
        let fade = tween(TRANSITION);
        let mut e = row()
            .key(self.trigger_key())
            .role("combobox")
            .label(
                self.label
                    .clone()
                    .unwrap_or_else(|| self.placeholder.clone()),
            )
            .a11y_value(shown.unwrap_or(&self.placeholder).to_string())
            .expanded(state.open)
            .h(m.height)
            .px(m.padding_x.unwrap_or(10.0))
            .gap(m.gap)
            .center_y()
            .justify_between()
            .radius(paint.radius.unwrap_or(m.radius))
            .border(1.0, paint.border.rune())
            .fill(paint.fill.rune())
            .focusable(true)
            .focus_ring(3.0, 0.0, theme.ring.fade(0.5).rune())
            .cursor(::rune::Cursor::Pointer)
            .on_press(move |s: &mut S| {
                let st = lens(s);
                st.open = !st.open;
            });
        e = match self.width {
            Some(w) => e.w(w),
            None => e.self_start(),
        };
        if paint.radius.is_some() {
            e = e.continuous_corners();
        }
        if let Some((top, bottom)) = paint.gradient {
            e = e.gradient(top.rune(), bottom.rune());
        }
        for s in &paint.shadows {
            e = e.box_shadow(s.rune());
        }
        let overlay = match paint.hover {
            ButtonHover::Brightness(f) => Some(brightness_overlay(
                paint.gradient.map_or(paint.fill, |(t, b)| t.mix(b, 0.5)),
                f,
            )),
            ButtonHover::Fill { fill, .. } if !state.open => Some(fill),
            _ => None,
        };
        if let Some(o) = overlay {
            e = e.hover(o.rune()).hover_motion(fade, fade);
        }
        let value = text(shown.unwrap_or(&self.placeholder).to_string())
            .font(sans(m.font_size, 400.0, m.line_height))
            .color(ink.rune())
            .truncate();
        e.child(if self.width.is_some() {
            value.grow(1.0)
        } else {
            value
        })
        .child(
            icon(Icon::ChevronsUpDown.rune(), metrics::CHEVRON)
                .color(skin.chevron.rune())
                .a11y_hidden(),
        )
    }

    /// The open list, to mount after the trigger; `None` while closed.
    pub fn popup<S: 'static>(
        &self,
        state: &SelectState,
        theme: &Theme,
        lens: Lens<S>,
        on_change: impl Fn(&mut S, &str) + 'static,
    ) -> Option<El> {
        if !state.open {
            return None;
        }
        let skin = SelectSkin::new(theme);
        let change: Change<S> = Rc::new(on_change);
        let option = |o: &SelectOption| self.option(o, &skin, lens, &change);
        let mut list = col()
            .key(format!("{}-listbox", self.key))
            .role("listbox")
            .label(self.label.clone().unwrap_or_default())
            .focus_group()
            .items_stretch()
            .padding(metrics::LIST_PADDING);
        for (i, entry) in self.entries.iter().enumerate() {
            list = list.child(match entry {
                SelectEntry::Group { label, options } => {
                    let mut group = col()
                        .key(format!("{}-group-{i}", self.key))
                        .role("group")
                        .items_stretch()
                        .padding(metrics::GROUP_PADDING);
                    if let Some(l) = label {
                        group = group.label(l.clone()).child(
                            row()
                                .px(rows::LABEL_PADDING_X)
                                .py(rows::LABEL_PADDING_Y)
                                .a11y_hidden()
                                .child(
                                    text(l.clone())
                                        .font(sans(rows::LABEL_FONT, 500.0, rows::LABEL_LINE))
                                        .color(skin.label.rune()),
                                ),
                        );
                    }
                    group.children(options.iter().map(option))
                }
                SelectEntry::Option(o) => option(o),
                SelectEntry::Separator => el()
                    .key(format!("{}-sep-{i}", self.key))
                    .h(1.0)
                    .mx(rows::SEPARATOR_MARGIN_X)
                    .my(rows::SEPARATOR_MARGIN_Y)
                    .fill(skin.separator.rune())
                    .a11y_hidden(),
            });
        }
        let width = self.width.unwrap_or(0.0).max(metrics::MIN_WIDTH);
        let surface = depth_surface(&skin.surface)
            .key(format!("{}-popup", self.key))
            .w(width)
            .max_h(rows::MAX_HEIGHT)
            .padding(1.0)
            .child(list);
        let trigger = self.trigger_key();
        Some(
            popup(surface, &trigger, PLACEMENT, &MOTION)
                .focus_return(trigger)
                .modal(true, move |s: &mut S| lens(s).open = false),
        )
    }

    fn option<S: 'static>(
        &self,
        o: &SelectOption,
        skin: &SelectSkin,
        lens: Lens<S>,
        change: &Change<S>,
    ) -> El {
        let selected = self.value == Some(o.value.as_str());
        let (_, ink, _) = skin.row(false, false);
        let (fill, hot, _) = skin.row(true, false);
        let mut e = row()
            .key(self.option_key(&o.value))
            .role("option")
            .label(o.label.clone())
            .checked(selected)
            .min_h(rows::ITEM_HEIGHT)
            .pl(rows::ITEM_PADDING_X)
            .pr(OPTION_PADDING_RIGHT)
            .py(rows::ITEM_PADDING_Y)
            .gap(rows::ITEM_GAP)
            .center_y()
            .radius(rows::ITEM_RADIUS)
            .continuous_corners()
            .hover_group()
            .focusable(true)
            .focus_highlight(fill.rune())
            .focus_selected(selected)
            .child(
                text(o.label.clone())
                    .font(sans(rows::ITEM_FONT, 400.0, rows::ITEM_LINE))
                    .color(ink.rune())
                    .group_color(hot.rune())
                    .truncate(),
            );
        if selected {
            e = e.autofocus().child(
                el().pin(None, Some(8.0), Some(rows::INDICATOR_RIGHT), None)
                    .size(rows::ICON, rows::ICON)
                    .a11y_hidden()
                    .child(
                        icon(Icon::Check.rune(), rows::ICON)
                            .color(ink.rune())
                            .group_color(hot.rune()),
                    ),
            );
        }
        if o.disabled {
            return e.layer_opacity(rows::DISABLED_OPACITY).disabled(true);
        }
        let value = o.value.clone();
        let change = change.clone();
        e.cursor(::rune::Cursor::Pointer)
            .on_press_tx(move |s: &mut S, tx: &mut Tx| {
                lens(s).open = false;
                change(s, &value);
                tx.animated();
            })
    }
}
