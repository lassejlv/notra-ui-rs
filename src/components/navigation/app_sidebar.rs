//! The Notra dashboard sidebar block (`app-sidebar`): project switcher, search
//! row, grouped navigation whose rows reveal their pages, the account menu, the
//! scroll-more fade and the icon-rail variant. Its dropdowns and tooltips are a
//! minimal private menu and tooltip matched to the site.

use std::time::Duration;

use crate::Icon;
use crate::motion::{Tween, ease};
use crate::theme::{Color, Shadow, Theme};

/// A page link.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavLink {
    pub title: &'static str,
    pub href: &'static str,
    /// A short tag after the label, such as "Beta".
    pub badge: Option<&'static str>,
}

/// One row per job: it opens `href`, and its `items` show underneath while the
/// row or one of them is the active page. On the icon rail a row with items opens
/// them in a flyout menu instead.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavGroup {
    pub title: &'static str,
    pub href: &'static str,
    pub icon: Icon,
    pub badge: Option<&'static str>,
    pub items: &'static [NavLink],
}

impl NavGroup {
    /// The row lists its own page among its items.
    pub fn lists_own_page(&self) -> bool {
        self.items.iter().any(|item| item.href == self.href)
    }

    /// The row or one of its pages is active.
    pub fn is_active(&self, active: &str) -> bool {
        self.href == active || self.items.iter().any(|item| item.href == active)
    }

    /// The row itself is the active page (and does not list it underneath).
    pub fn own_page_active(&self, active: &str) -> bool {
        self.href == active && !self.lists_own_page()
    }

    /// Its sub-rows are open.
    pub fn is_open(&self, active: &str) -> bool {
        self.is_active(active) && !self.items.is_empty()
    }

    /// The badge on the row: hidden while its pages show.
    pub fn row_badge(&self, active: &str) -> Option<&'static str> {
        if self.is_open(active) {
            None
        } else {
            self.badge
        }
    }

    /// Entries of the rail's flyout: the row's own page first unless listed.
    pub fn flyout(&self) -> Vec<NavLink> {
        let own = NavLink {
            title: self.title,
            href: self.href,
            badge: self.badge,
        };
        (!self.lists_own_page())
            .then_some(own)
            .into_iter()
            .chain(self.items.iter().copied())
            .collect()
    }
}

/// A project in the switcher.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitcherOption {
    pub value: &'static str,
    pub label: &'static str,
    pub description: Option<&'static str>,
    /// The initial shown in the 28 px square.
    pub initial: &'static str,
}

/// A row of a dropdown menu.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuEntry {
    pub icon: Icon,
    pub label: &'static str,
    /// Key caps after the label, one cap per entry.
    pub keys: &'static [&'static str],
    pub destructive: bool,
}

impl MenuEntry {
    pub const fn new(icon: Icon, label: &'static str) -> Self {
        Self {
            icon,
            label,
            keys: &[],
            destructive: false,
        }
    }

    pub const fn keys(mut self, keys: &'static [&'static str]) -> Self {
        self.keys = keys;
        self
    }

    pub const fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }
}

/// Which popup is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppSidebarMenu {
    Switcher,
    User,
    /// The rail flyout of the group at this index.
    Flyout(usize),
}

/// The block's state, owned by the app.
#[derive(Clone, Debug, PartialEq)]
pub struct AppSidebarState {
    pub sidebar: super::sidebar::SidebarState,
    pub project: &'static str,
    pub active: &'static str,
    pub menu: Option<AppSidebarMenu>,
    /// The highlighted row of the open menu.
    pub highlight: Option<usize>,
    /// Content overflows below the visible rows.
    pub has_more: bool,
}

impl AppSidebarState {
    pub fn new(open: bool, active: &'static str) -> Self {
        Self {
            sidebar: super::sidebar::SidebarState::new(open),
            project: demo::PROJECTS[0].value,
            active,
            menu: None,
            highlight: None,
            has_more: false,
        }
    }

    /// Opens `menu`, or closes it when it is already open.
    pub fn toggle_menu(&mut self, menu: AppSidebarMenu) {
        self.menu = (self.menu != Some(menu)).then_some(menu);
        self.highlight = None;
    }

    pub fn close_menu(&mut self) {
        self.menu = None;
        self.highlight = None;
    }

