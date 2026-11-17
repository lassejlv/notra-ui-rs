//! Notra's dropdown menu (`notra-dropdown-menu`): the Depth popup surface, its rows, and
//! Base UI's menu keyboard and pointer behaviour as a state machine. The select reuses the
//! surface and row skins.

use std::time::Duration;

use super::{Align, Placement, PopupMotion, Side};
use crate::icons::Icon;
use crate::motion::{Tween, ease};
use crate::theme::{Color, Shadow, Theme};

/// Measured box metrics, in logical pixels.
pub mod metrics {
    /// `rounded-[0.875rem]` with `corner-shape: squircle`.
    pub const SURFACE_RADIUS: f32 = 14.0;
    /// `p-1` inside the 1 px border.
    pub const SURFACE_PADDING: f32 = 4.0;
    /// `min-w-32`.
    pub const MIN_WIDTH: f32 = 128.0;
    /// `min-w-40` of a submenu.
    pub const SUB_MIN_WIDTH: f32 = 160.0;
    /// `max-h-[min(24rem, var(--available-height))]`.
    pub const MAX_HEIGHT: f32 = 384.0;
    /// `min-h-8`.
    pub const ITEM_HEIGHT: f32 = 32.0;
    pub const ITEM_PADDING_X: f32 = 10.0;
    pub const ITEM_PADDING_Y: f32 = 6.0;
    pub const ITEM_GAP: f32 = 8.0;
    /// `rounded-[0.625rem]` squircle.
    pub const ITEM_RADIUS: f32 = 10.0;
    pub const ITEM_FONT: f32 = 14.0;
    pub const ITEM_LINE: f32 = 20.0;
    pub const ICON: f32 = 16.0;
    /// `pr-8` of checkbox and radio rows, which hold the indicator at `right-2.5`.
    pub const INDICATOR_ROW_PADDING_RIGHT: f32 = 32.0;
    pub const INDICATOR_RIGHT: f32 = 10.0;
    /// `data-inset:pl-8`.
    pub const INSET_PADDING_LEFT: f32 = 32.0;
    /// Labels: `px-2.5 py-1.5`, 12/16 medium, 28 tall.
    pub const LABEL_PADDING_X: f32 = 10.0;
    pub const LABEL_PADDING_Y: f32 = 6.0;
    pub const LABEL_FONT: f32 = 12.0;
    pub const LABEL_LINE: f32 = 16.0;
    /// Shortcuts: `ml-auto pl-4`, 12/16 mono.
    pub const SHORTCUT_PADDING_LEFT: f32 = 16.0;
    pub const SHORTCUT_FONT: f32 = 12.0;
    pub const SHORTCUT_LINE: f32 = 16.0;
    /// Separators: `-mx-1 my-1` from shadcn, `mx-2` from Notra.
    pub const SEPARATOR_MARGIN_X: f32 = 8.0;
    pub const SEPARATOR_MARGIN_Y: f32 = 4.0;
    /// `data-disabled:opacity-50`.
    pub const DISABLED_OPACITY: f32 = 0.5;
}

/// `transition-colors duration-75` on a row's highlight.
pub const HIGHLIGHT: Tween = Tween::new(Duration::from_millis(75), ease::IN_OUT);

/// Base UI opens a submenu after the pointer rests on its trigger this long.
pub const SUBMENU_DELAY: Duration = Duration::from_millis(100);

/// shadcn's `DropdownMenuContent`: below the trigger, aligned to its start, 4 px away.
pub const PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Start).side_offset(4.0);

/// shadcn's `DropdownMenuSubContent`: to the right of its row, 3 px above it.
pub const SUBMENU_PLACEMENT: Placement =
    Placement::new(Side::Right, Align::Start).align_offset(-3.0);

/// `transition-[opacity,scale] duration-150`, `data-ending-style:duration-100`,
/// `scale-[0.97]`; `motion-reduce:transition-none`.
pub const MOTION: PopupMotion = PopupMotion::DEPTH;

/// The Depth popup surface (`DROPDOWN_POPUP_CLASS_NAME`) shared by menus and selects.
#[derive(Clone, Debug, PartialEq)]
pub struct DepthSurface {
    pub fill: Color,
    /// `bg-linear-to-b` stops, top to bottom.
    pub gradient: (Color, Color),
    pub border: Color,
    pub shadows: Vec<Shadow>,
    pub text: Color,
}

