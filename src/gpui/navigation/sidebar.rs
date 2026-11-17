use std::rc::Rc;
use std::time::Instant;

use ::gpui::{
    AnyElement, App, ElementId, FocusHandle, FontWeight, Hsla, IntoElement, MouseButton,
    MouseMoveEvent, MouseUpEvent, ParentElement, RenderOnce, SharedString, Styled, Window, canvas,
    div, prelude::*, px, svg,
};

use crate::components::navigation::app_sidebar::motion::COLLAPSE_SETTLE;
use crate::components::navigation::sidebar::{
    COLLAPSE, Collapsible, LABEL_DRIFT, LABEL_FADE, MenuButtonSize, QUICK, RailKey, SidebarSkin,
    SidebarState, SidebarVariant, metrics,
};
use crate::fonts;
use crate::gpui::{ActiveTheme, shadows};
use crate::icons::Icon;
use crate::motion::Animated;
use crate::theme::{Shadow, Theme};

/// The sidebar's animated values for one frame, handed to everything inside it.
#[derive(Clone, Debug, PartialEq)]
pub struct SidebarFrame {
    pub variant: SidebarVariant,
    pub skin: SidebarSkin,
    /// The panel's outer width.
    pub width: f32,
    /// Collapse progress toward the icon rail, eased.
    pub collapse: f32,
    /// Opacity of row labels and trailing parts.
    pub labels: f32,
    /// Progress of the quick (150 ms) changes: row padding, search height.
    pub quick: f32,
    /// The sidebar is collapsed (the rail styles apply).
    pub collapsed: bool,
    /// The collapse has finished: rows with pages turn into flyouts.
    pub settled: bool,
}

impl SidebarFrame {
    /// The panel's inner width, inside its gutter.
    pub fn inner_width(&self) -> f32 {
        (self.width - 2.0 * self.variant.gutter()).max(0.0)
    }
}

/// Per-sidebar motion: keep one beside the [`SidebarState`] and call
/// [`SidebarMotion::frame`] every render.
pub struct SidebarMotion {
    width: Animated,
    collapse: Animated,
    labels: Animated,
    quick: Animated,
    collapsed: bool,
    collapsed_at: Option<Instant>,
}

impl SidebarMotion {
    pub fn new(state: &SidebarState, variant: SidebarVariant, collapsible: Collapsible) -> Self {
        let rail = if state.collapsed() { 1.0 } else { 0.0 };
        Self {
            width: Animated::new(state.outer_width(variant, collapsible), COLLAPSE),
            collapse: Animated::new(rail, COLLAPSE),
            labels: Animated::new(1.0 - rail, LABEL_FADE),
            quick: Animated::new(rail, QUICK),
            collapsed: state.collapsed(),
            collapsed_at: None,
        }
    }

    /// Retargets toward `state` and samples the frame at `now`.
    pub fn frame(
        &mut self,
        state: &SidebarState,
        variant: SidebarVariant,
        collapsible: Collapsible,
        theme: &Theme,
        now: Instant,
        reduce_motion: bool,
    ) -> SidebarFrame {
        if state.collapsed() != self.collapsed {
            self.collapsed = state.collapsed();
            self.collapsed_at = Some(now);
        }
        let rail = if self.collapsed { 1.0 } else { 0.0 };
        self.width.set(state.outer_width(variant, collapsible), now);
        self.collapse.set(rail, now);
        self.labels.set(1.0 - rail, now);
        self.quick.set(rail, now);
        if reduce_motion || state.resizing {
            self.width.snap(self.width.target());
        }
        if reduce_motion {
            for a in [&mut self.collapse, &mut self.labels, &mut self.quick] {
                a.snap(a.target());
            }
        }
        SidebarFrame {
            variant,
            skin: SidebarSkin::new(theme),
            width: self.width.value(now),
            collapse: self.collapse.value(now),
            labels: self.labels.value(now),
            quick: self.quick.value(now),
            collapsed: self.collapsed,
            settled: self.collapsed
                && (reduce_motion
                    || self
                        .collapsed_at
                        .is_none_or(|at| now - at >= COLLAPSE_SETTLE)),
        }
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        [&self.width, &self.collapse, &self.labels, &self.quick]
            .iter()
            .any(|a| a.is_animating(now))
            || self
                .collapsed_at
                .is_some_and(|at| self.collapsed && now - at < COLLAPSE_SETTLE)
    }
}

type ClickHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A sidebar row: a leading icon, a label that fades on the icon rail, and
/// optional trailing parts (a badge, a key cap, a chevron).
#[derive(IntoElement)]
pub struct SidebarMenuButton {
    id: ElementId,
    frame: SidebarFrame,
    icon: Option<Icon>,
    leading: Option<AnyElement>,
    label: Option<AnyElement>,
    trailing: Vec<AnyElement>,
    size: MenuButtonSize,
    active: bool,
    strong: bool,
    focus: Option<FocusHandle>,
    on_click: Option<ClickHandler>,
}

impl SidebarMenuButton {
    pub fn new(id: impl Into<ElementId>, frame: &SidebarFrame) -> Self {
        Self {
            id: id.into(),
            frame: frame.clone(),
            icon: None,
            leading: None,
            label: None,
            trailing: Vec::new(),
            size: MenuButtonSize::Default,
            active: false,
            strong: false,
            focus: None,
            on_click: None,
        }
    }

    /// A 16 px glyph in front.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Any leading element (a project square, an avatar).
    pub fn leading(mut self, el: impl IntoElement) -> Self {
        self.leading = Some(el.into_any_element());
        self
    }

    /// The row's text; it fades and drifts right on the icon rail.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        let frame = &self.frame;
        self.label = Some(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .relative()
                .left(px(LABEL_DRIFT * (1.0 - frame.labels)))
                .opacity(frame.labels)
                .child(label.into())
                .into_any_element(),
        );
        self
    }

    /// A custom label block (the switcher's two lines).
    pub fn label_element(mut self, el: impl IntoElement) -> Self {
        let labels = self.frame.labels;
        self.label = Some(
            div()
                .flex_1()
                .min_w_0()
                .relative()
                .left(px(LABEL_DRIFT * (1.0 - labels)))
                .opacity(labels)
                .child(el)
                .into_any_element(),
        );
        self
    }

    /// Trailing parts fade with the label.
    pub fn trailing(mut self, el: impl IntoElement) -> Self {
        let labels = self.frame.labels;
        self.trailing.push(
            div()
                .flex_none()
                .opacity(labels)
                .child(el)
                .into_any_element(),
        );
        self
    }

    pub fn size(mut self, size: MenuButtonSize) -> Self {
        self.size = size;
        self
    }

    /// The row is the current page: accent fill and medium weight.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// The row's group holds the current page: medium weight, foreground text.
    pub fn strong(mut self, strong: bool) -> Self {
        self.strong = strong;
        self
    }

    pub fn focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SidebarMenuButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let skin = &self.frame.skin;
        let (accent, accent_text, ring) = (skin.accent, skin.accent_foreground, skin.ring);
        let text = if self.active || self.strong {
            skin.strong
        } else {
            skin.foreground
        };
        let on_click = self.on_click;
        let key_click = on_click.clone();
        let group = SharedString::from(format!("sidebar-row-{}", self.id));
        let icon = self.icon.map(|icon| {
            let group = group.clone();
            svg()
                .path(icon.path())
                .flex_none()
                .size(px(metrics::ICON))
                .text_color(text)
                .group_hover(group, move |s| s.text_color(accent_text))
        });
        div()
            .id(self.id)
            .group(group)
            .when_some(self.focus, |d, focus| d.track_focus(&focus))
            .flex()
            .items_center()
            .gap(px(metrics::ROW_GAP))
            .w_full()
            .h(px(self.size.height()))
            .p(px(metrics::ROW_PADDING))
            .overflow_hidden()
            .rounded(px(metrics::ROW_RADIUS))
            .text_size(px(self.size.font_size()))
            .line_height(px(metrics::LINE_HEIGHT))
            .text_color(text)
            .when(self.active || self.strong, |d| {
                d.font_weight(FontWeight(500.0))
            })
            .when(self.active, |d| d.bg(accent))
            .cursor_pointer()
            .hover(move |s| s.bg(accent).text_color(accent_text))
            .focus_visible(move |s| s.shadow(shadows(&[Shadow::inset(0.0, 0.0, 2.0, ring)])))
            .when_some(on_click, |d, handler| {
                d.on_click(move |_, window, cx| handler(window, cx))
            })
            .when_some(key_click, |d, handler| {
                d.on_key_down(move |event, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        handler(window, cx);
                        cx.stop_propagation();
                    }
                })
            })
            .children(icon)
            .children(self.leading)
            .children(self.label)
            .children(self.trailing)
    }
}

