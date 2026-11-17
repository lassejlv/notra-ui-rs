use std::f32::consts::PI;
use std::rc::Rc;

use ::rune::{Align, Color as RuneColor, El, Side, col, icon, row, text};
use ::rune_motion::{Spring, SpringPair, Transition};

use super::{ToRune, sans, tween};
use crate::components::chat_minimap::{
    CARD_ALIGN_OFFSET, CARD_ENTER, CARD_ENTER_SCALE, CARD_EXIT, CARD_GAP, CARD_HEADER_GAP,
    CARD_PADDING, CARD_RADIUS, CARD_WIDTH, CHEVRON_SIZE, CLOSE_DELAY, ChatMinimapItem,
    ChatMinimapSide, ChatMinimapSkin, DESCRIPTION_FONT, DESCRIPTION_LINES, ITEM_HEIGHT,
    LINE_THICKNESS, LineEmphasis, NAV_FADE, NAV_HEIGHT, OPEN_DELAY, RAIL_WIDTH, ROW_BORDER,
    ROW_PADDING_X, TITLE_FONT, WIDTH_TRANSITION,
};
use crate::icons::Icon;
use crate::theme::Theme;

type Lens<S> = fn(&mut S) -> &mut ChatMinimapState;
type Select<S> = Rc<dyn Fn(&mut S, usize)>;
type Step<S> = Rc<dyn Fn(&mut S)>;

/// What the minimap remembers between views: the turn under the pointer and the turn
/// whose card is open. Keep one per minimap in your state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChatMinimapState {
    /// The turn under the pointer, which the falloff centres on.
    pub hovered: Option<usize>,
    /// The turn whose card is open, after the open delay.
    pub open: Option<usize>,
    /// The pointer is on the card, which holds it open.
    pub on_card: bool,
}

/// The conversation minimap: one line per turn, a preview card on hover, and arrows
/// that appear under the pointer.
///
/// ```ignore
/// ChatMinimap::new("minimap", &s.minimap, |s: &mut State| &mut s.minimap)
///     .items(demo_items(s.window))
///     .on_select(|s: &mut State, index| s.window.select(index))
///     .previous(s.window.can_previous(), |s: &mut State| s.window.previous())
///     .next(s.window.can_next(), |s: &mut State| s.window.next())
///     .build(&theme)
/// ```
pub struct ChatMinimap<S> {
    key: String,
    state: ChatMinimapState,
    lens: Lens<S>,
    side: ChatMinimapSide,
    items: Vec<ChatMinimapItem>,
    select: Option<Select<S>>,
    previous: Option<(bool, Step<S>)>,
    next: Option<(bool, Step<S>)>,
}