impl DepthSurface {
    pub fn new(theme: &Theme) -> Self {
        if theme.is_dark() {
            Self {
                fill: theme.popover,
                gradient: (theme.muted, theme.shell),
                border: theme.input,
                shadows: vec![
                    Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffff0f)),
                    Shadow::drop(2.0, 4.0, 0.0, Color::rgba(0x0000004d)),
                    Shadow::drop(16.0, 32.0, -8.0, Color::rgba(0x00000099)),
                ],
                text: theme.popover_foreground,
            }
        } else {
            Self {
                fill: theme.popover,
                gradient: (
                    theme.background,
                    theme.background.mix(theme.foreground, 0.025),
                ),
                border: theme.border,
                shadows: vec![
                    Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffffcc)),
                    Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x0000000d)),
                    Shadow::drop(12.0, 32.0, -8.0, Color::rgba(0x00000029)),
                ],
                text: theme.popover_foreground,
            }
        }
    }
}

/// The resolved colours of a menu's rows for one theme.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuSkin {
    pub surface: DepthSurface,
    pub text: Color,
    /// Row icons at rest (`*:[svg]:text-muted-foreground`).
    pub icon: Color,
    /// Highlighted rows: `bg-foreground/[0.05]`, dark `/[0.07]`.
    pub highlight: Color,
    /// Dark highlighted rows' top edge (`inset 0 1px 0 rgba(255,255,255,.04)`).
    pub highlight_inset: Option<Shadow>,
    /// Highlighted text and icons.
    pub highlight_text: Color,
    pub label: Color,
    pub shortcut: Color,
    pub separator: Color,
    pub destructive: Color,
    /// `focus:bg-destructive/10`, dark `/20`.
    pub destructive_highlight: Color,
}

impl MenuSkin {
    pub fn new(theme: &Theme) -> Self {
        let dark = theme.is_dark();
        Self {
            surface: DepthSurface::new(theme),
            text: theme.popover_foreground,
            icon: theme.muted_foreground,
            highlight: theme.foreground.fade(if dark { 0.07 } else { 0.05 }),
            highlight_inset: dark.then_some(Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffff0a))),
            highlight_text: theme.foreground,
            label: theme.muted_foreground,
            shortcut: theme.muted_foreground,
            separator: theme.border,
            destructive: theme.destructive,
            destructive_highlight: theme.destructive.fade(if dark { 0.2 } else { 0.1 }),
        }
    }

    /// Fill, text and icon colour of a row.
    pub fn row(&self, highlighted: bool, destructive: bool) -> (Color, Color, Color) {
        match (highlighted, destructive) {
            (false, false) => (Color::TRANSPARENT, self.text, self.icon),
            (true, false) => (self.highlight, self.highlight_text, self.highlight_text),
            (false, true) => (Color::TRANSPARENT, self.destructive, self.destructive),
            (true, true) => (
                self.destructive_highlight,
                self.destructive,
                self.destructive,
            ),
        }
    }
}

/// What a row does when activated.
#[derive(Clone, Debug, PartialEq)]
pub enum ItemKind {
    /// A plain item: runs and closes the menu.
    Action,
    /// `DropdownMenuCheckboxItem`: toggles and stays open.
    Checkbox { checked: bool },
    /// `DropdownMenuRadioItem`: picks and stays open.
    Radio { checked: bool },
    /// `DropdownMenuSubTrigger`: opens a nested menu.
    Submenu(Vec<MenuEntry>),
}

/// One row of a menu.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuItem {
    /// Handed back when the row is activated.
    pub id: String,
    pub label: String,
    pub icon: Option<Icon>,
    /// `DropdownMenuShortcut` text (`⌘,`).
    pub shortcut: Option<String>,
    pub kind: ItemKind,
    pub disabled: bool,
    /// `variant="destructive"`.
    pub destructive: bool,
    /// `inset`: text lines up with rows that have an icon.
    pub inset: bool,
}