    /// Moves the menu highlight by `delta` among `count` rows, wrapping.
    pub fn move_highlight(&mut self, delta: i32, count: usize) {
        if count == 0 {
            return;
        }
        let count = count as i32;
        let next = match self.highlight {
            Some(at) => (at as i32 + delta).rem_euclid(count),
            None if delta < 0 => count - 1,
            None => 0,
        };
        self.highlight = Some(next as usize);
    }

    pub fn navigate(&mut self, href: &'static str) {
        self.active = href;
        self.close_menu();
    }

    pub fn toggle_sidebar(&mut self) {
        self.sidebar.toggle();
        self.close_menu();
    }
}

/// Motion of the block.
pub mod motion {
    use super::*;

    /// `useSidebarCollapseSettled`: sub-rows give way to flyouts once the
    /// collapse has run (`SIDEBAR_COLLAPSE_MS`).
    pub const COLLAPSE_SETTLE: Duration = Duration::from_millis(300);
    /// Menus open with a fade from 97 % scale (`duration-150`, Notra's ease-out).
    pub const MENU_IN: Tween = Tween::new(Duration::from_millis(150), ease::OUT);
    /// And close faster (`data-ending-style:duration-100`).
    pub const MENU_OUT: Tween = Tween::new(Duration::from_millis(100), ease::OUT);
    pub const MENU_SCALE: f32 = 0.97;
    /// Menu rows highlight (`transition-colors duration-75`).
    pub const MENU_HIGHLIGHT: Tween = Tween::new(Duration::from_millis(75), ease::IN_OUT);
    /// Tooltips: the same fade and scale, drifting 2 px toward their side.
    pub const TOOLTIP_IN: Tween = MENU_IN;
    pub const TOOLTIP_OUT: Tween = MENU_OUT;
    pub const TOOLTIP_DRIFT: f32 = 2.0;
}

/// Box metrics measured on the live demo.
pub mod metrics {
    /// The switcher's square (`size-7 rounded-lg text-xs font-semibold`) and the
    /// menu's small one (`size-5 rounded-md text-[0.625rem]`).
    pub const SWITCHER_ICON: f32 = 28.0;
    pub const SWITCHER_ICON_RADIUS: f32 = 8.0;
    pub const SWITCHER_ICON_SMALL: f32 = 20.0;
    pub const SWITCHER_ICON_SMALL_RADIUS: f32 = 6.0;
    /// The avatar (`size-8 rounded-lg`).
    pub const AVATAR: f32 = 32.0;
    pub const AVATAR_RADIUS: f32 = 8.0;
    /// The header row reserves `pr-8` for the trigger, which sits at
    /// `left: calc(100% - 1.75rem)`, `top-2.5`; on the rail it moves to `left-0.5 top-13`
    /// and the row reserves `pb-8` instead.
    pub const HEADER_RESERVE: f32 = 32.0;
    pub const TRIGGER_TOP: (f32, f32) = (10.0, 52.0);
    pub const TRIGGER_RAIL_LEFT: f32 = 2.0;
    /// The demo's header padding (`pt-px pb-0.5`).
    pub const HEADER_PADDING: (f32, f32) = (1.0, 2.0);
    /// The search block (`px-2 pt-2.5 pb-3`) and row (`h-9 rounded-lg border`,
    /// `h-8` on the rail).
    pub const SEARCH_PADDING: (f32, f32, f32) = (8.0, 10.0, 12.0);
    pub const SEARCH_HEIGHT: (f32, f32) = (36.0, 32.0);
    pub const SEARCH_RADIUS: f32 = 8.0;
    /// The ⌘K cap (`rounded-[5px] border px-1.5 h-5 text-xs font-medium`).
    pub const SEARCH_KBD_RADIUS: f32 = 5.0;
    pub const SEARCH_KBD_PADDING: f32 = 6.0;
    /// Kbd caps in menus (`h-5 min-w-5 px-1 rounded-sm`).
    pub const KBD_HEIGHT: f32 = 20.0;
    pub const KBD_PADDING: f32 = 4.0;
    pub const KBD_RADIUS: f32 = 4.0;
    pub const KBD_GAP: f32 = 4.0;
    pub const KBD_FONT: f32 = 12.0;
    /// Nav badges (`h-[1.125rem] px-1.5 rounded-full text-[0.625rem] font-medium`).
    pub const BADGE_HEIGHT: f32 = 18.0;
    pub const BADGE_PADDING: f32 = 6.0;
    pub const BADGE_FONT: f32 = 10.0;
    /// Footer navigation (`mt-auto gap-1 pb-0`) and the ask button above the footer (`pt-2`).
    pub const FOOTER_GROUP_GAP: f32 = 4.0;
    pub const ASK_TOP: f32 = 8.0;
    /// The account footer (`pt-1.75`).
    pub const FOOTER_TOP: f32 = 7.0;
    /// The fade over the last rows while more content is below (`h-8`).
    pub const SCROLL_FADE: f32 = 32.0;
    /// Menus.
    pub const MENU_RADIUS: f32 = 14.0;
    pub const MENU_PADDING: f32 = 4.0;
    pub const MENU_ROW_HEIGHT: f32 = 32.0;
    pub const MENU_ROW_PADDING_X: f32 = 10.0;
    pub const MENU_ROW_RADIUS: f32 = 10.0;
    pub const MENU_ROW_GAP: f32 = 8.0;
    /// Radio rows keep `pr-8` for the check at `right-2.5`.
    pub const MENU_INDICATOR_RESERVE: f32 = 32.0;
    pub const MENU_LABEL_HEIGHT: f32 = 28.0;
    pub const MENU_SEPARATOR_MARGIN: (f32, f32) = (8.0, 4.0);
    /// The switcher menu matches its trigger (`min-w-56`); the account menu is `w-64`;
    /// the rail flyout is `min-w-48`.
    pub const SWITCHER_MENU_WIDTH: f32 = 224.0;
    pub const USER_MENU_WIDTH: f32 = 256.0;
    pub const FLYOUT_MIN_WIDTH: f32 = 192.0;
    /// Gaps between trigger and menu.
    pub const MENU_OFFSET: f32 = 4.0;
    pub const USER_MENU_OFFSET: f32 = 6.0;
    /// The account header in its menu (`px-2 py-1.5 gap-0.5`).
    pub const USER_HEADER_PADDING: (f32, f32) = (8.0, 6.0);
    /// Tooltips beside the rail (`sideOffset={8}`, `px-2.5 py-1 rounded-[0.625rem] text-xs`).
    pub const TOOLTIP_OFFSET: f32 = 8.0;
    pub const TOOLTIP_PADDING: (f32, f32) = (10.0, 4.0);
    pub const TOOLTIP_RADIUS: f32 = 10.0;
    pub const TOOLTIP_ARROW: f32 = 10.0;
}