impl<S: 'static> ChatMinimap<S> {
    /// A minimap keyed `key` (its parts are keyed `{key}-item-{i}`, `{key}-card`, ..),
    /// drawing `state`, which `lens` finds again in the app's state.
    pub fn new(key: impl Into<String>, state: &ChatMinimapState, lens: Lens<S>) -> Self {
        Self {
            key: key.into(),
            state: *state,
            lens,
            side: ChatMinimapSide::Right,
            items: Vec::new(),
            select: None,
            previous: None,
            next: None,
        }
    }

    /// Which side of the rail the card opens on.
    pub fn side(mut self, side: ChatMinimapSide) -> Self {
        self.side = side;
        self
    }

    /// The turns, top to bottom.
    pub fn items(mut self, items: impl IntoIterator<Item = ChatMinimapItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// Called with the turn's index when a turn is pressed.
    pub fn on_select(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.select = Some(Rc::new(f));
        self
    }

    /// The "Previous turn" arrow above the turns, disabled unless `enabled`.
    pub fn previous(mut self, enabled: bool, f: impl Fn(&mut S) + 'static) -> Self {
        self.previous = Some((enabled, Rc::new(f)));
        self
    }

    /// The "Next turn" arrow below the turns, disabled unless `enabled`.
    pub fn next(mut self, enabled: bool, f: impl Fn(&mut S) + 'static) -> Self {
        self.next = Some((enabled, Rc::new(f)));
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let skin = ChatMinimapSkin::new(theme);
        let left = self.side == ChatMinimapSide::Left;
        let lens = self.lens;
        let key = &self.key;
        let ring = skin.focus_ring.rune();
        let open = self.state.open.filter(|&i| i < self.items.len());

        let place = |e: El| {
            if left {
                e.justify_end()
            } else {
                e.justify_start()
            }
        };
        let mut rail = col()
            .key(key.clone())
            .w(RAIL_WIDTH)
            .role("navigation")
            .label("Conversation turns");
        rail = if left {
            rail.items_end()
        } else {
            rail.items_start()
        };

        let nav = |slot: &'static str, label: &'static str, button: Option<(bool, Step<S>)>| {
            let (enabled, step) = button?;
            let mut chevron =
                icon(Icon::MinimapChevron.rune(), CHEVRON_SIZE.0).color(skin.nav.rune());
            if slot == "next" {
                chevron = chevron.rotate(PI);
            }
            let mut e = place(row())
                .key(format!("{key}-{slot}"))
                .w(RAIL_WIDTH)
                .h(NAV_HEIGHT)
                .px(ROW_PADDING_X + ROW_BORDER)
                .center_y()
                .role("button")
                .label(label)
                .disabled(!enabled)
                .focusable(enabled)
                .focus_ring(3.0, 0.0, ring);
            e = if enabled {
                let fade = tween(NAV_FADE);
                e.hover_group()
                    .hover_motion(fade, fade)
                    .cursor(::rune::Cursor::Pointer)
                    .on_press(move |s: &mut S| step(s))
                    .child(
                        chevron
                            .group_color(skin.nav_hover.rune())
                            .group_opacity(0.0, 1.0),
                    )
            } else {
                e.child(chevron.opacity(0.0))
            };
            Some(e)
        };

        rail = rail.children(nav("previous", "Previous turn", self.previous.clone()));

        for (index, item) in self.items.iter().enumerate() {
            let emphasis = LineEmphasis::of(
                index,
                self.state.hovered,
                item.active || open == Some(index),
            );
            let line = ::rune::el()
                .key(format!("{key}-line-{index}"))
                .h(LINE_THICKNESS)
                .w(emphasis.width())
                .fill(skin.line(emphasis).rune())
                .motion(Spring::new(WIDTH_TRANSITION.duration.as_secs_f32(), 0.0));
            let mut button = place(row())
                .key(format!("{key}-item-{index}"))
                .w(RAIL_WIDTH)
                .h(ITEM_HEIGHT)
                .px(ROW_PADDING_X + ROW_BORDER)
                .center_y()
                .role("button")
                .label(item.title.clone())
                .focusable(true)
                .focus_ring(3.0, 0.0, ring)
                .cursor(::rune::Cursor::Pointer)
                .on_hover_change(move |s: &mut S, inside, _| {
                    let state = lens(s);
                    if inside {
                        state.hovered = Some(index);
                    } else if state.hovered == Some(index) {
                        state.hovered = None;
                    }
                })
                .child(line);
            if let Some(select) = self.select.clone() {
                button = button.on_press(move |s: &mut S| select(s, index));
            }
            rail = rail.child(
                ::rune::el()
                    .key(format!("{key}-intent-{index}"))
                    .hover_delays(OPEN_DELAY, CLOSE_DELAY)
                    .on_hover_change(move |s: &mut S, inside, _| {
                        let state = lens(s);
                        if inside {
                            state.open = Some(index);
                        } else if state.open == Some(index) && !state.on_card {
                            state.open = None;
                        }
                    })
                    .child(button),
            );
        }

        rail = rail.children(nav("next", "Next turn", self.next.clone()));

        if let Some(index) = open {
            rail = rail.child(self.card(&skin, index));
        }
        rail
    }

    fn card(&self, skin: &ChatMinimapSkin, index: usize) -> El {
        let item = &self.items[index];
        let lens = self.lens;
        let side = match self.side {
            ChatMinimapSide::Left => Side::Left,
            ChatMinimapSide::Right => Side::Right,
        };
        let springs = SpringPair::new(
            Spring::new(CARD_ENTER.duration.as_secs_f32(), 0.0),
            Spring::new(CARD_EXIT.duration.as_secs_f32(), 0.0),
        );
        let enter = Transition::scale(CARD_ENTER_SCALE)
            .combined(Transition::offset(self.side.enter_travel(), 0.0));

        let mut header = col().gap(CARD_HEADER_GAP).px(CARD_PADDING).child(
            text(item.title.clone())
                .key("title")
                .font(sans(
                    TITLE_FONT.size,
                    TITLE_FONT.weight,
                    TITLE_FONT.line_height,
                ))
                .color(skin.card_title.rune())
                .truncate(),
        );
        if let Some(description) = &item.description {
            header = header.child(
                text(description.clone())
                    .key("description")
                    .font(sans(
                        DESCRIPTION_FONT.size,
                        DESCRIPTION_FONT.weight,
                        DESCRIPTION_FONT.line_height,
                    ))
                    .color(skin.card_description.rune())
                    .max_lines(DESCRIPTION_LINES),
            );
        }
        let mut body = col()
            .key(format!("{}-card-body", self.key))
            .w(CARD_WIDTH)
            .py(CARD_PADDING)
            .radius(CARD_RADIUS)
            .fill(skin.card_fill.rune())
            .hover_delays(std::time::Duration::ZERO, CLOSE_DELAY)
            .on_hover_change(move |s: &mut S, inside, _| {
                let state = lens(s);
                state.on_card = inside;
                if !inside && state.hovered.is_none() {
                    state.open = None;
                }
            })
            .child(header);
        for shadow in &skin.card_shadows {
            body = body.box_shadow(shadow.rune());
        }

        col()
            .key(format!("{}-card", self.key))
            .anchored(
                format!("{}-item-{index}", self.key),
                side,
                Align::Center,
                CARD_GAP,
            )
            .pt(CARD_ALIGN_OFFSET * 2.0)
            .fill(RuneColor::TRANSPARENT)
            .motion(Spring::NONE)
            .transition(
                Transition::asymmetric(enter, Transition::scale(CARD_ENTER_SCALE)).springs(springs),
            )
            .child(body)
    }
}