impl MenuItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            shortcut: None,
            kind: ItemKind::Action,
            disabled: false,
            destructive: false,
            inset: false,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn checkbox(mut self, checked: bool) -> Self {
        self.kind = ItemKind::Checkbox { checked };
        self
    }

    pub fn radio(mut self, checked: bool) -> Self {
        self.kind = ItemKind::Radio { checked };
        self
    }

    pub fn submenu(mut self, entries: Vec<MenuEntry>) -> Self {
        self.kind = ItemKind::Submenu(entries);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    pub fn inset(mut self) -> Self {
        self.inset = true;
        self
    }

    /// The checked state of a checkbox or radio row.
    pub fn checked(&self) -> Option<bool> {
        match self.kind {
            ItemKind::Checkbox { checked } | ItemKind::Radio { checked } => Some(checked),
            _ => None,
        }
    }

    pub fn submenu_entries(&self) -> Option<&[MenuEntry]> {
        match &self.kind {
            ItemKind::Submenu(entries) => Some(entries),
            _ => None,
        }
    }

    /// ARIA role: `menuitem`, `menuitemcheckbox` or `menuitemradio`.
    pub fn role(&self) -> &'static str {
        match self.kind {
            ItemKind::Checkbox { .. } => "menuitemcheckbox",
            ItemKind::Radio { .. } => "menuitemradio",
            _ => "menuitem",
        }
    }

    /// Whether activating the row closes the whole menu.
    pub fn closes_menu(&self) -> bool {
        matches!(self.kind, ItemKind::Action)
    }
}

/// One entry of a menu: a row, a group label or a separator.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuEntry {
    Item(MenuItem),
    /// `DropdownMenuLabel`.
    Label(String),
    Separator,
}

impl From<MenuItem> for MenuEntry {
    fn from(item: MenuItem) -> Self {
        MenuEntry::Item(item)
    }
}

impl MenuEntry {
    /// The row, if this entry is an enabled one.
    pub fn enabled_item(&self) -> Option<&MenuItem> {
        match self {
            MenuEntry::Item(item) if !item.disabled => Some(item),
            _ => None,
        }
    }
}

/// Where the highlight starts when a menu opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenFocus {
    /// Opened by the pointer: nothing highlighted.
    None,
    /// Enter, Space or ArrowDown on the trigger.
    First,
    /// ArrowUp on the trigger.
    Last,
}

/// Keys a menu reacts to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuKey {
    Up,
    Down,
    Home,
    End,
    Left,
    Right,
    Enter,
    Space,
    Escape,
    Tab,
    /// A printable character, for type-ahead.
    Char(char),
}

/// What a key or press asks the menu's owner to do.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuCommand {
    None,
    /// The highlight moved or a submenu opened or closed.
    Changed,
    /// Run the row at `path` (indices into the entries at each level). `close` is whether the
    /// menu closes afterwards.
    Activate {
        path: Vec<usize>,
        close: bool,
    },
    /// Close the whole menu (Escape at the root, Tab).
    Close,
}

/// Base UI's menu navigation: one highlighted row per open level, the deepest level taking
/// keys. `levels[0]` is the root menu; each further level is the submenu opened from the
/// highlighted row of the level before.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuNav {
    levels: Vec<Option<usize>>,
}

impl MenuNav {
    /// A menu just opened, highlighting per `focus`.
    pub fn opened(entries: &[MenuEntry], focus: OpenFocus) -> Self {
        let first = match focus {
            OpenFocus::None => None,
            OpenFocus::First => step(entries, None, 1),
            OpenFocus::Last => step(entries, None, -1),
        };
        Self {
            levels: vec![first],
        }
    }

    /// The highlighted row at `level`.
    pub fn highlighted(&self, level: usize) -> Option<usize> {
        self.levels.get(level).copied().flatten()
    }

    /// How many menus are open (the root and its open submenus).
    pub fn depth(&self) -> usize {
        self.levels.len()
    }

    /// Whether the row at `index` of `level` has its submenu open.
    pub fn is_submenu_open(&self, level: usize, index: usize) -> bool {
        self.levels.len() > level + 1 && self.highlighted(level) == Some(index)
    }

