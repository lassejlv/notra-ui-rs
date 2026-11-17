use std::collections::HashMap;
use std::time::Instant;

use ::gpui::{
    AnyElement, App, Bounds, Context, Div, FocusHandle, FontWeight, Hsla, IntoElement,
    ParentElement, PathBuilder, Pixels, Render, ScrollHandle, SharedString, Stateful, Styled,
    Window, anchored, canvas, deferred, div, linear_color_stop, linear_gradient, point, prelude::*,
    px, svg,
};

use super::sidebar::{
    RailEvent, SidebarInset, SidebarMenuButton, SidebarMotion, SidebarRail, SidebarTrigger,
};
use crate::components::navigation::app_sidebar::{
    AppSidebarMenu, AppSidebarSkin, AppSidebarState, MenuEntry, NavGroup, demo, metrics, motion,
};
use crate::components::navigation::sidebar::{
    Collapsible, MenuButtonSize, SUB_PANEL, SidebarVariant, metrics as row,
};
use crate::fonts;
use crate::gpui::{ActiveTheme, gradient, shadows};
use crate::icons::Icon;
use crate::motion::Animated;
use crate::theme::{Color, Shadow};

type InsetBuilder = Box<dyn Fn(&AppSidebarState, &mut Window, &mut App) -> AnyElement>;

/// The Notra dashboard sidebar block with its inset page, as a GPUI view.
///
/// ```ignore
/// cx.new(|cx| AppSidebar::new(true, "#traffic", cx).inset(|state, _, _| page(state)))
/// ```
pub struct AppSidebar {
    state: AppSidebarState,
    motion: SidebarMotion,
    variant: SidebarVariant,
    collapsible: Collapsible,
    rail_focus: FocusHandle,
    menu_focus: FocusHandle,
    panels: HashMap<&'static str, Animated>,
    bounds: HashMap<SharedString, Bounds<Pixels>>,
    tooltip: Option<(SharedString, SharedString)>,
    tooltip_fade: Animated,
    menu_fade: Animated,
    shown_menu: Option<AppSidebarMenu>,
    ask_width: Animated,
    scroll: ScrollHandle,
    inset: Option<InsetBuilder>,
}

impl AppSidebar {
    pub fn new(open: bool, active: &'static str, cx: &mut Context<Self>) -> Self {
        let state = AppSidebarState::new(open, active);
        let variant = SidebarVariant::Inset;
        let collapsible = Collapsible::Icon;
        let panels = demo::NAV
            .iter()
            .chain(demo::FOOTER.iter())
            .map(|g| {
                let open = if g.is_open(active) { 1.0 } else { 0.0 };
                (g.href, Animated::new(open, SUB_PANEL))
            })
            .collect();
        Self {
            motion: SidebarMotion::new(&state.sidebar, variant, collapsible),
            state,
            variant,
            collapsible,
            rail_focus: cx.focus_handle(),
            menu_focus: cx.focus_handle(),
            panels,
            bounds: HashMap::new(),
            tooltip: None,
            tooltip_fade: Animated::new(0.0, motion::TOOLTIP_IN),
            menu_fade: Animated::new(0.0, motion::MENU_IN),
            shown_menu: None,
            ask_width: Animated::new(
                if open { 1.0 } else { 0.0 },
                crate::components::navigation::sidebar::QUICK,
            ),
            scroll: ScrollHandle::new(),
            inset: None,
        }
    }

