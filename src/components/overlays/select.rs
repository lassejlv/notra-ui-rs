//! Notra's select (`notra-select`): a Depth button trigger and a scrolling listbox in the
//! Depth popup below it, with Base UI's select keyboard behaviour as a state machine.

use super::menu::metrics as menu_metrics;
use super::{Align, Collision, Placement, Side};
use crate::components::button::{ButtonMetrics, ButtonSize, ButtonSkin, ButtonVariant};
use crate::theme::{Color, Theme};

pub use super::menu::{DepthSurface, MOTION, MenuSkin as SelectSkin};

/// Measured box metrics, in logical pixels.
pub mod metrics {
    /// `min-w-36`; otherwise the popup is as wide as its trigger.
    pub const MIN_WIDTH: f32 = 144.0;
    /// `SelectPrimitive.List`'s `p-1` inside the surface's `p-0`.
    pub const LIST_PADDING: f32 = 4.0;
    /// shadcn's `SelectGroup` adds `p-1` of its own around its rows.
    pub const GROUP_PADDING: f32 = 4.0;
    /// The chevrons at the trigger's end.
    pub const CHEVRON: f32 = 16.0;
    /// Height of the scroll arrows shown when the list overflows.
    pub const SCROLL_BUTTON: f32 = 24.0;
}

/// `side="bottom" sideOffset={4} align="start"`.
pub const PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Start).side_offset(4.0);

/// `SELECT_COLLISION_AVOIDANCE`: flip the side, shift the alignment.
pub const COLLISION: Collision = Collision::FLIP_SHIFT;

/// `<SelectTrigger variant>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SelectVariant {
    /// The Depth outline button.
    #[default]
    Default,
    /// The ghost button, for inline property pickers.
    Ghost,
}

/// `<SelectTrigger size>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SelectSize {
    #[default]
    Default,
    Sm,
}

impl SelectSize {
    /// The button size the trigger borrows (`buttonVariants({ size })`).
    pub fn button(self) -> ButtonSize {
        match self {
            SelectSize::Default => ButtonSize::Default,
            SelectSize::Sm => ButtonSize::Sm,
        }
    }

    pub fn metrics(self) -> ButtonMetrics {
        self.button().metrics()
    }
}

/// The trigger's resolved paint: the outline or ghost button skin, with the select's own
/// `aria-expanded` fill, regular weight text and muted chevrons.
#[derive(Clone, Debug, PartialEq)]
pub struct TriggerSkin {
    pub button: ButtonSkin,
    /// The button while its popup is open (`aria-expanded:from-secondary to-secondary` on the
    /// Depth trigger in light mode, `aria-expanded:bg-muted` on the ghost one).
    pub expanded: ButtonSkin,
    pub placeholder: Color,
    pub chevron: Color,
}

impl TriggerSkin {
    pub fn new(theme: &Theme, variant: SelectVariant) -> Self {
        let button = ButtonSkin::new(
            theme,
            match variant {
                SelectVariant::Default => ButtonVariant::Outline,
                SelectVariant::Ghost => ButtonVariant::Ghost,
            },
        );
        let mut expanded = button.clone();
        match variant {
            SelectVariant::Default if !theme.is_dark() => {
                expanded.gradient = Some((theme.secondary, theme.secondary));
            }
            SelectVariant::Default => {}
            SelectVariant::Ghost => {
                expanded = button.hovered();
                expanded.fill = theme.muted;
            }
        }
        Self {
            button,
            expanded,
            placeholder: theme.muted_foreground,
            chevron: theme.muted_foreground,
        }
    }
}

/// `transition-[background-color,border-color,box-shadow,filter]` at Tailwind's default.
pub use crate::components::button::TRANSITION;

/// One option of a select.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// One entry of a select's list.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectEntry {
    /// `SelectGroup`, optionally headed by a `SelectLabel`.
    Group {
        label: Option<String>,
        options: Vec<SelectOption>,
    },
    Option(SelectOption),
    /// `SelectSeparator`.
    Separator,
}

impl From<SelectOption> for SelectEntry {
    fn from(option: SelectOption) -> Self {
        SelectEntry::Option(option)
    }
}

/// Every option in list order.
pub fn options(entries: &[SelectEntry]) -> Vec<&SelectOption> {
    entries
        .iter()
        .flat_map(|e| match e {
            SelectEntry::Group { options, .. } => options.iter().collect::<Vec<_>>(),
            SelectEntry::Option(o) => vec![o],
            SelectEntry::Separator => Vec::new(),
        })
        .collect()
}

/// The label of the option whose value is `value`.
pub fn label_of<'a>(entries: &'a [SelectEntry], value: &str) -> Option<&'a str> {
    options(entries)
        .into_iter()
        .find(|o| o.value == value)
        .map(|o| o.label.as_str())
}

/// Keys a select's list reacts to.
pub use super::menu::MenuKey as SelectKey;

/// What a key asks the select's owner to do.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectCommand {
    None,
    /// The highlight moved.
    Changed,
    /// Pick this value and close.
    Select(String),
    Close,
}

/// Base UI's select list navigation over the flattened options: the highlight starts on the
/// selected option, arrows stop at the ends, disabled options can be highlighted but not
/// picked, and typing jumps to the next option starting with that letter.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectNav {
    highlighted: Option<usize>,
}