    /// The entries shown at `level`, following the open submenus from `root`.
    pub fn entries_at<'a>(&self, root: &'a [MenuEntry], level: usize) -> Option<&'a [MenuEntry]> {
        let mut entries = root;
        for depth in 0..level {
            let index = self.highlighted(depth)?;
            entries = match entries.get(index)? {
                MenuEntry::Item(item) => item.submenu_entries()?,
                _ => return None,
            };
        }
        Some(entries)
    }

    /// The pointer moved onto row `index` of `level`: it takes the highlight and closes
    /// deeper submenus opened from other rows. Returns whether that row is a submenu trigger
    /// whose menu isn't open yet (open it after [`SUBMENU_DELAY`]).
    pub fn hover(&mut self, root: &[MenuEntry], level: usize, index: usize) -> bool {
        let Some(entries) = self.entries_at(root, level) else {
            return false;
        };
        let Some(item) = entries.get(index).and_then(MenuEntry::enabled_item) else {
            return false;
        };
        if level >= self.levels.len() {
            return false;
        }
        if self.highlighted(level) != Some(index) {
            self.levels.truncate(level + 1);
        }
        self.levels[level] = Some(index);
        item.submenu_entries().is_some() && !self.is_submenu_open(level, index)
    }

    /// The pointer left the rows of `level` without entering a submenu: the highlight goes,
    /// unless the row's submenu is open.
    pub fn unhover(&mut self, level: usize) {
        if self.levels.len() == level + 1
            && let Some(slot) = self.levels.get_mut(level)
        {
            *slot = None;
        }
    }

    /// Opens the submenu of the highlighted row at `level`, highlighting its first row when
    /// opened from the keyboard.
    pub fn open_submenu(&mut self, root: &[MenuEntry], level: usize, keyboard: bool) -> bool {
        let Some(entries) = self.entries_at(root, level) else {
            return false;
        };
        let Some(sub) = self
            .highlighted(level)
            .and_then(|i| entries.get(i))
            .and_then(MenuEntry::enabled_item)
            .and_then(MenuItem::submenu_entries)
        else {
            return false;
        };
        self.levels.truncate(level + 1);
        self.levels
            .push(if keyboard { step(sub, None, 1) } else { None });
        true
    }

    /// Closes submenus deeper than `level`.
    pub fn close_submenus(&mut self, level: usize) {
        self.levels.truncate(level + 1);
    }

    /// The path of the row a press at `index` of `level` activates.
    pub fn press(&mut self, root: &[MenuEntry], level: usize, index: usize) -> MenuCommand {
        let Some(item) = self
            .entries_at(root, level)
            .and_then(|e| e.get(index))
            .and_then(MenuEntry::enabled_item)
        else {
            return MenuCommand::None;
        };
        if item.submenu_entries().is_some() {
            self.levels.truncate(level + 1);
            self.levels[level] = Some(index);
            self.open_submenu(root, level, false);
            return MenuCommand::Changed;
        }
        let mut path: Vec<usize> = (0..level).filter_map(|l| self.highlighted(l)).collect();
        path.push(index);
        MenuCommand::Activate {
            path,
            close: item.closes_menu(),
        }
    }

    /// A key pressed while the menu has focus.
    pub fn key(&mut self, root: &[MenuEntry], key: MenuKey) -> MenuCommand {
        let level = self.levels.len().saturating_sub(1);
        let Some(entries) = self.entries_at(root, level) else {
            return MenuCommand::None;
        };
        let current = self.highlighted(level);
        let moved = |nav: &mut Self, to: Option<usize>| {
            if to.is_some() && to != current {
                nav.levels[level] = to;
                MenuCommand::Changed
            } else {
                MenuCommand::None
            }
        };
        match key {
            MenuKey::Down => moved(self, step(entries, current, 1)),
            MenuKey::Up => moved(self, step(entries, current, -1)),
            MenuKey::Home => moved(self, step(entries, None, 1)),
            MenuKey::End => moved(self, step(entries, None, -1)),
            MenuKey::Char(c) => moved(self, type_ahead(entries, current, c)),
            MenuKey::Right => {
                if self.open_submenu(root, level, true) {
                    MenuCommand::Changed
                } else {
                    MenuCommand::None
                }
            }
            MenuKey::Left if level > 0 => {
                self.levels.truncate(level);
                MenuCommand::Changed
            }
            MenuKey::Left => MenuCommand::None,
            MenuKey::Escape if level > 0 => {
                self.levels.truncate(level);
                MenuCommand::Changed
            }
            MenuKey::Escape | MenuKey::Tab => MenuCommand::Close,
            MenuKey::Enter | MenuKey::Space => match current {
                Some(index) => {
                    let opens = entries
                        .get(index)
                        .and_then(MenuEntry::enabled_item)
                        .is_some_and(|i| i.submenu_entries().is_some());
                    if opens {
                        self.open_submenu(root, level, true);
                        MenuCommand::Changed
                    } else {
                        self.press(root, level, index)
                    }
                }
                None => MenuCommand::None,
            },
        }
    }
}