/// Colours of the block's popups and small parts for one theme.
#[derive(Clone, Debug, PartialEq)]
pub struct AppSidebarSkin {
    pub menu_border: Color,
    /// `bg-linear-to-b`, top to bottom.
    pub menu_fill: (Color, Color),
    pub menu_shadows: Vec<Shadow>,
    pub menu_text: Color,
    pub menu_icon: Color,
    pub menu_label: Color,
    pub menu_highlight: Color,
    /// Dark rows gain a hairline highlight.
    pub menu_highlight_shadow: Option<Shadow>,
    pub menu_separator: Color,
    pub destructive: Color,
    pub destructive_highlight: Color,
    pub kbd: Color,
    pub kbd_text: Color,
    pub badge: Color,
    pub badge_text: Color,
    /// The avatar's hairline (`after:border`).
    pub avatar_border: Color,
    /// The account row while its menu is open: `bg-sidebar-accent/90` and a ring.
    pub user_open: Color,
    pub user_open_ring: Color,
    /// The outline `Ask agent` button.
    pub ask: Color,
    pub ask_border: Color,
    pub tooltip_fill: (Color, Color),
    pub tooltip_border: Color,
    pub tooltip_text: Color,
    pub tooltip_shadows: Vec<Shadow>,
    /// The page cards in the demo inset (`bg-muted/60`).
    pub placeholder: Color,
}