    /// Builds the page shown in the inset beside the sidebar.
    pub fn inset(
        mut self,
        build: impl Fn(&AppSidebarState, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.inset = Some(Box::new(build));
        self
    }

    pub fn state(&self) -> &AppSidebarState {
        &self.state
    }

    /// A transparent child that records its parent's bounds under `key`.
    fn measure(&self, key: SharedString, cx: &Context<Self>) -> impl IntoElement + use<> {
        let this = cx.entity().downgrade();
        canvas(
            move |bounds, _, cx| {
                this.update(cx, |this, _| {
                    this.bounds.insert(key, bounds);
                })
                .ok();
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full()
    }

    fn navigate(&mut self, href: &'static str, cx: &mut Context<Self>) {
        self.state.navigate(href);
        let now = Instant::now();
        for g in demo::NAV.iter().chain(demo::FOOTER.iter()) {
            if let Some(panel) = self.panels.get_mut(g.href) {
                panel.set(if g.is_open(href) { 1.0 } else { 0.0 }, now);
            }
        }
        cx.notify();
    }

    fn toggle_menu(&mut self, menu: AppSidebarMenu, window: &mut Window, cx: &mut Context<Self>) {
        self.state.toggle_menu(menu);
        self.tooltip = None;
        if self.state.menu.is_some() {
            window.focus(&self.menu_focus, cx);
        }
        cx.notify();
    }

    fn hover_tip(&mut self, key: SharedString, text: Option<SharedString>, hovered: bool) {
        match (hovered, text) {
            (true, Some(text)) => {
                self.tooltip = Some((key, text));
                self.tooltip_fade.set(1.0, Instant::now());
            }
            _ if self.tooltip.as_ref().is_some_and(|(k, _)| *k == key) => {
                self.tooltip_fade.set(0.0, Instant::now());
            }
            _ => {}
        }
    }

    fn menu_rows(&self) -> usize {
        match self.state.menu {
            Some(AppSidebarMenu::Switcher) => demo::PROJECTS.len() + demo::SWITCHER_ACTIONS.len(),
            Some(AppSidebarMenu::User) => demo::USER_MENU.iter().map(|g| g.len()).sum(),
            Some(AppSidebarMenu::Flyout(ix)) => {
                all_groups().nth(ix).map_or(0, |g| g.flyout().len())
            }
            None => 0,
        }
    }

    fn activate_highlight(&mut self, cx: &mut Context<Self>) {
        let Some(at) = self.state.highlight else {
            return;
        };
        match self.state.menu {
            Some(AppSidebarMenu::Switcher) => {
                if let Some(p) = demo::PROJECTS.get(at) {
                    self.state.project = p.value;
                }
                self.state.close_menu();
            }
            Some(AppSidebarMenu::Flyout(ix)) => {
                if let Some(link) = all_groups()
                    .nth(ix)
                    .and_then(|g| g.flyout().get(at).copied())
                {
                    self.navigate(link.href, cx);
                }
            }
            _ => self.state.close_menu(),
        }
        cx.notify();
    }
}

fn all_groups() -> impl Iterator<Item = &'static NavGroup> {
    demo::NAV.iter().chain(demo::FOOTER.iter())
}

fn glyph(icon: Icon, size: f32, color: Color) -> impl IntoElement {
    svg()
        .path(icon.path())
        .flex_none()
        .size(px(size))
        .text_color(color)
}

fn square(
    initial: &'static str,
    size: f32,
    radius: f32,
    font: f32,
    fill: Color,
) -> impl IntoElement {
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(size))
        .rounded(px(radius))
        .bg(fill)
        .text_size(px(font))
        .font_weight(FontWeight(600.0))
        .child(initial)
}

fn kbd(cap: &'static str, skin: &AppSidebarSkin) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .h(px(metrics::KBD_HEIGHT))
        .min_w(px(metrics::KBD_HEIGHT))
        .px(px(metrics::KBD_PADDING))
        .rounded(px(metrics::KBD_RADIUS))
        .bg(skin.kbd)
        .text_color(skin.kbd_text)
        .text_size(px(metrics::KBD_FONT))
        .line_height(px(16.0))
        .font_weight(FontWeight(500.0))
        .child(cap)
}

fn badge(text: &'static str, skin: &AppSidebarSkin) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .h(px(metrics::BADGE_HEIGHT))
        .px(px(metrics::BADGE_PADDING))
        .rounded_full()
        .bg(skin.badge)
        .text_color(skin.badge_text)
        .text_size(px(metrics::BADGE_FONT))
        .line_height(px(metrics::BADGE_HEIGHT))
        .font_weight(FontWeight(500.0))
        .child(text)
}

/// A dropdown row: icon or square, label, trailing caps or a check.
struct MenuRow {
    leading: Option<AnyElement>,
    label: SharedString,
    trailing: Option<AnyElement>,
    checked: bool,
    destructive: bool,
}

impl AppSidebar {
    fn menu_row(
        &self,
        ix: usize,
        r: MenuRow,
        skin: &AppSidebarSkin,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let highlighted = self.state.highlight == Some(ix);
        let (text, fill) = if r.destructive {
            (skin.destructive, skin.destructive_highlight)
        } else {
            (skin.menu_text, skin.menu_highlight)
        };
        let shadow = skin.menu_highlight_shadow;
        let check = r.checked.then(|| {
            div()
                .absolute()
                .right(px(metrics::MENU_ROW_PADDING_X))
                .top(px(8.0))
                .child(glyph(Icon::Check, 16.0, text))
        });
        div()
            .id(SharedString::from(format!("menu-row-{ix}")))
            .relative()
            .flex()
            .items_center()
            .gap(px(metrics::MENU_ROW_GAP))
            .h(px(metrics::MENU_ROW_HEIGHT))
            .px(px(metrics::MENU_ROW_PADDING_X))
            .when(
                r.checked
                    || ix < demo::PROJECTS.len()
                        && self.state.menu == Some(AppSidebarMenu::Switcher),
                |d| d.pr(px(metrics::MENU_INDICATOR_RESERVE)),
            )
            .rounded(px(metrics::MENU_ROW_RADIUS))
            .text_color(text)
            .cursor_pointer()
            .when(highlighted, |d| {
                d.bg(fill).when_some(shadow, |d, s| d.shadow(shadows(&[s])))
            })
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                if *hovered {
                    this.state.highlight = Some(ix);
                } else if this.state.highlight == Some(ix) {
                    this.state.highlight = None;
                }
                cx.notify();
            }))
            .on_click(cx.listener(move |this, _, window, cx| on_click(this, window, cx)))
            .children(r.leading)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(r.label),
            )
            .children(r.trailing)
            .children(check)
    }

    fn menu_separator(skin: &AppSidebarSkin) -> impl IntoElement {
        div()
            .h(px(1.0))
            .mx(px(metrics::MENU_SEPARATOR_MARGIN.0))
            .my(px(metrics::MENU_SEPARATOR_MARGIN.1))
            .bg(skin.menu_separator)
    }

    fn entry_row(
        &self,
        ix: usize,
        entry: MenuEntry,
        skin: &AppSidebarSkin,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let icon_color = if entry.destructive {
            skin.destructive
        } else if self.state.highlight == Some(ix) {
            skin.menu_text
        } else {
            skin.menu_icon
        };
        let caps = (!entry.keys.is_empty()).then(|| {
            div()
                .flex()
                .gap(px(metrics::KBD_GAP))
                .children(entry.keys.iter().map(|k| kbd(k, skin)))
                .into_any_element()
        });
        self.menu_row(
            ix,
            MenuRow {
                leading: Some(glyph(entry.icon, 16.0, icon_color).into_any_element()),
                label: entry.label.into(),
                trailing: caps,
                checked: false,
                destructive: entry.destructive,
            },
            skin,
            |this, _, cx| {
                this.state.close_menu();
                cx.notify();
            },
            cx,
        )
    }

    fn menu_body(
        &self,
        menu: AppSidebarMenu,
        skin: &AppSidebarSkin,
        cx: &mut Context<Self>,
    ) -> (f32, AnyElement) {
        let theme = *cx.notra();
        match menu {
            AppSidebarMenu::Switcher => {
                let mut rows: Vec<AnyElement> = vec![
                    div()
                        .h(px(metrics::MENU_LABEL_HEIGHT))
                        .px(px(metrics::MENU_ROW_PADDING_X))
                        .py(px(6.0))
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .font_weight(FontWeight(500.0))
                        .text_color(skin.menu_label)
                        .child(demo::SWITCHER_LABEL)
                        .into_any_element(),
                ];
                for (i, p) in demo::PROJECTS.iter().enumerate() {
                    let value = p.value;
                    rows.push(
                        self.menu_row(
                            i,
                            MenuRow {
                                leading: Some(
                                    square(
                                        p.initial,
                                        metrics::SWITCHER_ICON_SMALL,
                                        metrics::SWITCHER_ICON_SMALL_RADIUS,
                                        10.0,
                                        theme.sidebar_accent,
                                    )
                                    .into_any_element(),
                                ),
                                label: p.label.into(),
                                trailing: None,
                                checked: self.state.project == value,
                                destructive: false,
                            },
                            skin,
                            move |this, _, cx| {
                                this.state.project = value;
                                this.state.close_menu();
                                cx.notify();
                            },
                            cx,
                        )
                        .into_any_element(),
                    );
                }
                rows.push(Self::menu_separator(skin).into_any_element());
                for (i, entry) in demo::SWITCHER_ACTIONS.iter().enumerate() {
                    rows.push(
                        self.entry_row(demo::PROJECTS.len() + i, *entry, skin, cx)
                            .into_any_element(),
                    );
                }
                (
                    metrics::SWITCHER_MENU_WIDTH,
                    div().flex().flex_col().children(rows).into_any_element(),
                )
            }
            AppSidebarMenu::User => {
                let mut rows: Vec<AnyElement> = vec![
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .px(px(metrics::USER_HEADER_PADDING.0))
                        .py(px(metrics::USER_HEADER_PADDING.1))
                        .child(
                            div()
                                .text_size(px(14.0))
                                .line_height(px(20.0))
                                .font_weight(FontWeight(500.0))
                                .child(demo::USER_NAME),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(skin.menu_label)
                                .child(demo::USER_EMAIL),
                        )
                        .into_any_element(),
                    Self::menu_separator(skin).into_any_element(),
                ];
                let mut ix = 0;
                for (g, group) in demo::USER_MENU.iter().enumerate() {
                    if g > 0 {
                        rows.push(Self::menu_separator(skin).into_any_element());
                    }
                    for entry in group.iter() {
                        rows.push(self.entry_row(ix, *entry, skin, cx).into_any_element());
                        ix += 1;
                    }
                }
                (
                    metrics::USER_MENU_WIDTH,
                    div().flex().flex_col().children(rows).into_any_element(),
                )
            }
            AppSidebarMenu::Flyout(gi) => {
                let links = all_groups().nth(gi).map(|g| g.flyout()).unwrap_or_default();
                let active = self.state.active;
                let rows = links.into_iter().enumerate().map(|(i, link)| {
                    let href = link.href;
                    let trailing = link.badge.map(|b| badge(b, skin).into_any_element());
                    let current = href == active;
                    self.menu_row(
                        i,
                        MenuRow {
                            leading: None,
                            label: link.title.into(),
                            trailing,
                            checked: false,
                            destructive: false,
                        },
                        skin,
                        move |this, _, cx| this.navigate(href, cx),
                        cx,
                    )
                    .when(current, |d| d.font_weight(FontWeight(500.0)))
                    .into_any_element()
                });
                let rows: Vec<AnyElement> = rows.collect();
                (
                    metrics::FLYOUT_MIN_WIDTH,
                    div().flex().flex_col().children(rows).into_any_element(),
                )
            }
        }
    }

    fn menu_overlay(
        &self,
        skin: &AppSidebarSkin,
        now: Instant,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement + use<>> {
        let menu = self.shown_menu?;
        let fade = self.menu_fade.value(now);
        let (width, body) = self.menu_body(menu, skin, cx);
        let (anchor_key, gap): (SharedString, f32) = match menu {
            AppSidebarMenu::Switcher => ("switcher".into(), metrics::MENU_OFFSET),
            AppSidebarMenu::User => ("user".into(), metrics::USER_MENU_OFFSET),
            AppSidebarMenu::Flyout(ix) => (format!("nav-{ix}").into(), metrics::MENU_OFFSET),
        };
        let anchor = *self.bounds.get(&anchor_key)?;
        let collapsed = self.state.sidebar.collapsed();
        let anchor_w = f32::from(anchor.size.width);
        let width = width.max(if matches!(menu, AppSidebarMenu::User) {
            0.0
        } else {
            anchor_w
        });
        let (position, corner) = match menu {
            AppSidebarMenu::User if !collapsed => (
                point(anchor.origin.x, anchor.origin.y - px(gap)),
                ::gpui::Anchor::BottomLeft,
            ),
            AppSidebarMenu::Switcher => (
                point(
                    anchor.origin.x,
                    anchor.origin.y + anchor.size.height + px(gap),
                ),
                ::gpui::Anchor::TopLeft,
            ),
            _ => (
                point(
                    anchor.origin.x + anchor.size.width + px(gap),
                    anchor.origin.y,
                ),
                ::gpui::Anchor::TopLeft,
            ),
        };
        let focus = self.menu_focus.clone();
        let rows = self.menu_rows();
        let panel = div()
            .id("app-sidebar-menu")
            .track_focus(&focus)
            .occlude()
            .w(px(width))
            .p(px(metrics::MENU_PADDING))
            .rounded(px(metrics::MENU_RADIUS))
            .border_1()
            .border_color(skin.menu_border)
            .bg(gradient(skin.menu_fill.0, skin.menu_fill.1))
            .shadow(shadows(&skin.menu_shadows))
            .text_color(skin.menu_text)
            .text_size(px(14.0))
            .line_height(px(20.0))
            .font_family(fonts::SANS)
            .opacity(fade)
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                this.state.close_menu();
                cx.notify();
            }))
            .on_key_down(
                cx.listener(move |this, event: &::gpui::KeyDownEvent, _, cx| {
                    match event.keystroke.key.as_str() {
                        "escape" => this.state.close_menu(),
                        "down" => this.state.move_highlight(1, rows),
                        "up" => this.state.move_highlight(-1, rows),
                        "enter" | "space" => this.activate_highlight(cx),
                        _ => return,
                    }
                    cx.stop_propagation();
                    cx.notify();
                }),
            )
            .child(body);
        Some(
            deferred(
                anchored()
                    .position(position)
                    .anchor(corner)
                    .snap_to_window_with_margin(px(8.0))
                    .child(panel),
            )
            .with_priority(1),
        )
    }

    fn tooltip_overlay(
        &self,
        skin: &AppSidebarSkin,
        now: Instant,
    ) -> Option<impl IntoElement + use<>> {
        let (key, text) = self.tooltip.clone()?;
        let anchor = *self.bounds.get(&key)?;
        let fade = self.tooltip_fade.value(now);
        if fade <= 0.0 {
            return None;
        }
        let height = 16.0 + 2.0 * metrics::TOOLTIP_PADDING.1 + 2.0;
        let x = anchor.origin.x
            + anchor.size.width
            + px(metrics::TOOLTIP_OFFSET - motion::TOOLTIP_DRIFT * (1.0 - fade));
        let y = anchor.origin.y + anchor.size.height / 2.0 - px(height / 2.0);
        let arrow_color: Hsla = skin.tooltip_fill.1.mix(skin.tooltip_fill.0, 0.5).into();
        let arrow = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let mid = bounds.origin.y + bounds.size.height / 2.0;
                let x0 = bounds.origin.x;
                let mut path = PathBuilder::fill();
                path.move_to(point(x0 + px(1.0), mid - px(5.0)));
                path.line_to(point(x0 - px(4.0), mid));
                path.line_to(point(x0 + px(1.0), mid + px(5.0)));
                path.close();
                if let Ok(path) = path.build() {
                    window.paint_path(path, arrow_color);
                }
            },
        )
        .absolute()
        .size_full();
        let tip = div()
            .relative()
            .px(px(metrics::TOOLTIP_PADDING.0))
            .py(px(metrics::TOOLTIP_PADDING.1))
            .rounded(px(metrics::TOOLTIP_RADIUS))
            .border_1()
            .border_color(skin.tooltip_border)
            .bg(gradient(skin.tooltip_fill.0, skin.tooltip_fill.1))
            .shadow(shadows(&skin.tooltip_shadows))
            .text_color(skin.tooltip_text)
            .text_size(px(12.0))
            .line_height(px(16.0))
            .font_family(fonts::SANS)
            .whitespace_nowrap()
            .opacity(fade)
            .child(arrow)
            .child(text);
        Some(deferred(anchored().position(point(x, y)).child(tip)).with_priority(2))
    }
}

