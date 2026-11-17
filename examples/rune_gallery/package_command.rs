use notra_ui::Theme;
use notra_ui::components::package_command::demo;
use notra_ui::rune::{PackageCommand, PackageManager};
use rune::{El, col};

use crate::{State, section};

#[derive(Default)]
pub struct Demo {
    pub manager: PackageManager,
    pub copies: [u32; 2],
}

pub fn view(s: &State, t: &Theme) -> El {
    let commands = demo::COMMANDS.iter().enumerate().map(|(i, command)| {
        PackageCommand::new(*command, s.package_command.manager)
            .copies(s.package_command.copies[i])
            .on_select(|s: &mut State, m| s.package_command.manager = m)
            .on_copy(move |s: &mut State| s.package_command.copies[i] += 1)
            .build(t)
            .key(format!("package-command-{i}"))
    });
    col().child(section(
        "Demo",
        t,
        col().gap(16.0).max_w(528.0).w_full().children(commands),
    ))
}
