//! Text entry shared by every chat block: shadcn's `Input`, `Textarea` and `InputGroup`
//! (the base-nova style ui.usenotra.com ships), plus the editing model the GPUI field runs on.

use std::ops::Range;
use std::time::Duration;

use crate::motion::{Tween, ease};
use crate::theme::{Color, Shadow, Theme};

/// Box and type metrics of a text field, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldMetrics {
    /// Fixed height of single-line fields; the minimum height of multi-line ones.
    pub min_height: f32,
    /// Where a multi-line field stops growing and scrolls. `None` grows without bound.
    pub max_height: Option<f32>,
    pub padding_x: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub border: f32,
    pub font_size: f32,
    pub line_height: f32,
}

/// `Input`: `h-8 rounded-lg border px-2.5 py-1 md:text-sm`.
pub const INPUT: FieldMetrics = FieldMetrics {
    min_height: 32.0,
    max_height: Some(32.0),
    padding_x: 10.0,
    padding_y: 4.0,
    radius: 8.0,
    border: 1.0,
    font_size: 14.0,
    line_height: 20.0,
};

/// `Textarea`: `field-sizing-content min-h-16 rounded-lg border px-2.5 py-2 md:text-sm`.
pub const TEXTAREA: FieldMetrics = FieldMetrics {
    min_height: 64.0,
    max_height: None,
    padding_x: 10.0,
    padding_y: 8.0,
    radius: 8.0,
    border: 1.0,
    font_size: 14.0,
    line_height: 20.0,
};

impl FieldMetrics {
    /// The text box inside padding and border for `lines` lines, clamped to the field's bounds.
    pub fn content_height(&self, lines: usize) -> f32 {
        let chrome = 2.0 * (self.padding_y + self.border);
        let natural = lines.max(1) as f32 * self.line_height;
        let min = (self.min_height - chrome).max(self.line_height);
        let max = self.max_height.map_or(f32::INFINITY, |max| max - chrome);
        natural.clamp(min, max.max(min))
    }
}

/// shadcn's field colours for one theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldSkin {
    /// `bg-transparent`, `dark:bg-input/30`.
    pub background: Color,
    pub border: Color,
    pub text: Color,
    pub placeholder: Color,
    pub caret: Color,
    pub selection: Color,
    /// `focus-visible:border-ring`.
    pub focus_border: Color,
    /// `focus-visible:ring-3 ring-ring/50`.
    pub focus_ring: Shadow,
    /// `disabled:bg-input/50`, `dark:disabled:bg-input/80`.
    pub disabled_background: Color,
}

/// `disabled:opacity-50`.
pub const DISABLED_OPACITY: f32 = 0.5;

/// Chrome's default `::selection` highlight; the site leaves it unstyled.
pub const SELECTION: Color = Color::rgba(0x3390ff52);

/// `transition-colors`: border, ring and background fade over 150 ms.
pub const TRANSITION: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);

/// How long the caret stays lit, then dark, while a field is focused.
pub const CARET_BLINK: Duration = Duration::from_millis(500);

/// The caret's width, as browsers draw it.
pub const CARET_WIDTH: f32 = 1.0;

impl FieldSkin {
    pub fn new(theme: &Theme) -> Self {
        let dark = theme.is_dark();
        Self {
            background: if dark {
                theme.input.fade(0.3)
            } else {
                Color::TRANSPARENT
            },
            border: theme.input,
            text: theme.foreground,
            placeholder: theme.muted_foreground,
            caret: theme.foreground,
            selection: SELECTION,
            focus_border: theme.ring,
            focus_ring: Shadow::drop(0.0, 0.0, 3.0, theme.ring.fade(0.5)),
            disabled_background: theme.input.fade(if dark { 0.8 } else { 0.5 }),
        }
    }

    /// The same field without its own box: the control inside an `InputGroup` or a composer.
    pub fn bare(theme: &Theme) -> Self {
        Self {
            background: Color::TRANSPARENT,
            border: Color::TRANSPARENT,
            focus_border: Color::TRANSPARENT,
            focus_ring: Shadow::drop(0.0, 0.0, 0.0, Color::TRANSPARENT),
            disabled_background: Color::TRANSPARENT,
            ..Self::new(theme)
        }
    }
}