impl Render for AppSidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.notra();
        let skin = AppSidebarSkin::new(&theme);
        let now = Instant::now();
        let reduce = cx.reduce_motion();
        let frame = self.motion.frame(
            &self.state.sidebar,
            self.variant,
            self.collapsible,
            &theme,
            now,
            reduce,
        );
        let sb = frame.skin.clone();
        let collapsed = frame.collapsed;

        if self.state.menu.is_some() {
            self.shown_menu = self.state.menu;
            self.menu_fade = Animated::new(self.menu_fade.value(now), motion::MENU_IN);
            self.menu_fade.set(1.0, now);
        } else if self.menu_fade.target() > 0.0 {
            self.menu_fade = Animated::new(self.menu_fade.value(now), motion::MENU_OUT);
            self.menu_fade.set(0.0, now);
        } else if !self.menu_fade.is_animating(now) {
            self.shown_menu = None;
        }
        if !collapsed {
            self.tooltip = None;
        }
        if self.tooltip_fade.target() == 0.0 && !self.tooltip_fade.is_animating(now) {
            self.tooltip = None;
        }
        self.ask_width.set(if collapsed { 0.0 } else { 1.0 }, now);
        if reduce {
            for a in [
                &mut self.menu_fade,
                &mut self.tooltip_fade,
                &mut self.ask_width,
            ] {
                a.snap(a.target());
            }
            for p in self.panels.values_mut() {
                p.snap(p.target());
            }
        }
        let max = self.scroll.max_offset().y;
        let has_more = -self.scroll.offset().y < max - px(1.0);
        self.state.has_more = has_more;
        let animating = self.motion.is_animating(now)
            || self.menu_fade.is_animating(now)
            || self.tooltip_fade.is_animating(now)
            || self.ask_width.is_animating(now)
            || self.panels.values().any(|p| p.is_animating(now));
        if animating {
            window.request_animation_frame();
        }

        let gutter = frame.variant.gutter();
        let inner = frame.inner_width();
        let c = frame.collapse;
        let q = frame.quick;
        let lg_right = row::ROW_PADDING * (1.0 - q);
        let lg_left = |rail: f32| row::ROW_PADDING + (rail - row::ROW_PADDING) * q;
        let rail_content = frame.variant.rail_content();
        let tips = collapsed;

        // Header: the switcher, with the trigger beside it (below it on the rail).
        let project = demo::PROJECTS
            .iter()
            .find(|p| p.value == self.state.project)
            .copied()
            .unwrap_or(demo::PROJECTS[0]);
        let switcher_open = self.state.menu == Some(AppSidebarMenu::Switcher);
        let header_row_w = inner - 2.0 * row::SECTION_PADDING;
        let trigger_left =
            (header_row_w - row::TRIGGER) * (1.0 - c) + metrics::TRIGGER_RAIL_LEFT * c;
        let trigger_left = trigger_left.max(0.0);
        let trigger_top =
            metrics::TRIGGER_TOP.0 + (metrics::TRIGGER_TOP.1 - metrics::TRIGGER_TOP.0) * c;
        let switcher_pl = lg_left((rail_content - metrics::SWITCHER_ICON) / 2.0);
        let switcher = div()
            .id("switcher-wrap")
            .relative()
            .on_hover(cx.listener(move |this, h: &bool, _, cx| {
                this.hover_tip("switcher".into(), tips.then(|| project.label.into()), *h);
                cx.notify();
            }))
            .child(self.measure("switcher".into(), cx))
            .child(
                SidebarMenuButton::new("switcher", &frame)
                    .size(MenuButtonSize::Lg)
                    .active(switcher_open)
                    .leading(square(
                        project.initial,
                        metrics::SWITCHER_ICON,
                        metrics::SWITCHER_ICON_RADIUS,
                        12.0,
                        theme.sidebar_accent,
                    ))
                    .label_element(
                        div()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .line_height(px(20.0))
                                    .font_weight(FontWeight(500.0))
                                    .text_color(sb.foreground)
                                    .child(project.label),
                            )
                            .children(project.description.map(|d| {
                                div()
                                    .text_size(px(12.0))
                                    .line_height(px(16.0))
                                    .font_weight(FontWeight(400.0))
                                    .text_color(sb.muted_foreground)
                                    .child(d)
                            })),
                    )
                    .trailing(glyph(Icon::HugeUnfoldMore, 16.0, sb.muted_foreground))
                    .on_click({
                        let this = cx.entity().downgrade();
                        move |window, cx| {
                            this.update(cx, |this, cx| {
                                this.toggle_menu(AppSidebarMenu::Switcher, window, cx)
                            })
                            .ok();
                        }
                    }),
            );
        let switcher = div()
            .child(switcher)
            .pl(px(switcher_pl - row::ROW_PADDING))
            .pr(px(lg_right - row::ROW_PADDING));
        let toggle = {
            let this = cx.entity().downgrade();
            SidebarTrigger::new("sidebar-trigger", !collapsed, move |_, cx| {
                this.update(cx, |this, cx| {
                    this.state.toggle_sidebar();
                    cx.notify();
                })
                .ok();
            })
        };
        let header = div()
            .pt(px(metrics::HEADER_PADDING.0))
            .pb(px(metrics::HEADER_PADDING.1))
            .px(px(row::SECTION_PADDING))
            .overflow_hidden()
            .child(
                div()
                    .relative()
                    .pr(px(metrics::HEADER_RESERVE * (1.0 - c)))
                    .pb(px(metrics::HEADER_RESERVE * c))
                    .child(switcher)
                    .child(
                        div()
                            .absolute()
                            .left(px(trigger_left))
                            .top(px(trigger_top))
                            .child(toggle),
                    ),
            );

        // Search: a field-like row that becomes a plain icon on the rail.
        let search_h =
            metrics::SEARCH_HEIGHT.0 + (metrics::SEARCH_HEIGHT.1 - metrics::SEARCH_HEIGHT.0) * q;
        let search_text = if collapsed {
            sb.foreground
        } else {
            sb.muted_foreground
        };
        let search_hover: Hsla = sb.foreground.into();
        let search = div()
            .px(px(metrics::SEARCH_PADDING.0))
            .pt(px(metrics::SEARCH_PADDING.1))
            .pb(px(metrics::SEARCH_PADDING.2))
            .child(
                div()
                    .id("search")
                    .relative()
                    .flex()
                    .items_center()
                    .gap(px(row::ROW_GAP))
                    .h(px(search_h))
                    .w_full()
                    .p(px(row::ROW_PADDING))
                    .overflow_hidden()
                    .rounded(px(metrics::SEARCH_RADIUS))
                    .border_1()
                    .border_color(if collapsed {
                        Color::TRANSPARENT
                    } else {
                        sb.border
                    })
                    .text_color(search_text)
                    .text_size(px(14.0))
                    .line_height(px(20.0))
                    .cursor_pointer()
                    .hover(move |s| s.bg(sb.accent).text_color(search_hover))
                    .on_hover(cx.listener(move |this, h: &bool, _, cx| {
                        this.hover_tip(
                            "search".into(),
                            tips.then(|| demo::SEARCH_LABEL.into()),
                            *h,
                        );
                        cx.notify();
                    }))
                    .child(self.measure("search".into(), cx))
                    .child(glyph(Icon::HugeSearch, 16.0, search_text))
                    .child(
                        div()
                            .flex_1()
                            .whitespace_nowrap()
                            .relative()
                            .left(px(crate::components::navigation::sidebar::LABEL_DRIFT
                                * (1.0 - frame.labels)))
                            .opacity(frame.labels)
                            .child(format!("{}…", demo::SEARCH_LABEL)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .h(px(metrics::KBD_HEIGHT))
                            .px(px(metrics::SEARCH_KBD_PADDING))
                            .rounded(px(metrics::SEARCH_KBD_RADIUS))
                            .border_1()
                            .border_color(sb.border)
                            .text_size(px(metrics::KBD_FONT))
                            .line_height(px(16.0))
                            .font_weight(FontWeight(500.0))
                            .text_color(sb.muted_foreground)
                            .opacity(frame.labels)
                            .child(demo::SEARCH_SHORTCUT),
                    ),
            );

        // Navigation groups.
        let nav_row =
            |this: &Self, gi: usize, g: &'static NavGroup, cx: &mut Context<Self>| -> AnyElement {
                let active = this.state.active;
                let flyout = frame.settled && !g.items.is_empty();
                let group_active = g.is_active(active);
                let own = g.own_page_active(active);
                let key: SharedString = format!("nav-{gi}").into();
                let tip = (tips && g.items.is_empty()).then(|| SharedString::from(g.title));
                let hover_key = key.clone();
                let mut button = SidebarMenuButton::new(key.clone(), &frame)
                    .icon(g.icon)
                    .label(g.title)
                    .active(if flyout {
                        group_active || this.state.menu == Some(AppSidebarMenu::Flyout(gi))
                    } else {
                        own
                    })
                    .strong(!flyout && group_active && !own);
                if let Some(b) = g.row_badge(active) {
                    button = button.trailing(div().ml_auto().child(badge(b, &skin)));
                }
                let weak = cx.entity().downgrade();
                let href = g.href;
                button = button.on_click(move |window, cx| {
                    weak.update(cx, |this, cx| {
                        if flyout {
                            this.toggle_menu(AppSidebarMenu::Flyout(gi), window, cx);
                        } else {
                            this.navigate(href, cx);
                        }
                    })
                    .ok();
                });
                let panel = (!flyout && !g.items.is_empty()).then(|| {
                    let open = this.panels.get(g.href).map_or(0.0, |p| p.value(now));
                    let fold = 1.0 - c;
                    let n = g.items.len() as f32;
                    let full = n * row::SUB_ROW_HEIGHT
                        + (n - 1.0) * row::MENU_GAP
                        + 2.0 * row::SUB_PADDING_Y;
                    let subs = g.items.iter().map(|item| {
                        let href = item.href;
                        let current = href == active;
                        let accent_text: Hsla = sb.accent_foreground.into();
                        div()
                            .id(SharedString::from(format!("sub-{href}")))
                            .flex()
                            .items_center()
                            .gap(px(row::ROW_GAP))
                            .h(px(row::SUB_ROW_HEIGHT))
                            .px(px(row::SUB_ROW_PADDING_X))
                            .rounded(px(row::ROW_RADIUS))
                            .text_size(px(14.0))
                            .line_height(px(20.0))
                            .text_color(if current {
                                sb.accent_foreground
                            } else {
                                sb.foreground
                            })
                            .when(current, |d| d.bg(sb.accent))
                            .cursor_pointer()
                            .hover(move |s| s.bg(sb.accent).text_color(accent_text))
                            .on_click(cx.listener(move |this, _, _, cx| this.navigate(href, cx)))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .child(item.title),
                            )
                            .children(item.badge.map(|b| badge(b, &skin)))
                    });
                    div()
                        .h(px(full * open * fold))
                        .opacity(open * fold)
                        .overflow_hidden()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(row::MENU_GAP))
                                .mx(px(row::SUB_MARGIN_X))
                                .px(px(row::SUB_PADDING_X))
                                .py(px(row::SUB_PADDING_Y))
                                .border_l_1()
                                .border_color(sb.border)
                                .children(subs),
                        )
                });
                div()
                    .id(SharedString::from(format!("{key}-wrap")))
                    .relative()
                    .flex()
                    .flex_col()
                    .on_hover(cx.listener(move |this, h: &bool, _, cx| {
                        this.hover_tip(hover_key.clone(), tip.clone(), *h);
                        cx.notify();
                    }))
                    .child(div().relative().child(this.measure(key, cx)).child(button))
                    .children(panel)
                    .into_any_element()
            };
        let main_rows: Vec<AnyElement> = demo::NAV
            .iter()
            .enumerate()
            .map(|(gi, g)| nav_row(self, gi, g, cx))
            .collect();
        let footer_rows: Vec<AnyElement> = demo::FOOTER
            .iter()
            .enumerate()
            .map(|(i, g)| nav_row(self, demo::NAV.len() + i, g, cx))
            .collect();
        let docs = div()
            .id("docs-wrap")
            .relative()
            .on_hover(cx.listener(move |this, h: &bool, _, cx| {
                this.hover_tip("docs".into(), tips.then(|| demo::DOCS.title.into()), *h);
                cx.notify();
            }))
            .child(self.measure("docs".into(), cx))
            .child(
                SidebarMenuButton::new("docs", &frame)
                    .icon(demo::DOCS_ICON)
                    .label(demo::DOCS.title)
                    .on_click(|_, cx| cx.open_url(demo::DOCS.href)),
            );
        let ask_w = row::ICON * 2.0
            + (inner - 2.0 * row::SECTION_PADDING - 32.0) * self.ask_width.value(now);
        let ask_hover = theme.muted;
        let ask = div().pt(px(metrics::ASK_TOP)).child(
            div()
                .id("ask-agent")
                .flex()
                .items_center()
                .justify_center()
                .gap(px(6.0))
                .h(px(32.0))
                .w(px(ask_w.max(32.0)))
                .when(!collapsed, |d| d.px(px(10.0)))
                .rounded(px(8.0))
                .border_1()
                .border_color(skin.ask_border)
                .bg(skin.ask)
                .text_color(sb.foreground)
                .text_size(px(14.0))
                .line_height(px(20.0))
                .font_weight(FontWeight(500.0))
                .overflow_hidden()
                .whitespace_nowrap()
                .cursor_pointer()
                .hover(move |s| s.bg(ask_hover))
                .child(glyph(demo::ASK_ICON, 16.0, sb.foreground))
                .when(!collapsed, |d| d.child(demo::ASK_LABEL)),
        );
        let content = div()
            .id("sidebar-content")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_x_hidden()
            .when(!collapsed, |d| d.overflow_y_scroll())
            .track_scroll(&self.scroll)
            .on_scroll_wheel(cx.listener(|_, _, _, cx| cx.notify()))
            .child(
                div().flex().flex_col().p(px(row::SECTION_PADDING)).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(row::MENU_GAP))
                        .children(main_rows),
                ),
            )
            .child(
                div()
                    .mt_auto()
                    .flex()
                    .flex_col()
                    .gap(px(metrics::FOOTER_GROUP_GAP))
                    .pt(px(row::SECTION_PADDING))
                    .px(px(row::SECTION_PADDING))
                    .children(footer_rows)
                    .child(docs)
                    .child(ask),
            );
        let fade = (has_more && !collapsed).then(|| {
            div().relative().flex_none().child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .h(px(metrics::SCROLL_FADE))
                    .bg(linear_gradient(
                        0.0,
                        linear_color_stop(sb.background, 0.0),
                        linear_color_stop(sb.background.alpha(0.0), 1.0),
                    )),
            )
        });

        // Footer: the account row.
        let user_open = self.state.menu == Some(AppSidebarMenu::User);
        let user_pl = lg_left((rail_content - metrics::AVATAR) / 2.0);
        let avatar = div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(metrics::AVATAR))
            .rounded(px(metrics::AVATAR_RADIUS))
            .bg(sb.accent)
            .border_1()
            .border_color(skin.avatar_border)
            .text_color(sb.muted_foreground)
            .text_size(px(14.0))
            .child(
                demo::USER_NAME
                    .trim()
                    .chars()
                    .next()
                    .map(|ch| ch.to_uppercase().to_string())
                    .unwrap_or_default(),
            );
        let user = div()
            .id("user-wrap")
            .relative()
            .rounded(px(row::ROW_RADIUS))
            .when(user_open && !collapsed, |d| {
                d.bg(skin.user_open).shadow(shadows(&[Shadow::drop(
                    0.0,
                    0.0,
                    1.0,
                    skin.user_open_ring,
                )]))
            })
            .on_hover(cx.listener(move |this, h: &bool, _, cx| {
                this.hover_tip("user".into(), tips.then(|| demo::USER_NAME.into()), *h);
                cx.notify();
            }))
            .child(self.measure("user".into(), cx))
            .child(
                div()
                    .pl(px(user_pl - row::ROW_PADDING))
                    .pr(px(lg_right - row::ROW_PADDING))
                    .child(
                        SidebarMenuButton::new("user", &frame)
                            .size(MenuButtonSize::Lg)
                            .leading(avatar)
                            .label_element(
                                div()
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .text_ellipsis()
                                    .font_weight(FontWeight(500.0))
                                    .child(demo::USER_NAME),
                            )
                            .on_click({
                                let this = cx.entity().downgrade();
                                move |window, cx| {
                                    this.update(cx, |this, cx| {
                                        this.toggle_menu(AppSidebarMenu::User, window, cx)
                                    })
                                    .ok();
                                }
                            }),
                    ),
            );
        let footer = div()
            .pt(px(metrics::FOOTER_TOP))
            .px(px(row::SECTION_PADDING))
            .pb(px(row::SECTION_PADDING))
            .child(user);

        let rail = {
            let this = cx.entity().downgrade();
            SidebarRail::new(
                "sidebar-rail",
                &self.rail_focus,
                &self.state.sidebar,
                move |event, _, cx| {
                    this.update(cx, |this, cx| {
                        event.apply(&mut this.state.sidebar);
                        if !matches!(event, RailEvent::Move(_)) || this.state.sidebar.resizing {
                            this.state.close_menu();
                        }
                        cx.notify();
                    })
                    .ok();
                },
            )
        };

        let panel = div()
            .relative()
            .flex_none()
            .h_full()
            .w(px(frame.width))
            .p(px(gutter))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .overflow_hidden()
                    .bg(sb.background)
                    .text_color(sb.foreground)
                    .font_family(fonts::SANS)
                    .child(header)
                    .child(search)
                    .child(content)
                    .children(fade)
                    .child(footer),
            )
            .child(rail);

        let page = self
            .inset
            .as_ref()
            .map(|build| build(&self.state, window, cx));
        let menu = self.menu_overlay(&skin, now, cx);
        let tooltip = self.tooltip_overlay(&skin, now);

        div()
            .relative()
            .flex()
            .size_full()
            .overflow_hidden()
            .bg(sb.background)
            .child(panel)
            .child(SidebarInset::new().children(page))
            .children(menu)
            .children(tooltip)
    }
}

