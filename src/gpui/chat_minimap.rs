use std::f32::consts::PI;
use std::rc::Rc;
use std::time::Instant;

use ::gpui::{
    Anchor, AnchoredPositionMode, App, ClickEvent, ElementId, FocusHandle, FontWeight, Hsla,
    IntoElement, ParentElement, RenderOnce, Role, SharedString, Styled, Transformation, Window,
    anchored, deferred, div, point, prelude::*, px, radians, svg,
};

use super::{ActiveTheme, shadows};
use crate::components::chat_minimap::{
    CARD_ALIGN_OFFSET, CARD_EXIT, CARD_GAP, CARD_HEADER_GAP, CARD_PADDING, CARD_RADIUS, CARD_WIDTH,
    CHEVRON_SIZE, COLOR_TRANSITION, CardFrame, ChatMinimapItem, ChatMinimapSide, ChatMinimapSkin,
    DESCRIPTION_FONT, DESCRIPTION_LINES, HoverIntent, ITEM_HEIGHT, LINE_THICKNESS, LineEmphasis,
    NAV_FADE, NAV_HEIGHT, RAIL_WIDTH, ROW_PADDING_X, TITLE_FONT, WIDTH_TRANSITION,
};
use crate::fonts;
use crate::icons::Icon;
use crate::motion::Animated;
use crate::theme::{Color, Shadow};

type SelectHandler = Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>;
type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// The conversation minimap: one line per turn, a preview card on hover, and arrows
/// that appear under the pointer.
///
/// ```ignore
/// ChatMinimap::new("minimap")
///     .items(demo_items(self.window))
///     .on_select(cx.listener(|this, index, _, cx| this.window.select(*index)))
///     .previous(self.window.can_previous(), cx.listener(|this, _, _, cx| this.window.previous()))
///     .next(self.window.can_next(), cx.listener(|this, _, _, cx| this.window.next()))
/// ```
#[derive(IntoElement)]
pub struct ChatMinimap {
    id: ElementId,
    side: ChatMinimapSide,
    items: Vec<ChatMinimapItem>,
    on_select: Option<SelectHandler>,
    previous: Option<(bool, ClickHandler)>,
    next: Option<(bool, ClickHandler)>,
}

impl ChatMinimap {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            side: ChatMinimapSide::Right,
            items: Vec::new(),
            on_select: None,
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

    /// Called with the turn's index when a turn is clicked.
    pub fn on_select(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(move |index, window, cx| {
            handler(&index, window, cx)
        }));
        self
    }

    /// The "Previous turn" arrow above the turns, disabled unless `enabled`.
    pub fn previous(
        mut self,
        enabled: bool,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.previous = Some((enabled, Box::new(handler)));
        self
    }

    /// The "Next turn" arrow below the turns, disabled unless `enabled`.
    pub fn next(
        mut self,
        enabled: bool,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.next = Some((enabled, Box::new(handler)));
        self
    }
}

/// A colour easing toward its target, restarting from where it is when retargeted.
struct ColorTransition {
    from: Color,
    to: Color,
    progress: Animated,
}

impl ColorTransition {
    fn new(color: Color) -> Self {
        Self {
            from: color,
            to: color,
            progress: Animated::new(1.0, COLOR_TRANSITION),
        }
    }

    fn set(&mut self, target: Color, now: Instant, reduce_motion: bool) {
        if target == self.to {
            return;
        }
        self.from = self.value(now);
        self.to = target;
        self.progress.snap(if reduce_motion { 1.0 } else { 0.0 });
        self.progress.set(1.0, now);
    }

    fn value(&self, now: Instant) -> Color {
        match self.progress.value(now) {
            t if t >= 1.0 => self.to,
            t => self.from.mix(self.to, t),
        }
    }
}

struct LineState {
    focus: FocusHandle,
    width: Animated,
    color: ColorTransition,
}

struct NavState {
    focus: FocusHandle,
    hover: Animated,
}

/// The card on screen: the turn it previews, and when it began entering or leaving.
#[derive(Clone, Copy)]
struct CardPresence {
    index: usize,
    since: Instant,
    leaving: bool,
}

struct MinimapState {
    intent: HoverIntent,
    lines: Vec<LineState>,
    nav: [NavState; 2],
    card: Option<CardPresence>,
}

