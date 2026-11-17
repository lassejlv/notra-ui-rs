//! The Rune gallery: every Notra component in one window.
//!
//! ```sh
//! cargo run --example rune_gallery --features rune
//! cargo run --example rune_gallery --features rune -- --list-scenes
//! cargo run --example rune_gallery --features rune -- --snap out/button.png --scene button --scale 2
//! cargo run --example rune_gallery --features rune -- --snap out/button-dark.png --scene button-dark --scale 2
//! ```

mod button;
mod overlays_demo;
mod overlays_menu;

use notra_ui::rune::{FAMILIES, FONTS, ToRune, sans};
use notra_ui::{Appearance, Theme};
use rune::{AppSpec, Cx, El, ScrollSpec, col, row, text};

/// Gallery pages, in sidebar order. Add a page: a module with `view` and a row here.
#[allow(clippy::type_complexity)]
pub const PAGES: &[(&str, &str, fn(&State, &Theme) -> El)] = &[
    ("button", "Button", button::view),
    ("dropdown-menu", "Dropdown Menu", overlays_menu::menu_view),
    ("select", "Select", overlays_menu::select_view),
];

#[derive(Default)]
pub struct State {
    pub page: usize,
    pub appearance: Option<Appearance>,
    pub button: button::Demo,
    pub overlays_menu: overlays_menu::Demo,
}

fn view(s: &State, cx: &mut Cx<State>) -> El {
    let appearance = s.appearance.unwrap_or(if cx.system_dark() {
        Appearance::Dark
    } else {
        Appearance::Light
    });
    let theme = Theme::new(appearance);
    let t = &theme;
    let nav = PAGES
        .iter()
        .enumerate()
        .fold(col().gap(2.0), |nav, (i, (slug, title, _))| {
            let selected = i == s.page;
            nav.child(
                row()
                    .key(format!("nav-{slug}"))
                    .px(10.0)
                    .py(6.0)
                    .radius(8.0)
                    .fill(if selected {
                        t.muted.rune()
                    } else {
                        rune::Color::TRANSPARENT
                    })
                    .hover(t.muted.rune())
                    .on_press(move |s: &mut State| s.page = i)
                    .child(
                        text(*title)
                            .font(sans(13.0, 400.0, 18.0))
                            .color(if selected {
                                t.foreground.rune()
                            } else {
                                t.muted_foreground.rune()
                            }),
                    ),
            )
        });
    let toggle = row()
        .key("appearance")
        .mt(12.0)
        .px(10.0)
        .py(6.0)
        .radius(8.0)
        .hover(t.muted.rune())
        .on_press(move |s: &mut State| {
            s.appearance = Some(if appearance == Appearance::Dark {
                Appearance::Light
            } else {
                Appearance::Dark
            })
        })
        .child(
            text(if appearance == Appearance::Dark {
                "Light mode"
            } else {
                "Dark mode"
            })
            .font(sans(13.0, 400.0, 18.0))
            .color(t.muted_foreground.rune()),
        );
    let (_, title, page) = PAGES[s.page];
    row()
        .key("window")
        .w_full()
        .h_full()
        .fill(t.canvas.rune())
        .child(
            col()
                .key("nav")
                .w(220.0)
                .h_full()
                .padding(12.0)
                .fill(t.sidebar.rune())
                .border_sides([0.0, 1.0, 0.0, 0.0], t.border.rune())
                .child(
                    text("Notra UI")
                        .font(sans(15.0, 600.0, 22.0))
                        .color(t.foreground.rune())
                        .pb(10.0)
                        .px(10.0),
                )
                .child(nav)
                .child(toggle),
        )
        .child(
            col().key("page").grow(1.0).h_full().min_h(0.0).child(
                col()
                    .padding(40.0)
                    .gap(0.0)
                    .child(
                        text(title)
                            .font(sans(28.0, 500.0, 34.0))
                            .color(t.foreground.rune())
                            .pb(24.0),
                    )
                    .child(page(s, t))
                    .scroll(ScrollSpec {
                        stick_bottom: false,
                    })
                    .shrink()
                    .min_h(0.0),
            ),
        )
}

/// A titled preview area, like the site's Preview tab.
pub fn section(title: &str, t: &Theme, preview: El) -> El {
    col()
        .gap(10.0)
        .mb(28.0)
        .child(
            text(title)
                .font(sans(13.0, 500.0, 18.0))
                .color(t.muted_foreground.rune()),
        )
        .child(preview)
}

/// The bordered preview surface components sit on.
pub fn preview(t: &Theme) -> El {
    row()
        .flex_wrap()
        .center_y()
        .justify_center()
        .gap(12.0)
        .min_h(140.0)
        .padding(32.0)
        .radius(12.0)
        .border(1.0, t.border.rune())
        .fill(t.canvas.rune())
}

fn page_scene(slug: &str, dark: bool) -> impl Fn(&mut State) {
    let slug = slug.to_owned();
    move |s: &mut State| {
        s.page = PAGES.iter().position(|(p, _, _)| *p == slug).unwrap_or(0);
        s.appearance = Some(if dark {
            Appearance::Dark
        } else {
            Appearance::Light
        });
    }
}

fn spec() -> AppSpec<State> {
    AppSpec::<State> {
        min_size: Some((900.0, 600.0)),
        scenes: &[
            ("button", |s| page_scene("button", false)(s)),
            ("button-dark", |s| page_scene("button", true)(s)),
            ("button-loading", |s| {
                page_scene("button", false)(s);
                s.button = button::Demo::busy();
            }),
            ("dropdown-menu", |s| page_scene("dropdown-menu", false)(s)),
            ("dropdown-menu-dark", |s| {
                page_scene("dropdown-menu", true)(s)
            }),
            ("dropdown-menu-open", |s| {
                page_scene("dropdown-menu", false)(s);
                s.overlays_menu.menu.open = true;
            }),
            ("dropdown-menu-open-dark", |s| {
                page_scene("dropdown-menu", true)(s);
                s.overlays_menu.menu.open = true;
            }),
            ("dropdown-menu-submenu", |s| {
                page_scene("dropdown-menu", false)(s);
                s.overlays_menu.menu.open = true;
                s.overlays_menu.menu.submenus = vec![3];
            }),
            ("select", |s| page_scene("select", false)(s)),
            ("select-dark", |s| page_scene("select", true)(s)),
            ("select-open", |s| {
                page_scene("select", false)(s);
                s.overlays_menu.frequency_select.open = true;
            }),
            ("select-open-dark", |s| {
                page_scene("select", true)(s);
                s.overlays_menu.frequency_select.open = true;
            }),
            ("select-compact-open", |s| {
                page_scene("select", false)(s);
                s.overlays_menu.compact_select.open = true;
            }),
        ],
        fonts: FONTS,
        families: FAMILIES,
        ..AppSpec::new("Notra UI", (1200.0, 820.0), State::default, view)
    }
}

fn main() {
    rune::prewarm_gpu();
    rune::App::new(spec()).run();
}
