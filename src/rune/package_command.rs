//! Rune rendering of the package command.

use ::rune::{El, el, row, text};

use super::data_parts::{CopyButton, TabOption, Tabs};
use super::{ToRune, mono};
use crate::components::data_parts::copy;
use crate::components::package_command::{PackageCommands, PackageManager, TABS_LABEL, metrics};
use crate::theme::Theme;

/// Browsers draw these in the system mono, which has no programming ligatures.
pub(crate) const NO_LIGATURES: &str = "'calt' 0, 'liga' 0";

type Hook<T> = Box<dyn FnOnce(T) -> T>;

/// Notra's package command. The app owns the selected manager (shared by every
/// command it shows) and a copy count per command.
///
/// ```ignore
/// PackageCommand::new("npm install @usenotra/geo", s.manager)
///     .copies(s.copies)
///     .on_select(|s: &mut State, m| s.manager = m)
///     .on_copy(|s: &mut State| s.copies += 1)
///     .build(&theme)
/// ```
pub struct PackageCommand {
    command: String,
    manager: PackageManager,
    copies: u32,
    on_select: Option<Hook<Tabs>>,
    on_copy: Option<Hook<CopyButton>>,
}

impl PackageCommand {
    /// `command` is the npm form; the other managers' forms are derived from it.
    pub fn new(command: impl Into<String>, manager: PackageManager) -> Self {
        Self {
            command: command.into(),
            manager,
            copies: 0,
            on_select: None,
            on_copy: None,
        }
    }

    pub fn copies(mut self, copies: u32) -> Self {
        self.copies = copies;
        self
    }

    pub fn on_select<S: 'static>(
        mut self,
        f: impl Fn(&mut S, PackageManager) + Clone + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(move |tabs: Tabs| {
            tabs.on_select(move |s: &mut S, i| f(s, PackageManager::ALL[i]))
        }));
        self
    }

    pub fn on_copy<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.on_copy = Some(Box::new(move |b: CopyButton| b.on_copy(f)));
        self
    }

    pub fn build(self, theme: &Theme) -> El {
        let commands = PackageCommands::from_npm(&self.command);
        let command = commands.get(self.manager).to_owned();
        let options = PackageManager::ALL
            .iter()
            .map(|m| TabOption::new(m.name()).icon(m.icon()))
            .collect();
        let mut tabs = Tabs::new(options, self.manager.index())
            .mono(true)
            .label(TABS_LABEL);
        if let Some(hook) = self.on_select {
            tabs = hook(tabs);
        }
        let mut button = CopyButton::new(
            command.clone(),
            copy::COMMAND,
            copy::command_label(copy::CopyStatus::Idle),
        )
        .copies(self.copies);
        if let Some(hook) = self.on_copy {
            button = hook(button);
        }
        let font =
            mono(metrics::CODE_SIZE, 400.0, metrics::CODE_LINE_HEIGHT).features(NO_LIGATURES);
        let code = row()
            .key("command")
            .grow(1.0)
            .min_w(0.0)
            .clip_fade_linear(0.0, 0.0, metrics::CODE_FADE, 0.0)
            .child(
                text(metrics::PROMPT)
                    .font(font)
                    .color(theme.muted_foreground.rune()),
            )
            .child(
                text(command)
                    .key("command-text")
                    .font(font)
                    .color(theme.foreground.rune())
                    .selectable(true),
            )
            .child(el().w(metrics::CODE_FADE));
        let panel = row()
            .key(format!("panel-{}", self.manager.name()))
            .h(metrics::PANEL_HEIGHT)
            .center_y()
            .gap(metrics::PANEL_GAP)
            .pl(metrics::PANEL_PADDING_START)
            .pr(metrics::PANEL_PADDING_END)
            .radius(metrics::PANEL_RADIUS)
            .border(1.0, theme.table_card_border.rune())
            .fill(theme.table_card.rune())
            .role("tabpanel")
            .child(code)
            .child(button.build(theme).key("copy"));
        let panel = theme
            .table_card_shadow
            .iter()
            .fold(panel, |p, s| p.box_shadow(s.rune()));
        ::rune::col()
            .w_full()
            .min_w(0.0)
            .px(metrics::SHELL_PADDING)
            .pb(metrics::SHELL_PADDING)
            .radius(metrics::SHELL_RADIUS)
            .border(1.0, theme.shell_border.rune())
            .fill(theme.shell.rune())
            .child(el().px(metrics::TABS_PADDING_X).child(tabs.build(theme)))
            .child(panel)
    }
}
