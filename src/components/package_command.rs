//! Notra's package command: an install command with bun, pnpm, yarn and npm tabs,
//! rewritten for each manager, and a copy button.

use crate::icons::Icon;

/// A JavaScript package manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum PackageManager {
    #[default]
    Bun,
    Pnpm,
    Yarn,
    Npm,
}

impl PackageManager {
    pub const ALL: [PackageManager; 4] = [
        PackageManager::Bun,
        PackageManager::Pnpm,
        PackageManager::Yarn,
        PackageManager::Npm,
    ];

    pub fn name(self) -> &'static str {
        match self {
            PackageManager::Bun => "bun",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Yarn => "yarn",
            PackageManager::Npm => "npm",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            PackageManager::Bun => Icon::PmBun,
            PackageManager::Pnpm => Icon::PmPnpm,
            PackageManager::Yarn => Icon::PmYarn,
            PackageManager::Npm => Icon::PmNpm,
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    /// The manager a stored name names (`"pnpm"`), as `toPackageManager` reads storage.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.name() == name)
    }
}

/// An npm command rewritten for every manager.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageCommands([String; 4]);

impl PackageCommands {
    /// Rewrites `npm install`, `npx create-`, `npm create`, `npx` and `npm run`
    /// commands; anything else runs as is under every manager.
    pub fn from_npm(command: &str) -> Self {
        const RULES: [(&str, [&str; 3]); 5] = [
            ("npm install", ["bun add", "pnpm add", "yarn add"]),
            (
                "npx create-",
                ["bun create ", "pnpm create ", "yarn create "],
            ),
            ("npm create", ["bun create", "pnpm create", "yarn create"]),
            ("npx", ["bunx --bun", "pnpm dlx", "yarn dlx"]),
            ("npm run", ["bun run", "pnpm", "yarn"]),
        ];
        let swap = |from: &str, to: &str| command.replacen(from, to, 1);
        match RULES.iter().find(|(from, _)| command.starts_with(from)) {
            Some((from, [bun, pnpm, yarn])) => Self([
                swap(from, bun),
                swap(from, pnpm),
                swap(from, yarn),
                command.to_owned(),
            ]),
            None => Self(std::array::from_fn(|_| command.to_owned())),
        }
    }

    pub fn get(&self, manager: PackageManager) -> &str {
        &self.0[manager.index()]
    }
}

/// Box metrics, in logical pixels.
pub mod metrics {
    pub const SHELL_RADIUS: f32 = 16.0;
    /// `p-0.5 pt-0`.
    pub const SHELL_PADDING: f32 = 2.0;
    /// The tab list's `px-3`.
    pub const TABS_PADDING_X: f32 = 12.0;
    pub const PANEL_HEIGHT: f32 = 44.0;
    pub const PANEL_RADIUS: f32 = 14.0;
    pub const PANEL_PADDING_START: f32 = 14.0;
    pub const PANEL_PADDING_END: f32 = 6.0;
    pub const PANEL_GAP: f32 = 8.0;
    /// `text-[0.8125rem]`.
    pub const CODE_SIZE: f32 = 13.0;
    pub const CODE_LINE_HEIGHT: f32 = 19.5;
    /// The command fades out over its last 16 px.
    pub const CODE_FADE: f32 = 16.0;
    pub const PROMPT: &str = "$ ";
}

/// The tab list's accessible name.
pub const TABS_LABEL: &str = "Package manager";

/// The package command demo.
pub mod demo {
    pub const COMMANDS: [&str; 2] = [
        "npx shadcn@latest add @notra/package-command",
        "npm install @usenotra/geo",
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_installs_and_runners() {
        let c = PackageCommands::from_npm("npm install @usenotra/geo");
        assert_eq!(c.get(PackageManager::Bun), "bun add @usenotra/geo");
        assert_eq!(c.get(PackageManager::Npm), "npm install @usenotra/geo");
        let c = PackageCommands::from_npm("npx shadcn@latest add @notra/card");
        assert_eq!(
            c.get(PackageManager::Bun),
            "bunx --bun shadcn@latest add @notra/card"
        );
        assert_eq!(
            c.get(PackageManager::Pnpm),
            "pnpm dlx shadcn@latest add @notra/card"
        );
        let c = PackageCommands::from_npm("npx create-next-app");
        assert_eq!(c.get(PackageManager::Yarn), "yarn create next-app");
        let c = PackageCommands::from_npm("npm run dev");
        assert_eq!(c.get(PackageManager::Pnpm), "pnpm dev");
        let c = PackageCommands::from_npm("git status");
        assert_eq!(c.get(PackageManager::Bun), "git status");
    }

    #[test]
    fn names_round_trip() {
        for m in PackageManager::ALL {
            assert_eq!(PackageManager::from_name(m.name()), Some(m));
        }
        assert_eq!(PackageManager::default(), PackageManager::Bun);
    }
}
