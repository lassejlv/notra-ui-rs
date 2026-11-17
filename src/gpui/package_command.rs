//! GPUI rendering of the package command.

use ::gpui::{
    App, ElementId, FontFeatures, Global, IntoElement, ParentElement, RenderOnce, SharedString,
    Window, div, linear_color_stop, linear_gradient, prelude::*, px,
};

use super::data_parts::{CopyButton, CopyLabel, TabOption, Tabs};
use super::{ActiveTheme, shadows};
use crate::components::data_parts::copy;
use crate::components::package_command::{PackageCommands, PackageManager, metrics};
use crate::fonts;

/// The package manager every [`PackageCommand`] shows, shared app-wide as the site
/// shares it through local storage.
pub struct SelectedPackageManager(pub PackageManager);

impl Global for SelectedPackageManager {}

/// Notra's package command: `PackageCommand::new("add-card", "npx shadcn@latest add @notra/card")`.
#[derive(IntoElement)]
pub struct PackageCommand {
    id: ElementId,
    command: SharedString,
}

impl PackageCommand {
    /// `command` is the npm form; the other managers' forms are derived from it.
    pub fn new(id: impl Into<ElementId>, command: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            command: command.into(),
        }
    }
}

impl RenderOnce for PackageCommand {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let manager = cx
            .try_global::<SelectedPackageManager>()
            .map_or_else(PackageManager::default, |m| m.0);
        let commands = PackageCommands::from_npm(&self.command);
        let command = commands.get(manager).to_owned();
        let id = self.id.clone();
        let options = PackageManager::ALL
            .iter()
            .map(|m| TabOption::new(m.name()).icon(m.icon()))
            .collect();
        let fade = linear_gradient(
            90.0,
            linear_color_stop(theme.table_card.fade(0.0), 0.0),
            linear_color_stop(theme.table_card, 1.0),
        );
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .w_full()
            .min_w_0()
            .px(px(metrics::SHELL_PADDING))
            .pb(px(metrics::SHELL_PADDING))
            .rounded(px(metrics::SHELL_RADIUS))
            .border_1()
            .border_color(theme.shell_border)
            .bg(theme.shell)
            .child(
                div().px(px(metrics::TABS_PADDING_X)).child(
                    Tabs::new(
                        SharedString::from(format!("{id}-tabs")),
                        options,
                        manager.index(),
                    )
                    .mono(true)
                    .on_change(|i, _, cx| {
                        cx.set_global(SelectedPackageManager(PackageManager::ALL[*i]));
                        cx.refresh_windows();
                    }),
                ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(metrics::PANEL_HEIGHT))
                    .min_w_0()
                    .gap(px(metrics::PANEL_GAP))
                    .pl(px(metrics::PANEL_PADDING_START))
                    .pr(px(metrics::PANEL_PADDING_END))
                    .rounded(px(metrics::PANEL_RADIUS))
                    .border_1()
                    .border_color(theme.table_card_border)
                    .bg(theme.table_card)
                    .shadow(shadows(theme.table_card_shadow))
                    .child(
                        div()
                            .relative()
                            .flex()
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .font_family(fonts::MONO)
                            .font_features(FontFeatures::disable_ligatures())
                            .text_size(px(metrics::CODE_SIZE))
                            .line_height(px(metrics::CODE_LINE_HEIGHT))
                            .text_color(theme.foreground)
                            .child(
                                div()
                                    .flex_none()
                                    .text_color(theme.muted_foreground)
                                    .child(metrics::PROMPT),
                            )
                            .child(div().flex_none().child(command.clone()))
                            .child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .bottom_0()
                                    .right_0()
                                    .w(px(metrics::CODE_FADE))
                                    .bg(fade),
                            ),
                    )
                    .child(CopyButton::new(
                        SharedString::from(format!("{id}-copy-{}", manager.name())),
                        command,
                        copy::COMMAND,
                        CopyLabel::Command,
                    )),
            )
    }
}
