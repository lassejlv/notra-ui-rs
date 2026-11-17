//! The dualtone card (Notra's `InstrumentModule` and friends): a grey shell
//! with an eyebrow title, an optional hint, readout and action, over a lifted white
//! body; plus the flat card, the section heading, the empty state and the staggered
//! reveal. Metrics are the live site's.

use std::time::Duration;

use crate::motion::{Tween, ease};
use crate::theme::{Color, Shadow, Theme};

/// Which surface a module draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum CardVariant {
    /// shadcn's card: one white surface with a hairline ring.
    #[default]
    Flat,
    /// Dualtone with room for a description; the body pads 24 px.
    Panel,
    /// Dualtone with a centred one-line header; the body pads 16 px.
    Table,
    /// [`CardVariant::Table`] with a shorter header.
    TableCompact,
}

impl CardVariant {
    pub const ALL: [CardVariant; 4] = [
        CardVariant::Flat,
        CardVariant::Panel,
        CardVariant::Table,
        CardVariant::TableCompact,
    ];

    pub fn is_dualtone(self) -> bool {
        self != CardVariant::Flat
    }

    /// Table headers centre their one line and use the small title.
    pub fn has_table_header(self) -> bool {
        matches!(self, CardVariant::Table | CardVariant::TableCompact)
    }
}

/// Box metrics of a module, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardMetrics {
    /// Outer radius (`rounded-2xl` dualtone, `rounded-xl` flat).
    pub radius: f32,
    /// Shell border plus the 2 px rim (dualtone) or 0.
    pub shell_padding: f32,
    pub header_padding_top: f32,
    pub header_padding_bottom: f32,
    pub header_padding_x: f32,
    pub header_min_height: f32,
    /// Vertically centre the header's tracks (table headers) or pin them to the top.
    pub header_centered: bool,
    /// Space between the title and description.
    pub header_gap: f32,
    pub title_size: f32,
    pub title_line_height: f32,
    /// How far the body rides up under the header.
    pub body_overlap: f32,
    pub body_padding: f32,
    pub body_padding_x: f32,
    pub body_radius: f32,
    /// Space between header and body (flat and bare-body panels).
    pub gap: f32,
    /// Vertical padding of the whole card (flat only).
    pub padding_y: f32,
}

/// Radius of the body card and of the shell's top inner corner.
pub const BODY_RADIUS: f32 = 14.0;
/// Shell radius.
pub const SHELL_RADIUS: f32 = 16.0;
/// 1 px border plus `p-0.5`.
pub const SHELL_PADDING: f32 = 2.0;
/// The hint's info icon and its gap after the eyebrow.
pub const HINT_ICON: f32 = 14.0;
pub const HINT_GAP: f32 = 6.0;
/// Readout and action sit `gap-2` apart.
pub const ACTION_GAP: f32 = 8.0;
pub const READOUT_SIZE: f32 = 12.0;
pub const READOUT_LINE_HEIGHT: f32 = 16.0;
/// The strings the card renders.
pub const MORE_INFO: &str = "More info";

impl CardVariant {
    /// Metrics, with `bare_body` dropping the shell and the body card.
    pub fn metrics(self, bare_body: bool) -> CardMetrics {
        let dualtone = |pt, pb, min_h, centered, body_padding| CardMetrics {
            radius: SHELL_RADIUS,
            shell_padding: SHELL_PADDING,
            header_padding_top: pt,
            header_padding_bottom: pb,
            header_padding_x: 16.0,
            header_min_height: min_h,
            header_centered: centered,
            header_gap: 4.0,
            title_size: if centered { 14.0 } else { 16.0 },
            title_line_height: if centered { 20.0 } else { 22.0 },
            body_overlap: 20.0,
            body_padding,
            body_padding_x: body_padding,
            body_radius: BODY_RADIUS,
            gap: 0.0,
            padding_y: 0.0,
        };
        let m = match self {
            CardVariant::Flat => CardMetrics {
                radius: 12.0,
                shell_padding: 0.0,
                header_padding_top: 0.0,
                header_padding_bottom: 0.0,
                header_padding_x: 16.0,
                header_min_height: 0.0,
                header_centered: true,
                header_gap: 4.0,
                title_size: 14.0,
                title_line_height: 20.0,
                body_overlap: 0.0,
                body_padding: 0.0,
                body_padding_x: 16.0,
                body_radius: 0.0,
                gap: 16.0,
                padding_y: 16.0,
            },
            CardVariant::Panel => dualtone(16.0, 36.0, 96.0, false, 24.0),
            CardVariant::Table => dualtone(12.0, 32.0, 68.0, true, 16.0),
            CardVariant::TableCompact => dualtone(6.0, 26.0, 56.0, true, 16.0),
        };
        if !bare_body || self == CardVariant::Flat {
            return m;
        }
        CardMetrics {
            shell_padding: 0.0,
            header_padding_top: 0.0,
            header_padding_bottom: 0.0,
            header_padding_x: 4.0,
            header_min_height: 0.0,
            body_overlap: 0.0,
            body_padding: 0.0,
            body_padding_x: 0.0,
            gap: if self.has_table_header() { 0.0 } else { 16.0 },
            ..m
        }
    }
}

/// Resolved paint of a module.
#[derive(Clone, Debug, PartialEq)]
pub struct CardSkin {
    pub shell: Color,
    pub shell_border: Color,
    pub body: Color,
    pub body_border: Color,
    pub body_shadows: Vec<Shadow>,
    pub title: Color,
    pub description: Color,
    pub readout: Color,
    pub hint: Color,
    pub hint_hover: Color,
}