/// What happened on the rail; feed it to [`SidebarState`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RailEvent {
    Press(f32),
    Move(f32),
    Release,
    Reset,
    Key(RailKey),
}

impl RailEvent {
    /// Applies the event to `state`.
    pub fn apply(self, state: &mut SidebarState) {
        match self {
            RailEvent::Press(x) => state.rail_press(x),
            RailEvent::Move(x) => state.rail_move(x),
            RailEvent::Release => state.rail_release(),
            RailEvent::Reset => state.rail_reset(),
            RailEvent::Key(key) => {
                state.rail_key(key);
            }
        }
    }
}

type RailHandler = Rc<dyn Fn(RailEvent, &mut Window, &mut App)>;

/// The sidebar's edge: drag to resize or collapse, double-click to reset, arrow
/// keys to resize. Place it as the last child of the panel.
#[derive(IntoElement)]
pub struct SidebarRail {
    id: ElementId,
    focus: FocusHandle,
    dragging: bool,
    collapsed: bool,
    on_event: RailHandler,
}

impl SidebarRail {
    pub fn new(
        id: impl Into<ElementId>,
        focus: &FocusHandle,
        state: &SidebarState,
        on_event: impl Fn(RailEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            focus: focus.clone(),
            dragging: state.is_dragging(),
            collapsed: state.collapsed(),
            on_event: Rc::new(on_event),
        }
    }
}

struct RailLine {
    hover: Animated,
}