impl AppSidebarSkin {
    pub fn new(theme: &Theme) -> Self {
        let dark = theme.is_dark();
        Self {
            menu_border: if dark { theme.input } else { theme.border },
            menu_fill: if dark {
                (theme.muted, theme.shell)
            } else {
                (
                    theme.background,
                    theme.background.mix(theme.foreground, 0.025),
                )
            },
            menu_shadows: if dark {
                vec![
                    Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffff_ff0f)),
                    Shadow::drop(2.0, 4.0, 0.0, Color::rgba(0x0000_004d)),
                    Shadow::drop(16.0, 32.0, -8.0, Color::rgba(0x0000_0099)),
                ]
            } else {
                vec![
                    Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffff_ffcc)),
                    Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x0000_000d)),
                    Shadow::drop(12.0, 32.0, -8.0, Color::rgba(0x0000_0029)),
                ]
            },
            menu_text: theme.popover_foreground,
            menu_icon: theme.muted_foreground,
            menu_label: theme.muted_foreground,
            menu_highlight: theme.foreground.fade(if dark { 0.07 } else { 0.05 }),
            menu_highlight_shadow: dark
                .then(|| Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffff_ff0a))),
            menu_separator: theme.border,
            destructive: theme.destructive,
            destructive_highlight: theme.destructive.fade(0.1),
            kbd: theme.muted,
            kbd_text: theme.muted_foreground,
            badge: theme.secondary,
            badge_text: theme.muted_foreground,
            avatar_border: theme.border,
            user_open: theme.sidebar_accent.fade(0.9),
            user_open_ring: theme.sidebar_border.fade(0.7),
            ask: if dark {
                theme.input.fade(0.3)
            } else {
                theme.background
            },
            ask_border: if dark { theme.input } else { theme.border },
            tooltip_fill: (
                theme.foreground.mix(theme.background, 0.1),
                theme.foreground,
            ),
            tooltip_border: theme.foreground.fade(0.6),
            tooltip_text: theme.background,
            tooltip_shadows: vec![
                Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffff_ff38)),
                Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x0000_0029)),
                Shadow::drop(6.0, 16.0, -4.0, Color::rgba(0x0000_003d)),
            ],
            placeholder: theme.muted.fade(0.6),
        }
    }
}

/// The registry's demo data.
pub mod demo {
    use super::*;

    pub const PROJECTS: [SwitcherOption; 3] = [
        SwitcherOption {
            value: "notra",
            label: "Notra",
            description: Some("usenotra.com"),
            initial: "N",
        },
        SwitcherOption {
            value: "acme",
            label: "Acme",
            description: Some("acme.dev"),
            initial: "A",
        },
        SwitcherOption {
            value: "blume",
            label: "Blume",
            description: Some("blume.sh"),
            initial: "B",
        },
    ];

    const fn link(title: &'static str, href: &'static str) -> NavLink {
        NavLink {
            title,
            href,
            badge: None,
        }
    }

    const fn group(
        title: &'static str,
        href: &'static str,
        icon: Icon,
        items: &'static [NavLink],
    ) -> NavGroup {
        NavGroup {
            title,
            href,
            icon,
            badge: None,
            items,
        }
    }

    pub const NAV: [NavGroup; 8] = [
        group(
            "Overview",
            "#home",
            Icon::HugeHome01,
            &[
                link("Home", "#home"),
                link("Traffic", "#traffic"),
                link("Analytics", "#analytics"),
            ],
        ),
        group(
            "Prompts",
            "#prompts",
            Icon::HugePrompt,
            &[NavLink {
                title: "Personas",
                href: "#personas",
                badge: Some("Beta"),
            }],
        ),
        group(
            "Competitors",
            "#competitors",
            Icon::HugeChartAnalysis,
            &[link("Shelf Space", "#shelf-space")],
        ),
        group(
            "Improve",
            "#content-gaps",
            Icon::HugeSearchList01,
            &[link("Agent Readiness", "#agent-readiness")],
        ),
        group(
            "Automations",
            "#automations",
            Icon::HugeWorkflowSquare03,
            &[],
        ),
        group(
            "Content",
            "#content",
            Icon::HugeNote,
            &[link("Chat", "#chat")],
        ),
        group("Sites", "#sites", Icon::HugeWebDesign01, &[]),
        group("Feedback", "#feedback", Icon::HugeFeedback, &[]),
    ];

    pub const FOOTER: [NavGroup; 1] = [group(
        "Workspace",
        "#brand-identity",
        Icon::HugeBuilding03,
        &[
            link("Brand Identity", "#brand-identity"),
            link("Integrations", "#integrations"),
            link("Skills", "#skills"),
            link("API Keys", "#api-keys"),
        ],
    )];

