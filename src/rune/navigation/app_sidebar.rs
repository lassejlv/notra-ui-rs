use std::f32::consts::FRAC_PI_4;

use ::rune::{Align, Drag, El, Key, Phase, ScrollSpec, Side, Tx, col, el, icon, row, text};
use ::rune_motion::{Spring, SpringPair, Transition};

use crate::components::navigation::app_sidebar::{
    AppSidebarMenu, AppSidebarSkin, AppSidebarState, MenuEntry, NavGroup, demo, metrics, motion,
};
use crate::components::navigation::sidebar::{
    COLLAPSE, Collapsible, LABEL_DRIFT, LABEL_FADE, MenuButtonSize, QUICK, RailKey, SUB_PANEL,
    SidebarSkin, SidebarVariant, metrics as row_m,
};
use crate::icons::Icon;
use crate::rune::{ToRune, sans, tween};
use crate::theme::{Color, Shadow, Theme};

/// Reaches the block's state inside the app state.
pub type SidebarLens<S> = fn(&mut S) -> &mut AppSidebarState;

/// The Notra dashboard sidebar block beside its inset page:
/// `AppSidebar::new("app", &s.sidebar).inset(page).build(&theme, |s: &mut State| &mut s.sidebar)`.
pub struct AppSidebar<'a> {
    key: String,
    state: &'a AppSidebarState,
    inset: Option<El>,
}

fn seconds(t: crate::motion::Tween) -> f32 {
    t.duration.as_secs_f32()
}

fn label(content: impl Into<String>, size: f32, weight: f32, line: f32, color: Color) -> El {
    text(content)
        .font(sans(size, weight, line))
        .color(color.rune())
        .single_line()
}

fn glyph(i: Icon, size: f32, color: Color) -> El {
    icon(i.rune(), size).color(color.rune())
}

fn square(initial: &str, size: f32, radius: f32, font: f32, fill: Color, ink: Color) -> El {
    row()
        .size(size, size)
        .center_y()
        .justify_center()
        .radius(radius)
        .fill(fill.rune())
        .child(label(initial, font, 600.0, font + 4.0, ink))
}

fn kbd(cap: &str, skin: &AppSidebarSkin) -> El {
    row()
        .h(metrics::KBD_HEIGHT)
        .min_w(metrics::KBD_HEIGHT)
        .px(metrics::KBD_PADDING)
        .center_y()
        .justify_center()
        .radius(metrics::KBD_RADIUS)
        .fill(skin.kbd.rune())
        .child(label(cap, metrics::KBD_FONT, 500.0, 16.0, skin.kbd_text))
}

fn badge(content: &str, skin: &AppSidebarSkin) -> El {
    row()
        .h(metrics::BADGE_HEIGHT)
        .px(metrics::BADGE_PADDING)
        .center_y()
        .radius(metrics::BADGE_HEIGHT / 2.0)
        .fill(skin.badge.rune())
        .child(label(
            content,
            metrics::BADGE_FONT,
            500.0,
            metrics::BADGE_HEIGHT,
            skin.badge_text,
        ))
}

/// A sidebar row's look, before handlers.
struct Row {
    key: String,
    leading: El,
    body: El,
    trailing: Vec<El>,
    size: MenuButtonSize,
    active: bool,
    collapsed: bool,
}

impl Row {
    fn build(self, sb: &SidebarSkin) -> El {
        let fade = tween(LABEL_FADE);
        let shown = if self.collapsed { 0.0 } else { 1.0 };
        let drift = if self.collapsed { LABEL_DRIFT } else { 0.0 };
        let mut r = row()
            .key(self.key)
            .center_y()
            .gap(row_m::ROW_GAP)
            .w_full()
            .h(self.size.height())
            .padding(row_m::ROW_PADDING)
            .radius(row_m::ROW_RADIUS)
            .clip_rounded(row_m::ROW_RADIUS)
            .hover(sb.accent.rune())
            .hover_group()
            .focusable(true)
            .focus_ring(2.0, -2.0, sb.ring.rune())
            .cursor(::rune::Cursor::Pointer)
            .child(self.leading)
            .child(
                el().grow(1.0)
                    .min_w(0.0)
                    .opacity(shown)
                    .opacity_motion(fade)
                    .offset(drift, 0.0)
                    .offset_motion(fade)
                    .child(self.body),
            )
            .children(
                self.trailing
                    .into_iter()
                    .map(|t| t.opacity(shown).opacity_motion(fade)),
            );
        if self.active {
            r = r.fill(sb.accent.rune());
        }
        r
    }
}