/// The row at `path`.
pub fn item_at<'a>(root: &'a [MenuEntry], path: &[usize]) -> Option<&'a MenuItem> {
    let (last, parents) = path.split_last()?;
    let mut entries = root;
    for &i in parents {
        entries = match entries.get(i)? {
            MenuEntry::Item(item) => item.submenu_entries()?,
            _ => return None,
        };
    }
    match entries.get(*last)? {
        MenuEntry::Item(item) => Some(item),
        _ => None,
    }
}

/// The next enabled row from `from` in `dir` (±1), looping; from `None` the first or last.
fn step(entries: &[MenuEntry], from: Option<usize>, dir: isize) -> Option<usize> {
    let n = entries.len() as isize;
    if n == 0 {
        return None;
    }
    let start = match from {
        Some(i) => i as isize,
        None if dir > 0 => -1,
        None => n,
    };
    (1..=n)
        .map(|k| (start + dir * k).rem_euclid(n) as usize)
        .find(|&i| entries[i].enabled_item().is_some())
}

/// The next enabled row after `from` whose label starts with `c`, looping.
fn type_ahead(entries: &[MenuEntry], from: Option<usize>, c: char) -> Option<usize> {
    let n = entries.len();
    let start = from.map_or(n - 1, |i| i);
    let lower: String = c.to_lowercase().collect();
    (1..=n).map(|k| (start + k) % n).find(|&i| {
        entries[i]
            .enabled_item()
            .is_some_and(|item| item.label.to_lowercase().starts_with(&lower))
    })
}