/// The inset page of the registry demo: a titled header over placeholder cards.
pub fn demo_page(state: &AppSidebarState, cx: &App) -> AnyElement {
    let theme = cx.notra();
    let skin = AppSidebarSkin::new(theme);
    let card = |tall: Option<f32>| {
        div()
            .flex_1()
            .map(|d| match tall {
                Some(h) => d.h(px(h)),
                None => d.aspect_ratio(16.0 / 9.0),
            })
            .rounded(px(demo::CARD_RADIUS))
            .bg(skin.placeholder)
    };
    div()
        .flex()
        .flex_col()
        .size_full()
        .child(
            div()
                .flex()
                .flex_none()
                .items_center()
                .h(px(demo::HEADER_HEIGHT))
                .px(px(16.0))
                .border_b_1()
                .border_color(theme.border)
                .text_size(px(14.0))
                .line_height(px(20.0))
                .font_weight(FontWeight(500.0))
                .text_color(theme.foreground)
                .child(demo::page_title(state.active)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(demo::PAGE_PADDING))
                .p(px(demo::PAGE_PADDING))
                .child(
                    div()
                        .flex()
                        .gap(px(demo::PAGE_PADDING))
                        .child(card(None))
                        .child(card(None))
                        .child(card(None)),
                )
                .child(card(Some(demo::CARD_TALL))),
        )
        .into_any_element()
}