impl<'a> AppSidebar<'a> {
    pub fn new(key: impl Into<String>, state: &'a AppSidebarState) -> Self {
        Self {
            key: key.into(),
            state,
            inset: None,
        }
    }

    /// The page shown in the inset beside the sidebar.
    pub fn inset(mut self, page: El) -> Self {
        self.inset = Some(page);
        self
    }

    fn text_color(sb: &SidebarSkin, active: bool, strong: bool) -> Color {
        if active || strong {
            sb.strong
        } else {
            sb.foreground
        }
    }

    pub fn build<S: 'static>(self, theme: &Theme, lens: SidebarLens<S>) -> El {
        let sb = SidebarSkin::new(theme);
        let skin = AppSidebarSkin::new(theme);
        let state = self.state;
        let key = self.key;
        let variant = SidebarVariant::Inset;
        let collapsed = state.sidebar.collapsed();
        let outer = state.sidebar.outer_width(variant, Collapsible::Icon);
        let inner = outer - 2.0 * variant.gutter();
        let row_w = inner - 2.0 * row_m::SECTION_PADDING;
        let rail_content = variant.rail_content();
        let slide = Spring::new(seconds(COLLAPSE), 0.0);
        let quick = Spring::new(seconds(QUICK), 0.0);
        let k = |part: &str| format!("{key}/{part}");
        let hover_key = |name: &'static str| {
            move |s: &mut S, hovered: bool, _: &mut Tx| {
                let st = lens(s);
                if hovered {
                    st.hovered = Some(name);
                } else if st.hovered == Some(name) {
                    st.hovered = None;
                }
            }
        };

        // Header: the switcher, with the trigger beside it (below it on the rail).
        let project = demo::PROJECTS
            .iter()
            .find(|p| p.value == state.project)
            .copied()
            .unwrap_or(demo::PROJECTS[0]);
        let lg_left = |icon: f32| {
            if collapsed {
                (rail_content - icon) / 2.0
            } else {
                row_m::ROW_PADDING
            }
        };
        let lg_pad = if collapsed { 0.0 } else { row_m::ROW_PADDING };
        let switcher = Row {
            key: k("switcher"),
            leading: square(
                project.initial,
                metrics::SWITCHER_ICON,
                metrics::SWITCHER_ICON_RADIUS,
                12.0,
                theme.sidebar_accent,
                sb.foreground,
            ),
            body: col()
                .child(label(project.label, 14.0, 500.0, 20.0, sb.foreground).truncate())
                .children(
                    project
                        .description
                        .map(|d| label(d, 12.0, 400.0, 16.0, sb.muted_foreground).truncate()),
                ),
            trailing: vec![glyph(Icon::HugeUnfoldMore, 16.0, sb.muted_foreground)],
            size: MenuButtonSize::Lg,
            active: state.menu == Some(AppSidebarMenu::Switcher),
            collapsed,
        }
        .build(&sb)
        .pl(lg_left(metrics::SWITCHER_ICON))
        .pr(lg_pad)
        .py(lg_pad)
        .motion(quick)
        .role("button")
        .label(project.label)
        .expanded(state.menu == Some(AppSidebarMenu::Switcher))
        .on_hover_change(hover_key("switcher"))
        .on_press(move |s: &mut S| lens(s).toggle_menu(AppSidebarMenu::Switcher));

        let trigger_left = if collapsed {
            metrics::TRIGGER_RAIL_LEFT
        } else {
            row_w - row_m::TRIGGER
        };
        let trigger_top = if collapsed {
            metrics::TRIGGER_TOP.1
        } else {
            metrics::TRIGGER_TOP.0
        };
        let trigger = row()
            .key(k("trigger"))
            .absolute(trigger_left, trigger_top)
            .motion(slide)
            .size(row_m::TRIGGER, row_m::TRIGGER)
            .center_y()
            .justify_center()
            .radius(row_m::TRIGGER_RADIUS)
            .hover_group()
            .role("button")
            .label(if collapsed {
                "Show sidebar"
            } else {
                "Hide sidebar"
            })
            .expanded(!collapsed)
            .focusable(true)
            .cursor(::rune::Cursor::Pointer)
            .on_press(move |s: &mut S| lens(s).toggle_sidebar())
            .child(
                glyph(
                    if collapsed {
                        Icon::HugeSidebarRight01
                    } else {
                        Icon::HugeSidebarLeft01
                    },
                    16.0,
                    sb.trigger,
                )
                .group_color(sb.trigger_hover.rune()),
            );
        let reserve = |on: bool| if on { metrics::HEADER_RESERVE } else { 0.0 };
        let header = col()
            .pt(metrics::HEADER_PADDING.0)
            .pb(metrics::HEADER_PADDING.1)
            .px(row_m::SECTION_PADDING)
            .child(
                el().key(k("header-row"))
                    .pr(reserve(!collapsed))
                    .pb(reserve(collapsed))
                    .motion(slide)
                    .child(switcher)
                    .child(trigger),
            );

        // Search: a field-like row that becomes a plain icon on the rail.
        let search_ink = if collapsed {
            sb.foreground
        } else {
            sb.muted_foreground
        };
        let fade = tween(LABEL_FADE);
        let shown = if collapsed { 0.0 } else { 1.0 };
        let search = col()
            .px(metrics::SEARCH_PADDING.0)
            .pt(metrics::SEARCH_PADDING.1)
            .pb(metrics::SEARCH_PADDING.2)
            .child(
                row()
                    .key(k("search"))
                    .center_y()
                    .gap(row_m::ROW_GAP)
                    .w_full()
                    .h(if collapsed {
                        metrics::SEARCH_HEIGHT.1
                    } else {
                        metrics::SEARCH_HEIGHT.0
                    })
                    .motion(quick)
                    .padding(row_m::ROW_PADDING)
                    .radius(metrics::SEARCH_RADIUS)
                    .clip_rounded(metrics::SEARCH_RADIUS)
                    .border(
                        1.0,
                        if collapsed {
                            Color::TRANSPARENT
                        } else {
                            sb.border
                        }
                        .rune(),
                    )
                    .hover(sb.accent.rune())
                    .hover_group()
                    .role("button")
                    .label(demo::SEARCH_LABEL)
                    .shortcut_hint("Meta+K")
                    .focusable(true)
                    .cursor(::rune::Cursor::Pointer)
                    .on_hover_change(hover_key("search"))
                    .child(
                        glyph(Icon::HugeSearch, 16.0, search_ink).group_color(sb.foreground.rune()),
                    )
                    .child(
                        el().grow(1.0)
                            .min_w(0.0)
                            .opacity(shown)
                            .opacity_motion(fade)
                            .offset(if collapsed { LABEL_DRIFT } else { 0.0 }, 0.0)
                            .offset_motion(fade)
                            .child(
                                label(
                                    format!("{}…", demo::SEARCH_LABEL),
                                    14.0,
                                    400.0,
                                    20.0,
                                    search_ink,
                                )
                                .group_color(sb.foreground.rune()),
                            ),
                    )
                    .child(
                        row()
                            .h(metrics::KBD_HEIGHT)
                            .px(metrics::SEARCH_KBD_PADDING)
                            .center_y()
                            .radius(metrics::SEARCH_KBD_RADIUS)
                            .border(1.0, sb.border.rune())
                            .opacity(shown)
                            .opacity_motion(fade)
                            .child(label(
                                demo::SEARCH_SHORTCUT,
                                metrics::KBD_FONT,
                                500.0,
                                16.0,
                                sb.muted_foreground,
                            )),
                    ),
            );

        // Navigation.
        let sub_open = tween(SUB_PANEL);
        let nav_row = |gi: usize, g: &'static NavGroup| -> El {
            let active = state.active;
            let flyout = collapsed && !g.items.is_empty();
            let group_active = g.is_active(active);
            let own = g.own_page_active(active);
            let open_menu = state.menu == Some(AppSidebarMenu::Flyout(gi));
            let is_active = if flyout {
                group_active || open_menu
            } else {
                own
            };
            let strong = !flyout && group_active && !own;
            let ink = Self::text_color(&sb, is_active, strong);
            let weight = if is_active || strong { 500.0 } else { 400.0 };
            let href = g.href;
            let tip: &'static str = g.title;
            let mut button = Row {
                key: k(&format!("nav-{gi}")),
                leading: glyph(g.icon, 16.0, ink).group_color(sb.accent_foreground.rune()),
                body: label(g.title, 14.0, weight, 20.0, ink)
                    .group_color(sb.accent_foreground.rune())
                    .truncate(),
                trailing: g
                    .row_badge(active)
                    .map(|b| badge(b, &skin))
                    .into_iter()
                    .collect(),
                size: MenuButtonSize::Default,
                active: is_active,
                collapsed,
            }
            .build(&sb)
            .role(if flyout { "button" } else { "link" })
            .label(g.title);
            button = if flyout {
                button
                    .expanded(open_menu)
                    .on_press(move |s: &mut S| lens(s).toggle_menu(AppSidebarMenu::Flyout(gi)))
            } else {
                button.on_press(move |s: &mut S| lens(s).navigate(href))
            };
            if g.items.is_empty() {
                button = button.on_hover_change(move |s: &mut S, hovered: bool, _: &mut Tx| {
                    let st = lens(s);
                    if hovered {
                        st.hovered = Some(tip);
                    } else if st.hovered == Some(tip) {
                        st.hovered = None;
                    }
                });
            }
            let panel = (!g.items.is_empty()).then(|| {
                let open = g.is_open(active) && !collapsed;
                let n = g.items.len() as f32;
                let full = n * row_m::SUB_ROW_HEIGHT
                    + (n - 1.0) * row_m::MENU_GAP
                    + 2.0 * row_m::SUB_PADDING_Y;
                let subs = g.items.iter().map(|item| {
                    let href = item.href;
                    let current = href == active;
                    let ink = if current {
                        sb.accent_foreground
                    } else {
                        sb.foreground
                    };
                    let mut sub = row()
                        .key(k(&format!("sub-{href}")))
                        .center_y()
                        .gap(row_m::ROW_GAP)
                        .h(row_m::SUB_ROW_HEIGHT)
                        .px(row_m::SUB_ROW_PADDING_X)
                        .radius(row_m::ROW_RADIUS)
                        .hover(sb.accent.rune())
                        .hover_group()
                        .role("link")
                        .label(item.title)
                        .focusable(true)
                        .cursor(::rune::Cursor::Pointer)
                        .on_press(move |s: &mut S| lens(s).navigate(href))
                        .child(
                            el().grow(1.0).min_w(0.0).child(
                                label(item.title, 14.0, 400.0, 20.0, ink)
                                    .group_color(sb.accent_foreground.rune())
                                    .truncate(),
                            ),
                        )
                        .children(item.badge.map(|b| badge(b, &skin)));
                    if current {
                        sub = sub.fill(sb.accent.rune());
                    }
                    sub
                });
                col()
                    .key(k(&format!("panel-{gi}")))
                    .h(if open { full } else { 0.0 })
                    .motion(if collapsed {
                        slide
                    } else {
                        Spring::new(seconds(SUB_PANEL), 0.0)
                    })
                    .opacity(if open { 1.0 } else { 0.0 })
                    .opacity_motion(if collapsed { tween(COLLAPSE) } else { sub_open })
                    .clip_rounded(0.0)
                    .child(
                        col()
                            .gap(row_m::MENU_GAP)
                            .mx(row_m::SUB_MARGIN_X)
                            .px(row_m::SUB_PADDING_X)
                            .py(row_m::SUB_PADDING_Y)
                            .border_sides([0.0, 0.0, 0.0, 1.0], sb.border.rune())
                            .children(subs),
                    )
            });
            col()
                .key(k(&format!("item-{gi}")))
                .child(button)
                .children(panel)
        };
        let main_rows: Vec<El> = demo::NAV
            .iter()
            .enumerate()
            .map(|(gi, g)| nav_row(gi, g))
            .collect();
        let footer_rows: Vec<El> = demo::FOOTER
            .iter()
            .enumerate()
            .map(|(i, g)| nav_row(demo::NAV.len() + i, g))
            .collect();
        let docs = Row {
            key: k("docs"),
            leading: glyph(demo::DOCS_ICON, 16.0, sb.foreground)
                .group_color(sb.accent_foreground.rune()),
            body: label(demo::DOCS.title, 14.0, 400.0, 20.0, sb.foreground)
                .group_color(sb.accent_foreground.rune()),
            trailing: Vec::new(),
            size: MenuButtonSize::Default,
            active: false,
            collapsed,
        }
        .build(&sb)
        .role("link")
        .label(demo::DOCS.title)
        .href(demo::DOCS.href)
        .on_hover_change(hover_key("Docs"))
        .on_press_tx(|_: &mut S, tx: &mut Tx| {
            tx.open_url(demo::DOCS.href).ok();
        });
        let mut ask_button = row()
            .key(k("ask"))
            .h(32.0)
            .w(if collapsed { 32.0 } else { row_w })
            .motion(quick)
            .center_y()
            .justify_center()
            .gap(6.0)
            .radius(8.0)
            .border(1.0, skin.ask_border.rune())
            .fill(skin.ask.rune())
            .hover(theme.muted.rune())
            .clip_rounded(8.0)
            .role("button")
            .label(demo::ASK_LABEL)
            .focusable(true)
            .cursor(::rune::Cursor::Pointer)
            .child(glyph(demo::ASK_ICON, 16.0, sb.foreground));
        if !collapsed {
            ask_button = ask_button.px(10.0).child(
                label(demo::ASK_LABEL, 14.0, 500.0, 20.0, sb.foreground)
                    .key(k("ask/label"))
                    .transition(Transition::none()),
            );
        }
        let content = col()
            .key(k("content"))
            .grow(1.0)
            .min_h(0.0)
            .scroll(ScrollSpec {
                stick_bottom: false,
            })
            .on_at_bottom(move |s: &mut S, at| lens(s).has_more = !at)
            .child(
                col()
                    .padding(row_m::SECTION_PADDING)
                    .gap(row_m::MENU_GAP)
                    .children(main_rows),
            )
            .child(::rune::spacer())
            .child(
                col()
                    .gap(metrics::FOOTER_GROUP_GAP)
                    .pt(row_m::SECTION_PADDING)
                    .px(row_m::SECTION_PADDING)
                    .children(footer_rows)
                    .child(docs)
                    .child(col().pt(metrics::ASK_TOP).child(ask_button)),
            );
        let scroll_fade = (state.has_more && !collapsed).then(|| {
            el().key(k("scroll-fade")).h(0.0).w_full().child(
                el().absolute(0.0, -metrics::SCROLL_FADE)
                    .size(inner, metrics::SCROLL_FADE)
                    .gradient(sb.background.alpha(0.0).rune(), sb.background.rune()),
            )
        });

        // Footer: the account row.
        let user_open = state.menu == Some(AppSidebarMenu::User);
        let initial: String = demo::USER_NAME
            .trim()
            .chars()
            .next()
            .map(|c| c.to_uppercase().collect())
            .unwrap_or_default();
        let avatar = row()
            .size(metrics::AVATAR, metrics::AVATAR)
            .center_y()
            .justify_center()
            .radius(metrics::AVATAR_RADIUS)
            .fill(sb.accent.rune())
            .border(1.0, skin.avatar_border.rune())
            .child(label(initial, 14.0, 400.0, 20.0, sb.muted_foreground));
        let mut user = Row {
            key: k("user"),
            leading: avatar,
            body: label(demo::USER_NAME, 14.0, 500.0, 20.0, sb.foreground).truncate(),
            trailing: Vec::new(),
            size: MenuButtonSize::Lg,
            active: false,
            collapsed,
        }
        .build(&sb)
        .pl(lg_left(metrics::AVATAR))
        .pr(lg_pad)
        .py(lg_pad)
        .motion(quick)
        .role("button")
        .label(demo::USER_NAME)
        .expanded(user_open)
        .on_hover_change(hover_key("user"))
        .on_press(move |s: &mut S| lens(s).toggle_menu(AppSidebarMenu::User));
        if user_open && !collapsed {
            user = user
                .fill(skin.user_open.rune())
                .box_shadow(Shadow::drop(0.0, 0.0, 1.0, skin.user_open_ring).rune());
        }
        let footer = col()
            .pt(metrics::FOOTER_TOP)
            .px(row_m::SECTION_PADDING)
            .pb(row_m::SECTION_PADDING)
            .child(user);

        // The rail: drag to resize or collapse, double-click to reset, arrows to resize.
        let dragging = state.sidebar.is_dragging();
        let rail = el()
            .key(k("rail"))
            .pin(None, Some(0.0), Some(-row_m::RAIL_HIT / 2.0), Some(0.0))
            .w(row_m::RAIL_HIT)
            .role("separator")
            .label("Resize sidebar")
            .range_value(
                if collapsed {
                    0.0
                } else {
                    f64::from(state.sidebar.width)
                },
                0.0,
                f64::from(crate::components::navigation::sidebar::MAX_WIDTH),
            )
            .a11y_value(state.sidebar.rail_value_text())
            .focusable(true)
            .no_focus_ring()
            .hover_group()
            .cursor(if collapsed {
                ::rune::Cursor::Default
            } else {
                ::rune::Cursor::ResizeColumn
            })
            .on_drag(move |s: &mut S, drag: &mut Drag, tx: &mut Tx| {
                let sidebar = &mut lens(s).sidebar;
                match drag.phase {
                    Phase::Down => sidebar.rail_press(0.0),
                    Phase::Move => {
                        sidebar.rail_move(drag.dx);
                        if sidebar.resizing {
                            tx.instant();
                        }
                    }
                    Phase::Up => sidebar.rail_release(),
                }
            })
            .on_double_press(move |s: &mut S| lens(s).sidebar.rail_reset())
            .on_key(move |s: &mut S, ev, _: &mut Tx| {
                let action = match ev.key {
                    Key::Right => RailKey::Grow,
                    Key::Left => RailKey::Shrink,
                    Key::Home => RailKey::Home,
                    Key::End => RailKey::End,
                    Key::Enter => RailKey::Toggle,
                    _ => return false,
                };
                lens(s).sidebar.rail_key(action)
            })
            .child(
                el().pin(
                    Some((row_m::RAIL_HIT - row_m::RAIL_LINE) / 2.0),
                    Some(row_m::RAIL_INSET_Y),
                    None,
                    Some(row_m::RAIL_INSET_Y),
                )
                .w(row_m::RAIL_LINE)
                .radius(row_m::RAIL_LINE / 2.0)
                .fill(if dragging { sb.ring } else { sb.border }.rune())
                .group_opacity(if dragging { 1.0 } else { 0.0 }, 1.0)
                .hover_motion(tween(QUICK), tween(QUICK)),
            );

        let panel = el()
            .key(k("panel"))
            .w(outer)
            .h_full()
            .motion(slide)
            .padding(variant.gutter())
            .child(
                col()
                    .key(k("inner"))
                    .w_full()
                    .h_full()
                    .fill(sb.background.rune())
                    .clip_rounded(0.0)
                    .child(header)
                    .child(search)
                    .child(content)
                    .children(scroll_fade)
                    .child(footer),
            )
            .child(rail);

        let mut inset = col()
            .key(k("inset"))
            .grow(1.0)
            .min_w(0.0)
            .my(row_m::INSET_MARGIN)
            .mr(row_m::INSET_MARGIN)
            .radius(row_m::INSET_RADIUS)
            .border(1.0, sb.border.rune())
            .fill(sb.inset.rune())
            .clip_rounded(row_m::INSET_RADIUS);
        for s in &sb.inset_shadow {
            inset = inset.box_shadow(s.rune());
        }

        let menu = state
            .menu
            .map(|m| self::menu(m, state, &key, &skin, theme, lens));
        let tooltip = collapsed
            .then_some(state.hovered)
            .flatten()
            .filter(|_| state.menu.is_none())
            .map(|hovered| {
                let (anchor, content) = match hovered {
                    "switcher" => (k("switcher"), project.label),
                    "search" => (k("search"), demo::SEARCH_LABEL),
                    "user" => (k("user"), demo::USER_NAME),
                    "Docs" => (k("docs"), "Docs"),
                    title => (
                        all_groups()
                            .position(|g| g.title == title)
                            .map_or_else(|| k("docs"), |gi| k(&format!("nav-{gi}"))),
                        title,
                    ),
                };
                tooltip(&key, &anchor, content, &skin)
            });

        row()
            .key(key.clone())
            .w_full()
            .h_full()
            .fill(sb.background.rune())
            .child(panel)
            .child(inset.children(self.inset))
            .children(menu)
            .children(tooltip)
    }
}