impl MinimapState {
    fn sync_card(&mut self, now: Instant, reduce_motion: bool) {
        self.intent.tick(now);
        self.card = match (self.card, self.intent.open()) {
            (None, Some(index)) => Some(CardPresence {
                index,
                since: now,
                leaving: false,
            }),
            (Some(card), Some(index)) if card.leaving => Some(CardPresence {
                index,
                since: now,
                leaving: false,
            }),
            (Some(card), Some(index)) => Some(CardPresence { index, ..card }),
            (Some(card), None) if !card.leaving && !reduce_motion => Some(CardPresence {
                since: now,
                leaving: true,
                ..card
            }),
            (Some(card), None) if card.leaving && now - card.since < CARD_EXIT.total() => {
                Some(card)
            }
            (_, None) => None,
        };
    }
}

fn retarget(value: &mut Animated, target: f32, now: Instant, reduce_motion: bool) {
    if reduce_motion {
        value.snap(target);
    } else {
        value.set(target, now);
    }
}

impl RenderOnce for ChatMinimap {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let skin = ChatMinimapSkin::new(&theme);
        let now = Instant::now();
        let reduce_motion = cx.reduce_motion();
        let side = self.side;
        let left = side == ChatMinimapSide::Left;

        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| MinimapState {
            intent: HoverIntent::default(),
            lines: Vec::new(),
            nav: [0, 1].map(|_| NavState {
                focus: cx.focus_handle(),
                hover: Animated::new(0.0, NAV_FADE),
            }),
            card: None,
        });

        let focused = state
            .read(cx)
            .lines
            .iter()
            .position(|line| line.focus.is_focused(window) && window.last_input_was_keyboard());
        let items = &self.items;
        state.update(cx, |s, cx| {
            s.sync_card(now, reduce_motion);
            let hovered = s.intent.pointer().or(focused);
            let open = s.card.filter(|c| !c.leaving).map(|c| c.index);
            s.lines.truncate(items.len());
            for (index, item) in items.iter().enumerate() {
                let emphasis = LineEmphasis::of(index, hovered, item.active || open == Some(index));
                let (width, color) = (emphasis.width(), skin.line(emphasis));
                match s.lines.get_mut(index) {
                    Some(line) => {
                        retarget(&mut line.width, width, now, reduce_motion);
                        line.color.set(color, now, reduce_motion);
                    }
                    None => s.lines.push(LineState {
                        focus: cx.focus_handle(),
                        width: Animated::new(width, WIDTH_TRANSITION),
                        color: ColorTransition::new(color),
                    }),
                }
            }
        });

        let s = state.read(cx);
        let animating = s.intent.deadline().is_some()
            || s.card.is_some_and(|c| now - c.since < CARD_EXIT.total())
            || s.lines
                .iter()
                .any(|l| l.width.is_animating(now) || l.color.progress.is_animating(now))
            || s.nav.iter().any(|n| n.hover.is_animating(now));
        if animating {
            window.request_animation_frame();
        }

        let focus_border = skin.focus_border;
        let focus_ring = shadows(&[Shadow::drop(0.0, 0.0, 3.0, skin.focus_ring)]);
        let row = |id: ElementId, focus: &FocusHandle, height: f32| {
            div()
                .id(id)
                .track_focus(focus)
                .flex()
                .flex_none()
                .items_center()
                .map(|d| {
                    if left {
                        d.justify_end()
                    } else {
                        d.justify_start()
                    }
                })
                .w(px(RAIL_WIDTH))
                .h(px(height))
                .px(px(ROW_PADDING_X))
                .border_1()
                .border_color(Hsla::transparent_black())
                .focus_visible({
                    let ring = focus_ring.clone();
                    move |style| style.border_color(focus_border).shadow(ring)
                })
        };

        let lines = (0..self.items.len()).map(|index| {
            let line = &s.lines[index];
            let hover_state = state.clone();
            let on_select = self.on_select.clone();
            row(("item", index).into(), &line.focus, ITEM_HEIGHT)
                .role(Role::Button)
                .aria_label(SharedString::from(self.items[index].title.clone()))
                .tab_stop(true)
                .cursor_pointer()
                .on_hover(move |hovered, _, cx| {
                    hover_state.update(cx, |s, cx| {
                        s.intent.hover_item(index, *hovered, Instant::now());
                        cx.notify();
                    });
                })
                .when_some(on_select, |d, handler| {
                    d.on_click(move |_, window, cx| handler(index, window, cx))
                })
                .child(
                    div()
                        .flex_none()
                        .h(px(LINE_THICKNESS))
                        .w(px(line.width.value(now)))
                        .bg(line.color.value(now)),
                )
        });

        let nav = |slot: usize, button: Option<(bool, ClickHandler)>| {
            let (enabled, handler) = button?;
            let nav = &s.nav[slot];
            let hover = nav.hover.value(now);
            let hover_state = state.clone();
            let label = if slot == 0 {
                "Previous turn"
            } else {
                "Next turn"
            };
            Some(
                row((label, slot).into(), &nav.focus, NAV_HEIGHT)
                    .role(Role::Button)
                    .aria_label(label)
                    .tab_stop(enabled)
                    .opacity(if enabled { hover } else { 0.0 })
                    .when(enabled, |d| {
                        d.focus_visible(|style| style.opacity(1.0))
                            .cursor_pointer()
                            .on_hover(move |hovered, _, cx| {
                                hover_state.update(cx, |s, cx| {
                                    let target = if *hovered { 1.0 } else { 0.0 };
                                    retarget(
                                        &mut s.nav[slot].hover,
                                        target,
                                        Instant::now(),
                                        cx.reduce_motion(),
                                    );
                                    cx.notify();
                                });
                            })
                            .on_click(handler)
                    })
                    .child(
                        svg()
                            .path(Icon::MinimapChevron.path())
                            .flex_none()
                            .size(px(CHEVRON_SIZE.0))
                            .text_color(skin.nav.mix(skin.nav_hover, hover))
                            .when(slot == 1, |svg| {
                                svg.with_transformation(Transformation::rotate(radians(PI)))
                            }),
                    ),
            )
        };

        let card = s.card.and_then(|card| {
            let item = self.items.get(card.index)?;
            let elapsed = now - card.since;
            let frame = match (card.leaving, reduce_motion) {
                (_, true) => CardFrame::SHOWN,
                (true, false) => CardFrame::exiting(elapsed),
                (false, false) => CardFrame::entering(elapsed, side),
            };
            let centre = NAV_HEIGHT + (card.index as f32 + 0.5) * ITEM_HEIGHT + CARD_ALIGN_OFFSET;
            let (x, anchor) = if left {
                (-CARD_GAP, Anchor::RightCenter)
            } else {
                (RAIL_WIDTH + CARD_GAP, Anchor::LeftCenter)
            };
            let hover_state = state.clone();
            let body = div()
                .id("card")
                .relative()
                .left(px(frame.offset_x))
                .opacity(frame.opacity)
                .w(px(CARD_WIDTH))
                .py(px(CARD_PADDING))
                .rounded(px(CARD_RADIUS))
                .bg(skin.card_fill)
                .shadow(shadows(&skin.card_shadows))
                .overflow_hidden()
                .font_family(fonts::SANS)
                .on_hover(move |hovered, _, cx| {
                    hover_state.update(cx, |s, cx| {
                        s.intent.hover_card(*hovered, Instant::now());
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(CARD_HEADER_GAP))
                        .px(px(CARD_PADDING))
                        .child(
                            div()
                                .text_size(px(TITLE_FONT.size))
                                .line_height(px(TITLE_FONT.line_height))
                                .font_weight(FontWeight(TITLE_FONT.weight))
                                .text_color(skin.card_title)
                                .truncate()
                                .child(SharedString::from(item.title.clone())),
                        )
                        .when_some(item.description.clone(), |d, description| {
                            d.child(
                                div()
                                    .text_size(px(DESCRIPTION_FONT.size))
                                    .line_height(px(DESCRIPTION_FONT.line_height))
                                    .font_weight(FontWeight(DESCRIPTION_FONT.weight))
                                    .text_color(skin.card_description)
                                    .line_clamp(DESCRIPTION_LINES)
                                    .child(SharedString::from(description)),
                            )
                        }),
                );
            Some(
                div().absolute().top_0().left_0().child(deferred(
                    anchored()
                        .position_mode(AnchoredPositionMode::Local)
                        .anchor(anchor)
                        .position(point(px(x), px(centre)))
                        .child(body),
                )),
            )
        });

        div()
            .id(self.id)
            .role(Role::Navigation)
            .aria_label("Conversation turns")
            .relative()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(RAIL_WIDTH))
            .map(|d| if left { d.items_end() } else { d.items_start() })
            .children(nav(0, self.previous))
            .children(lines)
            .children(nav(1, self.next))
            .children(card)
    }
}
