use std::rc::Rc;

use ::rune::{El, Key, Tx, el, icon, row, text};
use ::rune_motion::{Curve, Spring};

use super::EASE_OUT_QUINT;
use crate::components::navigation::icon_tabs::{
    COLOR, ICON_REVEAL, INDICATOR_DURATION, IconTabsSkin, TabKey, metrics,
};
use crate::icons::Icon;
use crate::rune::{ToRune, sans, tween};
use crate::theme::Theme;

/// What icon tabs remember between frames: the selected tab and where each tab
/// was laid out, so the indicator can glide to it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IconTabsState {
    pub selected: usize,
    list: Option<[f32; 4]>,
    tabs: Vec<Option<[f32; 4]>>,
}

impl IconTabsState {
    pub fn new(selected: usize) -> Self {
        Self {
            selected,
            ..Self::default()
        }
    }

    fn indicator(&self) -> Option<[f32; 2]> {
        let list = self.list?;
        let tab = (*self.tabs.get(self.selected)?)?;
        Some([tab[0] - list[0], tab[2]])
    }
}

type Lens<S> = fn(&mut S) -> &mut IconTabsState;

/// Segmented tabs where only the active tab shows its icon:
/// `IconTabs::new("views", &s.tabs).tab(Icon::Send, "Deliveries").build(&theme, |s: &mut State| &mut s.tabs)`.
pub struct IconTabs<'a> {
    key: String,
    state: &'a IconTabsState,
    tabs: Vec<(Icon, String, bool)>,
    width: Option<f32>,
    label: String,
}

impl<'a> IconTabs<'a> {
    pub fn new(key: impl Into<String>, state: &'a IconTabsState) -> Self {
        Self {
            key: key.into(),
            state,
            tabs: Vec::new(),
            width: None,
            label: String::new(),
        }
    }

    /// Adds a tab; `icon` slides in beside `label` while it is active.
    pub fn tab(mut self, icon: Icon, label: impl Into<String>) -> Self {
        self.tabs.push((icon, label.into(), false));
        self
    }

    /// Adds a tab whose icon stays visible when inactive.
    pub fn pinned_tab(mut self, icon: Icon, label: impl Into<String>) -> Self {
        self.tabs.push((icon, label.into(), true));
        self
    }

    /// A fixed list width; the tabs then share it equally.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// The tab list's accessible name.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn build<S: 'static>(self, theme: &Theme, lens: Lens<S>) -> El {
        let skin = IconTabsSkin::new(theme);
        let count = self.tabs.len();
        let selected = self.state.selected.min(count.saturating_sub(1));
        let key = self.key;
        let reveal = tween(ICON_REVEAL);
        let colors = tween(COLOR);

        let indicator = self.state.indicator().map(|[left, width]| {
            el().key(format!("{key}/indicator"))
                .absolute(0.0, metrics::LIST_PADDING)
                .offset(left, 0.0)
                .offset_motion(Curve::tween(
                    INDICATOR_DURATION.as_secs_f32(),
                    EASE_OUT_QUINT,
                ))
                .w(width)
                .h(metrics::LIST_HEIGHT - 2.0 * metrics::LIST_PADDING)
                .motion(Spring::new(INDICATOR_DURATION.as_secs_f32(), 0.0))
                .radius(metrics::TAB_RADIUS)
                .fill(skin.indicator.rune())
                .box_shadow(skin.indicator_shadow.rune())
        });

        let focus_keys: Rc<Vec<String>> =
            Rc::new((0..count).map(|ix| format!("{key}/tab-{ix}")).collect());
        let tabs = self
            .tabs
            .into_iter()
            .enumerate()
            .map(|(ix, (glyph, label, pinned))| {
                let active = ix == selected;
                let shown = active || pinned;
                let color = if active { skin.text_active } else { skin.text };
                let slot = row()
                    .key(format!("{key}/tab-{ix}/icon"))
                    .center_y()
                    .justify_center()
                    .h(metrics::ICON_SIZE)
                    .w(if shown { metrics::ICON_SLOT } else { 0.0 })
                    .mr(if shown { 0.0 } else { metrics::ICON_TUCK })
                    .motion(Spring::new(ICON_REVEAL.duration.as_secs_f32(), 0.0))
                    .clip_rounded(0.0)
                    .opacity(if shown { 1.0 } else { 0.0 })
                    .opacity_motion(reveal)
                    .child(
                        icon(glyph.rune(), metrics::ICON_SIZE)
                            .color(color.rune())
                            .scale(if shown {
                                1.0
                            } else {
                                metrics::ICON_HIDDEN_SCALE
                            })
                            .scale_motion(reveal),
                    );
                let mut label_el = text(label.clone())
                    .font(sans(
                        metrics::FONT_SIZE,
                        metrics::FONT_WEIGHT,
                        metrics::LINE_HEIGHT,
                    ))
                    .color(color.rune())
                    .single_line();
                if !active {
                    label_el = label_el.group_color(skin.text_hover.rune());
                }
                let keys = focus_keys.clone();
                let mut tab = row()
                    .key(focus_keys[ix].clone())
                    .center_y()
                    .justify_center()
                    .gap(metrics::TAB_GAP)
                    .h_full()
                    .px(metrics::TAB_PADDING_X)
                    .radius(metrics::TAB_RADIUS)
                    .role("tab")
                    .label(label)
                    .checked(active)
                    .focusable(true)
                    .tab_index(if active { 0 } else { -1 })
                    .focus_ring(metrics::FOCUS_OUTLINE, 0.0, skin.ring.rune())
                    .hover_group()
                    .hover_motion(colors, colors)
                    .cursor(::rune::Cursor::Pointer)
                    .on_layout(move |s: &mut S, rect| {
                        let tabs = &mut lens(s).tabs;
                        if tabs.len() <= ix {
                            tabs.resize(ix + 1, None);
                        }
                        tabs[ix] = Some(rect);
                    })
                    .on_press(move |s: &mut S| lens(s).selected = ix)
                    .on_key(move |s: &mut S, ev, tx: &mut Tx| {
                        let step = match ev.key {
                            Key::Right => TabKey::Next,
                            Key::Left => TabKey::Previous,
                            Key::Home => TabKey::First,
                            Key::End => TabKey::Last,
                            Key::Enter => {
                                lens(s).selected = ix;
                                return true;
                            }
                            Key::Text(ref t) if t == " " => {
                                lens(s).selected = ix;
                                return true;
                            }
                            _ => return false,
                        };
                        tx.focus(keys[step.apply(ix, keys.len())].clone(), true);
                        true
                    })
                    .child(slot)
                    .child(label_el);
                if self.width.is_some() {
                    tab = tab.grow(1.0);
                }
                tab
            });

        let mut list = row()
            .key(key.clone())
            .center_y()
            .justify_center()
            .gap(metrics::LIST_GAP)
            .h(metrics::LIST_HEIGHT)
            .padding(metrics::LIST_PADDING)
            .radius(metrics::LIST_RADIUS)
            .fill(skin.list.rune())
            .role("tablist")
            .label(self.label)
            .on_layout(move |s: &mut S, rect| lens(s).list = Some(rect));
        if let Some(w) = self.width {
            list = list.w(w);
        }
        list.children(indicator).children(tabs)
    }
}