/// Where an `InputGroupAddon` sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AddonAlign {
    #[default]
    InlineStart,
    InlineEnd,
    BlockStart,
    BlockEnd,
}

/// `InputGroup` and its parts.
pub mod group {
    /// The group box: `h-8 rounded-lg border` (`h-auto` around a textarea or block addon).
    pub const HEIGHT: f32 = 32.0;
    pub const RADIUS: f32 = 8.0;
    /// Inline addons: `pl-2`/`pr-2`, `gap-2`, `text-sm font-medium`, icons `size-4`.
    pub const ADDON_INSET: f32 = 8.0;
    pub const ADDON_GAP: f32 = 8.0;
    pub const ADDON_ICON: f32 = 16.0;
    /// Block addons: `px-2.5 pt-2` / `pb-2`.
    pub const BLOCK_ADDON_PADDING_X: f32 = 10.0;
    pub const BLOCK_ADDON_PADDING_Y: f32 = 8.0;
    /// The input's padding on the side an inline addon takes: `pl-1.5`/`pr-1.5`.
    pub const INPUT_INSET_BESIDE_ADDON: f32 = 6.0;
    /// `InputGroupButton size="xs"`: `h-6 gap-1 px-1.5`, `rounded-[calc(var(--radius)-3px)]`.
    pub const BUTTON_HEIGHT: f32 = 24.0;
    pub const BUTTON_PADDING_X: f32 = 6.0;
    pub const BUTTON_RADIUS: f32 = 5.0;
    pub const BUTTON_ICON: f32 = 14.0;
    /// An addon holding a button pulls 0.3rem into the group's edge.
    pub const BUTTON_PULL: f32 = 4.8;
}

/// What a key press does to the caret, in platform-neutral terms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    WordLeft,
    WordRight,
    LineStart,
    LineEnd,
    Start,
    End,
}

/// A text buffer with a selection and an IME composition range, in UTF-8 byte offsets.
///
/// Boundaries are Unicode scalar values; the field keeps no undo history.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Editing {
    text: String,
    selection: Range<usize>,
    reversed: bool,
    marked: Option<Range<usize>>,
}