fn all_groups() -> impl Iterator<Item = &'static NavGroup> {
    demo::NAV.iter().chain(demo::FOOTER.iter())
}

fn popup_motion() -> Transition {
    Transition::scale(motion::MENU_SCALE).springs(SpringPair::new(
        Spring::new(seconds(motion::MENU_IN), 0.0),
        Spring::new(seconds(motion::MENU_OUT), 0.0),
    ))
}

fn tooltip(key: &str, anchor: &str, content: &str, skin: &AppSidebarSkin) -> El {
    let height = 16.0 + 2.0 * metrics::TOOLTIP_PADDING.1 + 2.0;
    let mut tip = row()
        .key(format!("{key}/tooltip"))
        .anchored(anchor, Side::Right, Align::Center, metrics::TOOLTIP_OFFSET)
        .px(metrics::TOOLTIP_PADDING.0)
        .py(metrics::TOOLTIP_PADDING.1)
        .radius(metrics::TOOLTIP_RADIUS)
        .border(1.0, skin.tooltip_border.rune())
        .gradient(skin.tooltip_fill.0.rune(), skin.tooltip_fill.1.rune())
        .role("tooltip")
        .transition(popup_motion().combined(Transition::offset(-motion::TOOLTIP_DRIFT, 0.0)))
        .child(
            el().absolute(
                -metrics::TOOLTIP_ARROW / 2.0 + 1.0,
                height / 2.0 - metrics::TOOLTIP_ARROW / 2.0 - 1.0,
            )
            .size(metrics::TOOLTIP_ARROW, metrics::TOOLTIP_ARROW)
            .radius(2.0)
            .rotate(FRAC_PI_4)
            .fill(skin.tooltip_fill.1.mix(skin.tooltip_fill.0, 0.5).rune()),
        )
        .child(label(content, 12.0, 400.0, 16.0, skin.tooltip_text));
    for s in &skin.tooltip_shadows {
        tip = tip.box_shadow(s.rune());
    }
    tip
}