impl SelectNav {
    /// A list just opened on `value` (or the first option when nothing is selected and the
    /// keyboard opened it).
    pub fn opened(entries: &[SelectEntry], value: Option<&str>, keyboard: bool) -> Self {
        let all = options(entries);
        let selected = value.and_then(|v| all.iter().position(|o| o.value == v));
        Self {
            highlighted: selected.or((keyboard && !all.is_empty()).then_some(0)),
        }
    }

    /// Index into [`options`] of the highlighted option.
    pub fn highlighted(&self) -> Option<usize> {
        self.highlighted
    }

    /// The pointer moved onto option `index`.
    pub fn hover(&mut self, index: usize) {
        self.highlighted = Some(index);
    }

    pub fn key(&mut self, entries: &[SelectEntry], key: SelectKey) -> SelectCommand {
        let all = options(entries);
        let last = all.len().checked_sub(1);
        let current = self.highlighted;
        let to = match key {
            SelectKey::Down => current.map_or(Some(0), |i| Some((i + 1).min(last.unwrap_or(0)))),
            SelectKey::Up => current.map_or(last, |i| Some(i.saturating_sub(1))),
            SelectKey::Home => (!all.is_empty()).then_some(0),
            SelectKey::End => last,
            SelectKey::Char(c) => {
                let lower: String = c.to_lowercase().collect();
                let n = all.len();
                let start = current.unwrap_or(n.saturating_sub(1));
                (1..=n)
                    .map(|k| (start + k) % n)
                    .find(|&i| all[i].label.to_lowercase().starts_with(&lower))
            }
            SelectKey::Enter | SelectKey::Space => {
                return match current.and_then(|i| all.get(i)) {
                    Some(o) if !o.disabled => SelectCommand::Select(o.value.clone()),
                    _ => SelectCommand::None,
                };
            }
            SelectKey::Escape | SelectKey::Tab => return SelectCommand::Close,
            SelectKey::Left | SelectKey::Right => return SelectCommand::None,
        };
        if to.is_some() && to != current {
            self.highlighted = to;
            SelectCommand::Changed
        } else {
            SelectCommand::None
        }
    }
}

/// The row padding of an option: the dropdown row with `pr-8` for the check.
pub const OPTION_PADDING_RIGHT: f32 = menu_metrics::INDICATOR_ROW_PADDING_RIGHT;

/// The live demo's options (`FREQUENCY_OPTIONS`).
pub fn frequency_options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("daily", "Daily"),
        SelectOption::new("weekly", "Weekly"),
        SelectOption::new("monthly", "Monthly"),
        SelectOption::new("custom", "Custom"),
    ]
}

/// The first demo select: a labelled group, a separator and a disabled option.
pub fn demo_grouped() -> Vec<SelectEntry> {
    let mut options = frequency_options();
    let custom = options.pop().map(|o| o.disabled(true));
    let mut entries = vec![
        SelectEntry::Group {
            label: Some("Frequency".into()),
            options,
        },
        SelectEntry::Separator,
    ];
    entries.extend(custom.map(SelectEntry::Option));
    entries
}

/// The compact demo select: the four options.
pub fn demo_flat() -> Vec<SelectEntry> {
    frequency_options()
        .into_iter()
        .map(SelectEntry::Option)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hl(entries: &[SelectEntry], nav: &SelectNav) -> String {
        options(entries)[nav.highlighted().unwrap()].label.clone()
    }

    #[test]
    fn matches_the_live_keyboard_walk() {
        let e = demo_grouped();
        let mut nav = SelectNav::opened(&e, Some("weekly"), true);
        assert_eq!(hl(&e, &nav), "Weekly");
        nav.key(&e, SelectKey::Up);
        nav.key(&e, SelectKey::Up);
        assert_eq!(hl(&e, &nav), "Daily", "arrows stop at the ends");
        nav.key(&e, SelectKey::End);
        assert_eq!(
            hl(&e, &nav),
            "Custom",
            "disabled options take the highlight"
        );
        assert_eq!(nav.key(&e, SelectKey::Enter), SelectCommand::None);
        nav.key(&e, SelectKey::Char('m'));
        assert_eq!(hl(&e, &nav), "Monthly");
        assert_eq!(
            nav.key(&e, SelectKey::Enter),
            SelectCommand::Select("monthly".into())
        );
        assert_eq!(nav.key(&e, SelectKey::Escape), SelectCommand::Close);
    }

    #[test]
    fn pointer_open_without_a_value_highlights_nothing() {
        assert_eq!(
            SelectNav::opened(&demo_flat(), None, false).highlighted(),
            None
        );
        assert_eq!(label_of(&demo_grouped(), "custom"), Some("Custom"));
    }

    #[test]
    fn the_depth_trigger_darkens_while_open_in_light_mode_only() {
        let light = TriggerSkin::new(&Theme::light(), SelectVariant::Default);
        assert_ne!(light.expanded.gradient, light.button.gradient);
        let dark = TriggerSkin::new(&Theme::dark(), SelectVariant::Default);
        assert_eq!(dark.expanded.gradient, dark.button.gradient);
        assert_eq!(SelectSize::Sm.metrics().height, 28.0);
    }
}
