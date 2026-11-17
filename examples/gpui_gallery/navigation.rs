use gpui::{
    App, AppContext as _, Context, Entity, FontWeight, IntoElement, ParentElement, Render, Styled,
    Window, div, px,
};
use notra_ui::components::navigation::app_sidebar::demo as sidebar_demo;
use notra_ui::components::navigation::icon_tabs::demo as tabs_demo;
use notra_ui::components::navigation::step_slider::{demo as slider_demo, nearest_step_index};
use notra_ui::gpui::{ActiveTheme, AppSidebar, IconTabs, StepSlider, app_sidebar_demo_page};

use crate::{Page, preview, section};

pub fn icon_tabs_page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "icon-tabs",
        title: "Icon Tabs",
        view: cx.new(|_| IconTabsPage { tab: 0 }).into(),
    }
}

struct IconTabsPage {
    tab: usize,
}

impl Render for IconTabsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tabs = tabs_demo::TABS
            .iter()
            .fold(IconTabs::new("webhook-views", self.tab), |tabs, tab| {
                tabs.tab(tab.icon, tab.label)
            })
            .width(px(tabs_demo::LIST_WIDTH))
            .on_change(cx.listener(|this, ix: &usize, _, cx| {
                this.tab = *ix;
                cx.notify();
            }));
        div().child(section("Preview", cx).child(preview(cx).child(tabs)))
    }
}

pub fn step_slider_page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "step-slider",
        title: "Step Slider",
        view: cx
            .new(|_| StepSliderPage {
                prompts: slider_demo::DEFAULT,
            })
            .into(),
    }
}

struct StepSliderPage {
    prompts: f64,
}

impl Render for StepSliderPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.notra();
        let value = slider_demo::STEPS[nearest_step_index(&slider_demo::STEPS, self.prompts)];
        let slider = StepSlider::new("prompts", &slider_demo::STEPS, self.prompts)
            .format(slider_demo::format)
            .major_steps(&slider_demo::MAJOR)
            .on_change(cx.listener(|this, v: &f64, _, cx| {
                this.prompts = *v;
                cx.notify();
            }));
        let example = div()
            .flex()
            .flex_col()
            .gap(px(slider_demo::GAP))
            .w_full()
            .max_w(px(slider_demo::MAX_WIDTH))
            .p(px(slider_demo::PADDING))
            .text_color(theme.foreground)
            .child(
                div()
                    .flex()
                    .items_end()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(slider_demo::TITLE_SIZE.0))
                            .line_height(px(slider_demo::TITLE_SIZE.1))
                            .font_weight(FontWeight(500.0))
                            .pb(px(2.0))
                            .child(slider_demo::LABEL),
                    )
                    .child(
                        div()
                            .text_size(px(slider_demo::VALUE_SIZE.0))
                            .line_height(px(slider_demo::VALUE_SIZE.1))
                            .child(format!("{value}")),
                    ),
            )
            .child(slider);
        div().child(section("Preview", cx).child(preview(cx).child(example)))
    }
}

pub fn sidebar_page(_window: &mut Window, cx: &mut App) -> Page {
    let frame = |open: bool, active: &'static str, cx: &mut App| -> Entity<AppSidebar> {
        cx.new(|cx| {
            AppSidebar::new(open, active, cx).inset(|state, _, cx| app_sidebar_demo_page(state, cx))
        })
    };
    let demo = frame(true, sidebar_demo::DEFAULT_HREF, cx);
    let collapsed = frame(false, sidebar_demo::COLLAPSED_HREF, cx);
    Page {
        slug: "app-sidebar",
        title: "App Sidebar",
        view: cx.new(|_| SidebarPage { demo, collapsed }).into(),
    }
}

struct SidebarPage {
    demo: Entity<AppSidebar>,
    collapsed: Entity<AppSidebar>,
}

impl Render for SidebarPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.notra();
        let frame = |view: Entity<AppSidebar>| {
            div()
                .h(px(sidebar_demo::FRAME_HEIGHT))
                .w_full()
                .rounded(px(12.0))
                .overflow_hidden()
                .border_1()
                .border_color(theme.border)
                .child(view)
        };
        div()
            .flex()
            .flex_col()
            .child(section("Demo", cx).child(frame(self.demo.clone())))
            .child(section("Collapsed", cx).child(frame(self.collapsed.clone())))
    }
}