fn menu<S: 'static>(
    menu: AppSidebarMenu,
    state: &AppSidebarState,
    key: &str,
    skin: &AppSidebarSkin,
    theme: &Theme,
    lens: SidebarLens<S>,
) -> El {
    let collapsed = state.sidebar.collapsed();
    let k = |part: &str| format!("{key}/{part}");
    let highlight = state.highlight;
    let menu_row = |ix: usize,
                    leading: Option<El>,
                    title: &str,
                    trailing: Option<El>,
                    checked: bool,
                    destructive: bool| {
        let ink = if destructive {
            skin.destructive
        } else {
            skin.menu_text
        };
        let hot = if destructive {
            skin.destructive_highlight
        } else {
            skin.menu_highlight
        };
        let mut r = row()
            .key(k(&format!("menu-row-{ix}")))
            .center_y()
            .gap(metrics::MENU_ROW_GAP)
            .h(metrics::MENU_ROW_HEIGHT)
            .px(metrics::MENU_ROW_PADDING_X)
            .radius(metrics::MENU_ROW_RADIUS)
            .continuous_corners()
            .hover(hot.rune())
            .hover_motion(tween(motion::MENU_HIGHLIGHT), tween(motion::MENU_HIGHLIGHT))
            .hover_group()
            .role("menuitem")
            .label(title.to_owned())
            .cursor(::rune::Cursor::Pointer)
            .children(leading)
            .child(
                el().grow(1.0)
                    .min_w(0.0)
                    .child(label(title, 14.0, 400.0, 20.0, ink).truncate()),
            )
            .children(trailing);
        if checked {
            r = r.pr(metrics::MENU_INDICATOR_RESERVE).checked(true).child(
                el().absolute(0.0, 8.0)
                    .pin(None, Some(8.0), Some(metrics::MENU_ROW_PADDING_X), None)
                    .child(glyph(Icon::Check, 16.0, ink)),
            );
        }
        if highlight == Some(ix) {
            r = r.fill(hot.rune());
            if let Some(s) = skin.menu_highlight_shadow {
                r = r.box_shadow(s.rune());
            }
        }
        r
    };
    let entry = |ix: usize, e: MenuEntry| {
        let ink = if e.destructive {
            skin.destructive
        } else {
            skin.menu_icon
        };
        let caps = (!e.keys.is_empty()).then(|| {
            row()
                .gap(metrics::KBD_GAP)
                .children(e.keys.iter().map(|cap| kbd(cap, skin)))
        });
        menu_row(
            ix,
            Some(
                glyph(e.icon, 16.0, ink)
                    .group_color(if e.destructive { ink } else { skin.menu_text }.rune()),
            ),
            e.label,
            caps,
            false,
            e.destructive,
        )
        .on_press(move |s: &mut S| lens(s).close_menu())
    };
    let separator = || {
        el().h(1.0)
            .mx(metrics::MENU_SEPARATOR_MARGIN.0)
            .my(metrics::MENU_SEPARATOR_MARGIN.1)
            .fill(skin.menu_separator.rune())
    };

    let (anchor, side, gap, width, rows, body): (String, Side, f32, f32, usize, Vec<El>) =
        match menu {
            AppSidebarMenu::Switcher => {
                let mut body = vec![
                    col()
                        .h(metrics::MENU_LABEL_HEIGHT)
                        .px(metrics::MENU_ROW_PADDING_X)
                        .py(6.0)
                        .child(label(
                            demo::SWITCHER_LABEL,
                            12.0,
                            500.0,
                            16.0,
                            skin.menu_label,
                        )),
                ];
                for (i, p) in demo::PROJECTS.iter().enumerate() {
                    let value = p.value;
                    body.push(
                        menu_row(
                            i,
                            Some(square(
                                p.initial,
                                metrics::SWITCHER_ICON_SMALL,
                                metrics::SWITCHER_ICON_SMALL_RADIUS,
                                10.0,
                                theme.sidebar_accent,
                                skin.menu_text,
                            )),
                            p.label,
                            None,
                            state.project == value,
                            false,
                        )
                        .pr(metrics::MENU_INDICATOR_RESERVE)
                        .on_press(move |s: &mut S| {
                            let st = lens(s);
                            st.project = value;
                            st.close_menu();
                        }),
                    );
                }
                body.push(separator());
                for (i, e) in demo::SWITCHER_ACTIONS.iter().enumerate() {
                    body.push(entry(demo::PROJECTS.len() + i, *e));
                }
                (
                    k("switcher"),
                    Side::Bottom,
                    metrics::MENU_OFFSET,
                    metrics::SWITCHER_MENU_WIDTH,
                    demo::PROJECTS.len() + demo::SWITCHER_ACTIONS.len(),
                    body,
                )
            }
            AppSidebarMenu::User => {
                let mut body = vec![
                    col()
                        .gap(2.0)
                        .px(metrics::USER_HEADER_PADDING.0)
                        .py(metrics::USER_HEADER_PADDING.1)
                        .child(label(demo::USER_NAME, 14.0, 500.0, 20.0, skin.menu_text).truncate())
                        .child(
                            label(demo::USER_EMAIL, 12.0, 400.0, 16.0, skin.menu_label).truncate(),
                        ),
                    separator(),
                ];
                let mut ix = 0;
                for (g, group) in demo::USER_MENU.iter().enumerate() {
                    if g > 0 {
                        body.push(separator());
                    }
                    for e in group.iter() {
                        body.push(entry(ix, *e));
                        ix += 1;
                    }
                }
                (
                    k("user"),
                    if collapsed { Side::Right } else { Side::Top },
                    metrics::USER_MENU_OFFSET,
                    metrics::USER_MENU_WIDTH,
                    ix,
                    body,
                )
            }
            AppSidebarMenu::Flyout(gi) => {
                let links = all_groups().nth(gi).map(|g| g.flyout()).unwrap_or_default();
                let count = links.len();
                let body = links
                    .into_iter()
                    .enumerate()
                    .map(|(i, link)| {
                        let href = link.href;
                        menu_row(
                            i,
                            None,
                            link.title,
                            link.badge.map(|b| badge(b, skin)),
                            false,
                            false,
                        )
                        .on_press(move |s: &mut S| lens(s).navigate(href))
                    })
                    .collect();
                (
                    k(&format!("nav-{gi}")),
                    Side::Right,
                    metrics::MENU_OFFSET,
                    metrics::FLYOUT_MIN_WIDTH,
                    count,
                    body,
                )
            }
        };

    let mut panel = col()
        .key(k("menu"))
        .anchored(anchor, side, Align::Start, gap)
        .w(width)
        .padding(metrics::MENU_PADDING)
        .radius(metrics::MENU_RADIUS)
        .continuous_corners()
        .border(1.0, skin.menu_border.rune())
        .gradient(skin.menu_fill.0.rune(), skin.menu_fill.1.rune())
        .role("menu")
        .transition(popup_motion())
        .modal(true, move |s: &mut S| lens(s).close_menu())
        .on_key(move |s: &mut S, ev, _: &mut Tx| {
            let st = lens(s);
            match ev.key {
                Key::Down => st.move_highlight(1, rows),
                Key::Up => st.move_highlight(-1, rows),
                _ => return false,
            }
            true
        })
        .children(body);
    for s in &skin.menu_shadows {
        panel = panel.box_shadow(s.rune());
    }
    panel
}