impl Editing {
    /// A buffer holding `text` with the caret at its end.
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let end = text.len();
        Self {
            text,
            selection: end..end,
            reversed: false,
            marked: None,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// The selected byte range, start before end.
    pub fn selection(&self) -> Range<usize> {
        self.selection.clone()
    }

    /// Whether the caret sits at the selection's start.
    pub fn is_reversed(&self) -> bool {
        self.reversed
    }

    /// The IME composition, if one is in progress.
    pub fn marked(&self) -> Option<Range<usize>> {
        self.marked.clone()
    }

    /// The caret: the selection end that moves.
    pub fn cursor(&self) -> usize {
        if self.reversed {
            self.selection.start
        } else {
            self.selection.end
        }
    }

    pub fn selected_text(&self) -> &str {
        &self.text[self.selection.clone()]
    }

    /// Replaces the whole text and puts the caret at its end.
    pub fn set_text(&mut self, text: impl Into<String>) {
        *self = Self::new(text);
    }

    /// Places the caret at `offset`, collapsing the selection.
    pub fn move_to(&mut self, offset: usize) {
        let offset = self.clamp(offset);
        self.selection = offset..offset;
        self.reversed = false;
    }

    /// Moves the caret to `offset`, keeping the anchor.
    pub fn select_to(&mut self, offset: usize) {
        let offset = self.clamp(offset);
        if self.reversed {
            self.selection.start = offset;
        } else {
            self.selection.end = offset;
        }
        if self.selection.end < self.selection.start {
            self.reversed = !self.reversed;
            self.selection = self.selection.end..self.selection.start;
        }
    }

    pub fn select_all(&mut self) {
        self.selection = 0..self.text.len();
        self.reversed = false;
    }

    /// Sets the selection directly (IME and accessibility requests).
    pub fn set_selection(&mut self, range: Range<usize>, reversed: bool) {
        let (a, b) = (self.clamp(range.start), self.clamp(range.end));
        self.selection = a.min(b)..a.max(b);
        self.reversed = reversed;
    }

    /// Selects the word under `offset` (a double click), or the character there between words.
    pub fn select_word_at(&mut self, offset: usize) {
        let offset = self.clamp(offset);
        let is_word = |c: char| c.is_alphanumeric() || c == '_';
        let start = self.text[..offset]
            .char_indices()
            .rev()
            .take_while(|&(_, c)| is_word(c))
            .last()
            .map_or(offset, |(i, _)| i);
        let end = offset
            + self.text[offset..]
                .char_indices()
                .find(|&(_, c)| !is_word(c))
                .map_or(self.text.len() - offset, |(i, _)| i);
        let end = if start == end {
            self.next_boundary(end)
        } else {
            end
        };
        self.selection = start..end;
        self.reversed = false;
    }

    /// The offset a motion lands on from the caret.
    pub fn target(&self, motion: Motion) -> usize {
        let at = self.cursor();
        match motion {
            Motion::Left => self.previous_boundary(at),
            Motion::Right => self.next_boundary(at),
            Motion::WordLeft => self.previous_word(at),
            Motion::WordRight => self.next_word(at),
            Motion::LineStart => self.text[..at].rfind('\n').map_or(0, |i| i + 1),
            Motion::LineEnd => self.text[at..]
                .find('\n')
                .map_or(self.text.len(), |i| at + i),
            Motion::Start => 0,
            Motion::End => self.text.len(),
        }
    }

    /// Moves the caret, or extends the selection with `extend`. A plain left or right over a
    /// selection lands on its edge, as browsers do.
    pub fn apply(&mut self, motion: Motion, extend: bool) {
        if extend {
            self.select_to(self.target(motion));
            return;
        }
        let target = match motion {
            Motion::Left if !self.selection.is_empty() => self.selection.start,
            Motion::Right if !self.selection.is_empty() => self.selection.end,
            _ => self.target(motion),
        };
        self.move_to(target);
    }

    /// Types `text` over the composition, or else the selection.
    pub fn insert(&mut self, text: &str) {
        let range = self.marked.take().unwrap_or(self.selection.clone());
        self.replace(range, text);
    }

    /// Replaces `range` with `text` and leaves the caret after it.
    pub fn replace(&mut self, range: Range<usize>, text: &str) {
        let range = self.clamp(range.start)..self.clamp(range.end);
        self.text.replace_range(range.clone(), text);
        let caret = range.start + text.len();
        self.selection = caret..caret;
        self.reversed = false;
        self.marked = None;
    }

    /// Replaces `range` (or the composition, or the selection) with composing `text`, selecting
    /// `selected` inside it (IME preedit).
    pub fn replace_and_mark(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
    ) {
        let range = range
            .or_else(|| self.marked.clone())
            .unwrap_or(self.selection.clone());
        let range = self.clamp(range.start)..self.clamp(range.end);
        self.text.replace_range(range.clone(), text);
        self.marked = (!text.is_empty()).then(|| range.start..range.start + text.len());
        let selected = selected.map_or(text.len()..text.len(), |r| {
            r.start.min(text.len())..r.end.min(text.len())
        });
        self.selection = range.start + selected.start..range.start + selected.end;
        self.reversed = false;
    }

    /// Ends a composition, keeping its text.
    pub fn unmark(&mut self) {
        self.marked = None;
    }

    /// Deletes the selection, or the character before the caret. Returns whether anything went.
    pub fn backspace(&mut self) -> bool {
        self.delete_towards(Motion::Left)
    }

    /// Deletes the selection, or the character after the caret.
    pub fn delete(&mut self) -> bool {
        self.delete_towards(Motion::Right)
    }

    /// Deletes the selection, or from the caret to where `motion` lands.
    pub fn delete_towards(&mut self, motion: Motion) -> bool {
        if self.selection.is_empty() {
            let target = self.target(motion);
            if target == self.cursor() {
                return false;
            }
            self.select_to(target);
        }
        self.replace(self.selection.clone(), "");
        true
    }

    /// Byte offset of a UTF-16 offset (platform input methods speak UTF-16).
    pub fn offset_from_utf16(&self, utf16: usize) -> usize {
        let mut units = 0;
        for (i, ch) in self.text.char_indices() {
            if units >= utf16 {
                return i;
            }
            units += ch.len_utf16();
        }
        self.text.len()
    }

    /// UTF-16 offset of a byte offset.
    pub fn offset_to_utf16(&self, offset: usize) -> usize {
        self.text[..self.clamp(offset)].encode_utf16().count()
    }

    pub fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }

    pub fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn clamp(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.text.len());
        while !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.text[..offset]
            .char_indices()
            .next_back()
            .map_or(0, |(i, _)| i)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.text[offset..]
            .chars()
            .next()
            .map_or(self.text.len(), |c| offset + c.len_utf8())
    }

    fn previous_word(&self, offset: usize) -> usize {
        let trimmed = self.text[..offset].trim_end_matches(|c: char| !c.is_alphanumeric());
        trimmed
            .char_indices()
            .rev()
            .find(|(_, c)| !c.is_alphanumeric())
            .map_or(0, |(i, c)| i + c.len_utf8())
    }

    fn next_word(&self, offset: usize) -> usize {
        let tail = &self.text[offset..];
        let skipped = tail.len()
            - tail
                .trim_start_matches(|c: char| !c.is_alphanumeric())
                .len();
        let word = &tail[skipped..];
        offset
            + skipped
            + word
                .find(|c: char| !c.is_alphanumeric())
                .unwrap_or(word.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_replaces_the_selection() {
        let mut e = Editing::new("hello world");
        e.move_to(6);
        e.select_to(11);
        e.insert("there");
        assert_eq!(e.text(), "hello there");
        assert_eq!(e.selection(), 11..11);
    }

    #[test]
    fn selecting_backwards_flips_the_anchor() {
        let mut e = Editing::new("abcdef");
        e.move_to(3);
        e.select_to(1);
        assert_eq!(e.selection(), 1..3);
        assert!(e.is_reversed());
        assert_eq!(e.cursor(), 1);
    }

    #[test]
    fn backspace_removes_one_scalar() {
        let mut e = Editing::new("añ");
        assert!(e.backspace());
        assert_eq!(e.text(), "a");
        e.move_to(0);
        assert!(!e.backspace());
    }

    #[test]
    fn arrows_collapse_a_selection_to_its_edge() {
        let mut e = Editing::new("abcdef");
        e.move_to(1);
        e.select_to(4);
        e.apply(Motion::Left, false);
        assert_eq!(e.selection(), 1..1);
    }

    #[test]
    fn word_motions_skip_punctuation() {
        let mut e = Editing::new("one, two three");
        e.move_to(14);
        assert_eq!(e.target(Motion::WordLeft), 9);
        e.move_to(3);
        assert_eq!(e.target(Motion::WordRight), 8);
    }

    #[test]
    fn double_click_selects_the_word() {
        let mut e = Editing::new("say hello_there now");
        e.select_word_at(6);
        assert_eq!(e.selected_text(), "hello_there");
        let mut gap = Editing::new("a  b");
        gap.select_word_at(2);
        assert_eq!(gap.selected_text(), " ");
    }

    #[test]
    fn line_motions_stop_at_newlines() {
        let mut e = Editing::new("first\nsecond");
        e.move_to(8);
        assert_eq!(e.target(Motion::LineStart), 6);
        e.move_to(2);
        assert_eq!(e.target(Motion::LineEnd), 5);
    }

    #[test]
    fn composition_marks_and_commits() {
        let mut e = Editing::new("a");
        e.replace_and_mark(None, "ni", None);
        assert_eq!(e.marked(), Some(1..3));
        e.insert("に");
        assert_eq!(e.text(), "aに");
        assert_eq!(e.marked(), None);
    }

    #[test]
    fn utf16_offsets_round_trip() {
        let e = Editing::new("a😀b");
        assert_eq!(e.offset_to_utf16(5), 3);
        assert_eq!(e.offset_from_utf16(3), 5);
    }

    #[test]
    fn textarea_grows_between_its_bounds() {
        let m = FieldMetrics {
            max_height: Some(120.0),
            ..TEXTAREA
        };
        assert_eq!(m.content_height(1), 46.0);
        assert_eq!(m.content_height(4), 80.0);
        assert_eq!(m.content_height(20), 102.0);
    }
}