    /// The external docs row under the footer navigation.
    pub const DOCS: NavLink = link("Docs", "https://docs.usenotra.com");
    pub const DOCS_ICON: Icon = Icon::HugeBookOpen01;

    pub const SEARCH_LABEL: &str = "Search";
    pub const SEARCH_SHORTCUT: &str = "⌘K";
    pub const SWITCHER_LABEL: &str = "Projects";
    pub const SWITCHER_ACTIONS: [MenuEntry; 2] = [
        MenuEntry::new(Icon::HugePlusSign, "New project"),
        MenuEntry::new(Icon::HugeSettings01, "Project settings"),
    ];

    pub const USER_NAME: &str = "Ada Lovelace";
    pub const USER_EMAIL: &str = "ada@usenotra.com";
    /// The account menu, in groups split by separators.
    pub const USER_MENU: [&[MenuEntry]; 2] = [
        &[
            MenuEntry::new(Icon::HugeUser, "My profile").keys(&["⌘", ","]),
            MenuEntry::new(Icon::HugeSettings01, "Appearance"),
            MenuEntry::new(Icon::HugeMessage01, "Feedback").keys(&["F"]),
            MenuEntry::new(Icon::HugeCalendar03, "Schedule demo").keys(&["S"]),
        ],
        &[MenuEntry::new(Icon::HugeLogout01, "Log out").destructive()],
    ];

    pub const ASK_LABEL: &str = "Ask agent";
    pub const ASK_ICON: Icon = Icon::NotraAgent;

    /// The demo's default page, and the collapsed example's.
    pub const DEFAULT_HREF: &str = "#traffic";
    pub const COLLAPSED_HREF: &str = "#chat";

    /// `DEMO_PAGE_TITLES`: the inset header shows the active page's title.
    pub fn page_title(href: &str) -> &'static str {
        NAV.iter()
            .chain(FOOTER.iter())
            .flat_map(|g| {
                std::iter::once((g.href, g.title)).chain(g.items.iter().map(|i| (i.href, i.title)))
            })
            .find(|(h, _)| *h == href)
            .map_or("Overview", |(_, title)| title)
    }

    /// The preview frame's height (`h-184`).
    pub const FRAME_HEIGHT: f32 = 736.0;
    /// The inset page: `h-12 px-4 border-b` header, `p-4 gap-4` grid of three
    /// `aspect-video` cards over an `h-64` card, all `rounded-xl`.
    pub const HEADER_HEIGHT: f32 = 48.0;
    pub const PAGE_PADDING: f32 = 16.0;
    pub const CARD_TALL: f32 = 256.0;
    pub const CARD_RADIUS: f32 = 12.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_page_opens_its_group() {
        let overview = demo::NAV[0];
        assert!(overview.is_open("#traffic"));
        assert!(
            !overview.own_page_active("#home"),
            "Home is listed underneath"
        );
        assert!(!demo::NAV[1].is_open("#traffic"));
        assert!(demo::NAV[4].own_page_active("#automations"));
    }

    #[test]
    fn flyout_leads_with_the_row_unless_listed() {
        assert_eq!(demo::NAV[0].flyout().len(), 3);
        let content = demo::NAV[5].flyout();
        assert_eq!(content[0].href, "#content");
        assert_eq!(content.len(), 2);
    }

    #[test]
    fn badges_hide_while_pages_show() {
        let mut prompts = demo::NAV[1];
        prompts.badge = Some("New");
        assert_eq!(prompts.row_badge("#personas"), None);
        assert_eq!(prompts.row_badge("#home"), Some("New"));
    }

    #[test]
    fn titles_resolve_items_and_groups() {
        assert_eq!(demo::page_title("#traffic"), "Traffic");
        assert_eq!(demo::page_title("#chat"), "Chat");
        assert_eq!(demo::page_title("#nowhere"), "Overview");
    }

    #[test]
    fn menus_toggle_and_highlight_wraps() {
        let mut s = AppSidebarState::new(true, demo::DEFAULT_HREF);
        s.toggle_menu(AppSidebarMenu::User);
        assert_eq!(s.menu, Some(AppSidebarMenu::User));
        s.move_highlight(-1, 5);
        assert_eq!(s.highlight, Some(4));
        s.move_highlight(1, 5);
        assert_eq!(s.highlight, Some(0));
        s.toggle_menu(AppSidebarMenu::User);
        assert_eq!(s.menu, None);
    }
}
