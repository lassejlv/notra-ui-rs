use notra_ui::Theme;
use notra_ui::components::navigation::app_sidebar::{
    AppSidebarMenu, AppSidebarState, demo as sidebar_demo,
};
use notra_ui::components::navigation::icon_tabs::demo as tabs_demo;
use notra_ui::components::navigation::step_slider::{demo as slider_demo, nearest_step_index};
use notra_ui::rune::{
    AppSidebar, IconTabs, IconTabsState, StepSlider, StepSliderState, ToRune, sans,
};
use rune::{El, col, el, row, text};

use crate::{State, preview, section};

pub struct Demo {
    pub sidebar: AppSidebarState,
    pub collapsed: AppSidebarState,
    pub tabs: IconTabsState,
    pub prompts: StepSliderState,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            sidebar: AppSidebarState::new(true, sidebar_demo::DEFAULT_HREF),
            collapsed: AppSidebarState::new(false, sidebar_demo::COLLAPSED_HREF),
            tabs: IconTabsState::new(0),
            prompts: StepSliderState::new(slider_demo::DEFAULT),
        }
    }
}

impl Demo {
    /// The account menu open, as after clicking the footer row.
    pub fn user_menu() -> Self {
        let mut demo = Self::default();
        demo.sidebar.menu = Some(AppSidebarMenu::User);
        demo
    }

    /// The second tab selected.
    pub fn second_tab() -> Self {
        Self {
            tabs: IconTabsState::new(1),
            ..Self::default()
        }
    }
}

fn page(state: &AppSidebarState, t: &Theme) -> El {
    let card = |h: Option<f32>| {
        let c = el()
            .w_full()
            .radius(sidebar_demo::CARD_RADIUS)
            .fill(t.muted.fade(0.6).rune());
        col().grow(1.0).min_w(0.0).child(match h {
            Some(h) => c.h(h),
            None => c.aspect_ratio(16.0 / 9.0),
        })
    };
    col()
        .w_full()
        .child(
            row()
                .h(sidebar_demo::HEADER_HEIGHT)
                .px(16.0)
                .center_y()
                .border_sides([0.0, 0.0, 1.0, 0.0], t.border.rune())
                .child(
                    text(sidebar_demo::page_title(state.active))
                        .font(sans(14.0, 500.0, 20.0))
                        .color(t.foreground.rune()),
                ),
        )
        .child(
            col()
                .padding(sidebar_demo::PAGE_PADDING)
                .gap(sidebar_demo::PAGE_PADDING)
                .child(
                    row()
                        .w_full()
                        .gap(sidebar_demo::PAGE_PADDING)
                        .child(card(None))
                        .child(card(None))
                        .child(card(None)),
                )
                .child(card(Some(sidebar_demo::CARD_TALL))),
        )
}

pub fn sidebar_view(s: &State, t: &Theme) -> El {
    let frame =
        |key: &str, state: &AppSidebarState, lens: fn(&mut State) -> &mut AppSidebarState| {
            el().h(sidebar_demo::FRAME_HEIGHT)
                .w_full()
                .radius(12.0)
                .border(1.0, t.border.rune())
                .border_layout()
                .clip_rounded(12.0)
                .child(
                    AppSidebar::new(key, state)
                        .inset(page(state, t))
                        .build(t, lens),
                )
        };
    col()
        .child(section(
            "Demo",
            t,
            frame("app-sidebar", &s.navigation.sidebar, |s| {
                &mut s.navigation.sidebar
            }),
        ))
        .child(section(
            "Collapsed",
            t,
            frame("app-sidebar-collapsed", &s.navigation.collapsed, |s| {
                &mut s.navigation.collapsed
            }),
        ))
}

pub fn icon_tabs_view(s: &State, t: &Theme) -> El {
    let tabs = tabs_demo::TABS
        .iter()
        .fold(
            IconTabs::new("webhook-views", &s.navigation.tabs),
            |tabs, tab| tabs.tab(tab.icon, tab.label),
        )
        .width(tabs_demo::LIST_WIDTH)
        .label(tabs_demo::LABEL)
        .build(t, |s: &mut State| &mut s.navigation.tabs);
    col().child(section("Preview", t, preview(t).child(tabs)))
}

pub fn step_slider_view(s: &State, t: &Theme) -> El {
    let state = &s.navigation.prompts;
    let value = slider_demo::STEPS[nearest_step_index(&slider_demo::STEPS, state.value)];
    let example = col()
        .w_full()
        .max_w(slider_demo::MAX_WIDTH)
        .padding(slider_demo::PADDING)
        .gap(slider_demo::GAP)
        .child(
            row()
                .justify_between()
                .items_end()
                .child(
                    text(slider_demo::LABEL)
                        .font(sans(
                            slider_demo::TITLE_SIZE.0,
                            500.0,
                            slider_demo::TITLE_SIZE.1,
                        ))
                        .color(t.foreground.rune())
                        .pb(2.0),
                )
                .child(
                    text(format!("{value}"))
                        .font(
                            sans(slider_demo::VALUE_SIZE.0, 400.0, slider_demo::VALUE_SIZE.1)
                                .features("'tnum' 1"),
                        )
                        .color(t.foreground.rune()),
                ),
        )
        .child(
            StepSlider::new("prompts", &slider_demo::STEPS, state)
                .format(slider_demo::format)
                .major_steps(&slider_demo::MAJOR)
                .label(slider_demo::LABEL)
                .value_text(slider_demo::value_text)
                .build(t, |s: &mut State| &mut s.navigation.prompts),
        );
    col().child(section("Preview", t, preview(t).child(example)))
}