impl RenderOnce for SidebarRail {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let skin = SidebarSkin::new(cx.notra());
        let now = Instant::now();
        let line = window.use_keyed_state(self.id.clone(), cx, |_, _| RailLine {
            hover: Animated::new(0.0, QUICK),
        });
        let focused = self.focus.is_focused(window);
        let hot = line.read(cx).hover.target() > 0.5 || focused || self.dragging;
        let lit = line.read(cx).hover.value(now);
        if line.read(cx).hover.is_animating(now) {
            window.request_animation_frame();
        }
        let color = if focused || self.dragging {
            skin.ring
        } else {
            skin.border.fade(lit)
        };
        let on_event = self.on_event;
        let (press, key, reset) = (on_event.clone(), on_event.clone(), on_event.clone());
        let dragging = self.dragging;
        let hover_line = line.clone();
        div()
            .id(self.id)
            .track_focus(&self.focus)
            .absolute()
            .top_0()
            .bottom_0()
            .right(px(-metrics::RAIL_HIT / 2.0))
            .w(px(metrics::RAIL_HIT))
            .cursor(if self.collapsed {
                ::gpui::CursorStyle::ResizeRight
            } else {
                ::gpui::CursorStyle::ResizeLeftRight
            })
            .on_hover(move |hovered, _, cx| {
                hover_line.update(cx, |l, cx| {
                    l.hover
                        .set(if *hovered { 1.0 } else { 0.0 }, Instant::now());
                    cx.notify();
                });
            })
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                if event.click_count >= 2 {
                    reset(RailEvent::Reset, window, cx);
                } else {
                    press(RailEvent::Press(f32::from(event.position.x)), window, cx);
                }
                cx.stop_propagation();
            })
            .on_key_down(move |event, window, cx| {
                let k = match event.keystroke.key.as_str() {
                    "right" => RailKey::Grow,
                    "left" => RailKey::Shrink,
                    "home" => RailKey::Home,
                    "end" => RailKey::End,
                    "enter" | "space" => RailKey::Toggle,
                    _ => return,
                };
                key(RailEvent::Key(k), window, cx);
                cx.stop_propagation();
            })
            .child(
                canvas(
                    |_, _, _| {},
                    move |_, _, window, _| {
                        if !dragging {
                            return;
                        }
                        let moved = on_event.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, _, window, cx| {
                            moved(RailEvent::Move(f32::from(event.position.x)), window, cx);
                        });
                        let released = on_event.clone();
                        window.on_mouse_event(move |_: &MouseUpEvent, _, window, cx| {
                            released(RailEvent::Release, window, cx);
                        });
                    },
                )
                .absolute()
                .size_full(),
            )
            .child(
                div()
                    .absolute()
                    .top(px(metrics::RAIL_INSET_Y))
                    .bottom(px(metrics::RAIL_INSET_Y))
                    .left(px((metrics::RAIL_HIT - metrics::RAIL_LINE) / 2.0))
                    .w(px(metrics::RAIL_LINE))
                    .rounded_full()
                    .when(hot || lit > 0.0, |d| d.bg(color)),
            )
    }
}

/// The page beside an inset sidebar: a rounded, bordered card.
#[derive(IntoElement)]
pub struct SidebarInset {
    children: Vec<AnyElement>,
}

impl SidebarInset {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl Default for SidebarInset {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for SidebarInset {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for SidebarInset {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let skin = SidebarSkin::new(cx.notra());
        div()
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .my(px(metrics::INSET_MARGIN))
            .mr(px(metrics::INSET_MARGIN))
            .overflow_hidden()
            .rounded(px(metrics::INSET_RADIUS))
            .border_1()
            .border_color(skin.border)
            .bg(skin.inset)
            .shadow(shadows(&skin.inset_shadow))
            .font_family(fonts::SANS)
            .children(self.children)
    }
}

/// The ghost icon button that shows or hides the sidebar.
#[derive(IntoElement)]
pub struct SidebarTrigger {
    id: ElementId,
    open: bool,
    on_click: ClickHandler,
}

impl SidebarTrigger {
    pub fn new(
        id: impl Into<ElementId>,
        open: bool,
        on_click: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            open,
            on_click: Rc::new(on_click),
        }
    }
}

impl RenderOnce for SidebarTrigger {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let skin = SidebarSkin::new(&theme);
        let hover: Hsla = skin.trigger_hover.into();
        let on_click = self.on_click;
        let group = SharedString::from(format!("sidebar-trigger-{}", self.id));
        div()
            .id(self.id)
            .group(group.clone())
            .flex()
            .items_center()
            .justify_center()
            .size(px(metrics::TRIGGER))
            .rounded(px(metrics::TRIGGER_RADIUS))
            .text_color(skin.trigger)
            .cursor_pointer()
            .hover(move |s| s.text_color(hover))
            .on_click(move |_, window, cx| on_click(window, cx))
            .child(
                svg()
                    .path(
                        if self.open {
                            Icon::HugeSidebarLeft01
                        } else {
                            Icon::HugeSidebarRight01
                        }
                        .path(),
                    )
                    .size(px(metrics::ICON))
                    .text_color(skin.trigger)
                    .group_hover(group, move |s| s.text_color(hover)),
            )
    }
}