impl CardSkin {
    pub fn new(theme: &Theme, variant: CardVariant) -> Self {
        let text = Self {
            shell: theme.shell,
            shell_border: theme.shell_border,
            body: theme.table_card,
            body_border: theme.table_card_border,
            body_shadows: theme.table_card_shadow.to_vec(),
            title: theme.foreground,
            description: theme.muted_foreground,
            readout: theme.muted_foreground,
            hint: theme.muted_foreground,
            hint_hover: theme.foreground,
        };
        if variant.is_dualtone() {
            return text;
        }
        Self {
            shell: theme.card,
            shell_border: theme.foreground.fade(0.1),
            body: theme.card,
            body_border: Color::TRANSPARENT,
            body_shadows: Vec::new(),
            ..text
        }
    }
}

/// The hint's colour transition (`hover:text-foreground`).
pub const HINT_TRANSITION: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);

/// `InstrumentSection`: a heading over bare content.
pub mod section {
    /// Header and body sit `gap-3` apart.
    pub const GAP: f32 = 12.0;
    /// Narrow layouts stack title and actions `gap-2` apart.
    pub const HEADER_GAP: f32 = 8.0;
    /// The title and description sit `space-y-1` apart.
    pub const TITLE_GAP: f32 = 4.0;
    /// Without a description, a title beside actions sits in a 28 px row.
    pub const ROW_HEIGHT: f32 = 28.0;
    /// Container width (`@min-[32rem]`) at which actions move beside the title.
    pub const WIDE_FROM: f32 = 512.0;

    /// Whether a section `width` px wide puts actions beside the title.
    pub fn is_wide(width: f32) -> bool {
        width >= WIDE_FROM
    }
}

/// `InstrumentEmpty`: the centred placeholder a module shows without data.
pub mod empty {
    pub const MIN_HEIGHT: f32 = 224.0;
    pub const GAP: f32 = 12.0;
    pub const MESSAGE_GAP: f32 = 8.0;
    /// The preview fades in by 24 % and out after 70 % of the height.
    pub const PREVIEW_FADE: [(f32, f32); 4] = [(0.0, 0.0), (0.24, 1.0), (0.70, 1.0), (1.0, 0.0)];
    pub const PREVIEW_OPACITY: f32 = 0.4;
    /// Over a preview, the message sits in a pill.
    pub const PILL_PADDING_X: f32 = 12.0;
    pub const PILL_PADDING_Y: f32 = 6.0;
    pub const PILL_ALPHA: f32 = 0.85;
}

/// `InstrumentReveal`: content rises 6 px and fades in, staggered by order.
pub mod reveal {
    use super::*;

    /// `duration-normal ease-emphasized`.
    pub const TWEEN: Tween = Tween::new(Duration::from_millis(200), ease::OUT);
    pub const STAGGER: Duration = Duration::from_millis(45);
    /// `translate-y-1.5` while hidden.
    pub const TRAVEL: f32 = 6.0;

    /// The tween for the `order`-th item.
    pub fn tween(order: u32) -> Tween {
        TWEEN.delay(STAGGER * order)
    }

    /// Opacity and vertical offset for the `order`-th item, `elapsed` after `active`
    /// last changed (`None`: it never changed, so it rests where it started).
    pub fn frame(order: u32, active: bool, elapsed: Option<Duration>) -> (f32, f32) {
        let shown = match elapsed {
            None => 1.0,
            Some(elapsed) => tween(order).progress(elapsed),
        };
        let p = if active { shown } else { 1.0 - shown };
        (p, TRAVEL * (1.0 - p))
    }
}

/// `InstrumentGrid`: modules in a grid `gap-3` apart.
pub const GRID_GAP: f32 = 12.0;

/// Upper-cases the first letter (CSS `first-letter:uppercase`).
pub fn capitalize_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The card demo: mention rate per engine.
pub mod demo {
    pub const ENGINES: [(&str, &str); 4] = [
        ("ChatGPT", "64%"),
        ("Claude", "58%"),
        ("Gemini", "41%"),
        ("Perplexity", "37%"),
    ];
    /// Each engine row: `py-2.5 text-sm`, divided by a hairline.
    pub const ROW_PADDING_Y: f32 = 10.0;
    pub const HINT: &str = "Share of answers that mention your brand, per engine.";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_header_heights_match_the_site() {
        let m = CardVariant::Table.metrics(false);
        assert_eq!(m.header_min_height, 68.0);
        assert_eq!(m.header_padding_top + 28.0 + m.header_padding_bottom, 72.0);
        assert_eq!(
            CardVariant::TableCompact.metrics(false).header_min_height,
            56.0
        );
    }

    #[test]
    fn bare_body_drops_the_shell() {
        let m = CardVariant::Panel.metrics(true);
        assert_eq!(m.shell_padding, 0.0);
        assert_eq!(m.body_padding, 0.0);
        assert_eq!(m.gap, 16.0);
        assert_eq!(CardVariant::Table.metrics(true).gap, 0.0);
    }

    #[test]
    fn reveal_staggers() {
        let at = Duration::from_millis(60);
        let (first, _) = reveal::frame(0, true, Some(at));
        let (third, offset) = reveal::frame(2, true, Some(at));
        assert!(first > third);
        assert_eq!(offset, reveal::TRAVEL);
        assert_eq!(reveal::frame(0, false, None), (0.0, reveal::TRAVEL));
        assert_eq!(reveal::frame(0, true, None), (1.0, 0.0));
    }

    #[test]
    fn first_letter_is_uppercased() {
        assert_eq!(capitalize_first("collecting answers"), "Collecting answers");
        assert_eq!(capitalize_first(""), "");
    }

    #[test]
    fn flat_skin_uses_the_card_ring() {
        let t = Theme::light();
        let skin = CardSkin::new(&t, CardVariant::Flat);
        assert_eq!(skin.shell, t.card);
        assert!(skin.body_shadows.is_empty());
    }
}