/// The live demo's menu (`dropdown-menu-demo`).
pub fn demo_entries(show_activity: bool, view: &str) -> Vec<MenuEntry> {
    vec![
        MenuEntry::Label("Workspace".into()),
        MenuItem::new("settings", "Settings")
            .icon(Icon::Settings)
            .shortcut("⌘,")
            .into(),
        MenuItem::new("invite", "Invite members")
            .icon(Icon::Users)
            .into(),
        MenuItem::new("export", "Export")
            .icon(Icon::Download)
            .submenu(vec![
                MenuItem::new("markdown", "Markdown").into(),
                MenuItem::new("csv", "CSV").into(),
                MenuItem::new("pdf", "PDF").disabled(true).into(),
            ])
            .into(),
        MenuEntry::Separator,
        MenuItem::new("activity", "Show activity")
            .checkbox(show_activity)
            .into(),
        MenuEntry::Separator,
        MenuEntry::Label("View".into()),
        MenuItem::new("team", "Team").radio(view == "team").into(),
        MenuItem::new("personal", "Personal")
            .radio(view == "personal")
            .into(),
        MenuEntry::Separator,
        MenuItem::new("delete", "Delete workspace")
            .icon(Icon::Trash2)
            .destructive()
            .into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label(entries: &[MenuEntry], nav: &MenuNav, level: usize) -> String {
        let entries = nav.entries_at(entries, level).unwrap();
        match &entries[nav.highlighted(level).unwrap()] {
            MenuEntry::Item(item) => item.label.clone(),
            _ => panic!("highlight on a non-item"),
        }
    }

    #[test]
    fn opening_from_the_keyboard_highlights_an_end() {
        let e = demo_entries(true, "team");
        assert_eq!(
            label(&e, &MenuNav::opened(&e, OpenFocus::First), 0),
            "Settings"
        );
        assert_eq!(
            label(&e, &MenuNav::opened(&e, OpenFocus::Last), 0),
            "Delete workspace"
        );
        assert_eq!(MenuNav::opened(&e, OpenFocus::None).highlighted(0), None);
    }

    #[test]
    fn arrows_skip_labels_and_separators_and_loop() {
        let e = demo_entries(true, "team");
        let mut nav = MenuNav::opened(&e, OpenFocus::Last);
        nav.key(&e, MenuKey::Down);
        assert_eq!(label(&e, &nav, 0), "Settings");
        nav.key(&e, MenuKey::Up);
        nav.key(&e, MenuKey::Up);
        assert_eq!(label(&e, &nav, 0), "Personal");
        nav.key(&e, MenuKey::Up);
        nav.key(&e, MenuKey::Up);
        assert_eq!(label(&e, &nav, 0), "Show activity");
    }

    #[test]
    fn type_ahead_finds_the_next_match() {
        let e = demo_entries(true, "team");
        let mut nav = MenuNav::opened(&e, OpenFocus::First);
        nav.key(&e, MenuKey::Char('s'));
        assert_eq!(label(&e, &nav, 0), "Show activity");
        nav.key(&e, MenuKey::Char('S'));
        assert_eq!(label(&e, &nav, 0), "Settings");
    }

    #[test]
    fn submenus_open_and_close_from_the_keyboard() {
        let e = demo_entries(true, "team");
        let mut nav = MenuNav::opened(&e, OpenFocus::First);
        nav.key(&e, MenuKey::Down);
        nav.key(&e, MenuKey::Down);
        assert_eq!(nav.key(&e, MenuKey::Right), MenuCommand::Changed);
        assert_eq!(nav.depth(), 2);
        assert_eq!(label(&e, &nav, 1), "Markdown");
        nav.key(&e, MenuKey::Up);
        assert_eq!(label(&e, &nav, 1), "CSV", "the disabled PDF is skipped");
        assert_eq!(
            nav.key(&e, MenuKey::Enter),
            MenuCommand::Activate {
                path: vec![3, 1],
                close: true
            }
        );
        assert_eq!(nav.key(&e, MenuKey::Escape), MenuCommand::Changed);
        assert_eq!(nav.depth(), 1);
        assert_eq!(nav.key(&e, MenuKey::Escape), MenuCommand::Close);
    }

    #[test]
    fn checkbox_and_radio_rows_keep_the_menu_open() {
        let e = demo_entries(true, "team");
        let mut nav = MenuNav::opened(&e, OpenFocus::None);
        assert_eq!(
            nav.press(&e, 0, 5),
            MenuCommand::Activate {
                path: vec![5],
                close: false
            }
        );
        assert_eq!(item_at(&e, &[5]).map(|i| i.id.as_str()), Some("activity"));
        assert_eq!(
            nav.press(&e, 0, 1),
            MenuCommand::Activate {
                path: vec![1],
                close: true
            }
        );
    }

    #[test]
    fn hovering_another_row_closes_the_open_submenu() {
        let e = demo_entries(true, "team");
        let mut nav = MenuNav::opened(&e, OpenFocus::None);
        assert!(nav.hover(&e, 0, 3), "Export wants its submenu");
        nav.open_submenu(&e, 0, false);
        assert!(nav.is_submenu_open(0, 3));
        assert!(!nav.hover(&e, 0, 3));
        nav.hover(&e, 1, 0);
        assert_eq!(nav.highlighted(1), Some(0));
        assert!(!nav.hover(&e, 0, 1));
        assert_eq!(nav.depth(), 1);
        nav.unhover(0);
        assert_eq!(nav.highlighted(0), None);
    }

    #[test]
    fn light_and_dark_surfaces_follow_the_class_list() {
        let light = MenuSkin::new(&Theme::light());
        assert_eq!(light.surface.border, Theme::light().border);
        assert!(light.highlight_inset.is_none());
        let dark = MenuSkin::new(&Theme::dark());
        assert_eq!(
            dark.surface.gradient,
            (Theme::dark().muted, Theme::dark().shell)
        );
        assert!(dark.highlight_inset.is_some());
    }
}
