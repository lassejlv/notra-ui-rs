//! Notra's tables: the `notra-table` primitives (a grey shell with a 2 px rim around
//! a lifted body) and the data table (sorting, selection, resizing, collapsing
//! columns, paging, virtual rows, the corner scrollbar, empty and loading states).
//!
//! This module holds everything both renderers share: column sizing (CSS
//! `table-layout: fixed` with the header floors the site computes), sort and
//! selection state, paging and its footer text, the viewport and virtual window,
//! the loading overlay and the corner scrollbar's geometry.

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::ops::Range;
use std::time::Duration;

use crate::components::data_parts::checkbox::CheckState;
use crate::motion::{Tween, ease};
use crate::theme::{Color, Theme};

/// Horizontal alignment of a column's header and cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// A column's width, as the React `width` prop takes CSS lengths.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ColumnWidth {
    /// Share the remaining space equally.
    #[default]
    Auto,
    Px(f32),
    /// `"7.5rem"`.
    Rem(f32),
    /// `"2fr"`: a weighted share of the remaining space.
    Fr(f32),
    /// `"20%"` of the table.
    Percent(f32),
}

impl ColumnWidth {
    fn flexible(self) -> bool {
        matches!(self, ColumnWidth::Auto | ColumnWidth::Fr(_))
    }
}

/// One column of a data table.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Column {
    pub key: String,
    pub header: String,
    /// Explains the column on an info icon beside its header.
    pub hint: Option<String>,
    pub sortable: bool,
    pub align: Align,
    pub width: ColumnWidth,
    /// Floor in px; defaults to the header label plus its chrome.
    pub min_width: Option<f32>,
    /// Columns with a priority drop out (highest first) when the table is too narrow.
    pub collapse_priority: Option<u32>,
}

impl Column {
    pub fn new(key: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            header: header.into(),
            ..Self::default()
        }
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn width(mut self, width: ColumnWidth) -> Self {
        self.width = width;
        self
    }

    pub fn min_width(mut self, px: f32) -> Self {
        self.min_width = Some(px);
        self
    }

    pub fn collapse_priority(mut self, priority: u32) -> Self {
        self.collapse_priority = Some(priority);
        self
    }

    /// The header floor (`headerMinWidth`): the label in `ch` plus padding, sort
    /// arrow and hint icon, never under `min_column_width`. `ch` is the width of
    /// "0" in the header font.
    pub fn floor(&self, ch: f32, min_column_width: f32) -> f32 {
        if let Some(min) = self.min_width {
            return min;
        }
        if self.header.is_empty() {
            return min_column_width;
        }
        let chrome = HEADER_PAD_X
            + if self.sortable { SORT_ICON_SPACE } else { 0.0 }
            + if self.hint.is_some() {
                HINT_ICON_SPACE
            } else {
                0.0
            };
        let label = (self.header.chars().count() as f32 + HEADER_CH_BUFFER) * ch;
        (label + chrome).max(min_column_width)
    }
}

/// Header labels' horizontal padding (`px-4` on both sides).
pub const HEADER_PAD_X: f32 = 32.0;
/// Sort arrow (14 px) plus its gap.
pub const SORT_ICON_SPACE: f32 = 18.0;
/// Hint icon plus its gap.
pub const HINT_ICON_SPACE: f32 = 18.0;
pub const HEADER_CH_BUFFER: f32 = 1.0;
pub const DEFAULT_MIN_COLUMN_WIDTH: f32 = 64.0;
pub const CHECKBOX_COLUMN_WIDTH: f32 = 48.0;
/// Shell border, rim and body border on both sides.
pub const FRAME_INSET: f32 = 8.0;
pub const ROW_HEIGHT: f32 = 48.0;
pub const HEADER_HEIGHT: f32 = 36.0;
/// The primitives' header (`h-10`).
pub const PRIMITIVE_HEADER_HEIGHT: f32 = 40.0;
pub const DEFAULT_HEIGHT: f32 = 440.0;
pub const OVERSCAN: usize = 10;
pub const SKELETON_ROWS: usize = 3;
/// Rows from the end at which an infinite table asks for more.
pub const END_THRESHOLD_ROWS: f32 = 4.0;
pub const CELL_PADDING_X: f32 = 16.0;
pub const SORT_ICON: f32 = 14.0;
pub const HINT_ICON: f32 = 13.0;
pub const HINT_OPACITY: f32 = 0.6;
pub const ICON_GAP: f32 = 4.0;
pub const SORT_IDLE_OPACITY: f32 = 0.35;
pub const RESIZE_HANDLE_WIDTH: f32 = 6.0;
pub const SHELL_RADIUS: f32 = 16.0;
pub const BODY_RADIUS: f32 = 14.0;
pub const RIM: f32 = 2.0;
/// The header band leaves 20 px under it that the body rides over.
pub const BODY_OVERLAP: f32 = 20.0;
/// Rows dim to this while a table that has rows reloads.
pub const DIM_OPACITY: f32 = 0.6;
/// The scroll fades' height.
pub const FADE_HEIGHT: f32 = 32.0;
pub const PAGE_SIZE_OPTIONS: [usize; 5] = [10, 20, 30, 40, 50];
/// The footer: `min-h-11 px-4 py-1.5 text-xs`.
pub const FOOTER_MIN_HEIGHT: f32 = 44.0;
pub const FOOTER_PADDING_X: f32 = 16.0;
pub const FOOTER_GAP: f32 = 12.0;
pub const PAGER_BUTTON_GAP: f32 = 4.0;
/// The page-size trigger pulls 10 px left so its label lines up with the padding.
pub const PAGE_SIZE_PULL: f32 = 10.0;

/// Row and sort-arrow motion.
pub mod motion {
    use super::*;

    /// Row hover (`transition-colors`).
    pub const ROW_HOVER: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);
    /// The sort arrow turns and brightens: `duration-fast ease-out`.
    pub const SORT: Tween = Tween::new(Duration::from_millis(150), ease::OUT);
    /// Scroll fades and the corner morph: `duration-200 ease-out`.
    pub const EDGE: Tween = Tween::new(Duration::from_millis(200), ease::CSS_EASE_OUT);
    /// Dimming while loading.
    pub const DIM: Tween = Tween::new(Duration::from_millis(200), ease::IN_OUT);
}

/// Sort direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Which column sorts the rows, and which way.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sort {
    pub key: String,
    pub direction: SortDirection,
}

impl Sort {
    /// A header click: ascending, then descending, then unsorted.
    pub fn toggle(current: Option<&Sort>, key: &str) -> Option<Sort> {
        match current {
            Some(s) if s.key == key && s.direction == SortDirection::Asc => Some(Sort {
                key: key.to_owned(),
                direction: SortDirection::Desc,
            }),
            Some(s) if s.key == key => None,
            _ => Some(Sort {
                key: key.to_owned(),
                direction: SortDirection::Asc,
            }),
        }
    }
}

/// A cell's sort value.
#[derive(Clone, Debug, PartialEq)]
pub enum SortValue {
    Number(f64),
    Text(String),
}

impl SortValue {
    fn compare(&self, other: &SortValue) -> Ordering {
        match (self, other) {
            (SortValue::Number(a), SortValue::Number(b)) => a.total_cmp(b),
            (a, b) => a.text().to_lowercase().cmp(&b.text().to_lowercase()),
        }
    }

    fn text(&self) -> String {
        match self {
            SortValue::Number(n) => n.to_string(),
            SortValue::Text(t) => t.clone(),
        }
    }
}

/// Row indices in display order: sorted by `sort` (stable), with pinned rows first.
pub fn display_order(
    count: usize,
    sort: Option<&Sort>,
    value: impl Fn(usize, &str) -> SortValue,
    pinned: impl Fn(usize) -> bool,
) -> Vec<usize> {
    let mut order: Vec<usize> = (0..count).collect();
    if let Some(sort) = sort {
        let values: Vec<SortValue> = order.iter().map(|&i| value(i, &sort.key)).collect();
        order.sort_by(|&a, &b| {
            let cmp = values[a].compare(&values[b]);
            match sort.direction {
                SortDirection::Asc => cmp,
                SortDirection::Desc => cmp.reverse(),
            }
        });
    }
    let (first, rest): (Vec<usize>, Vec<usize>) = order.into_iter().partition(|&i| pinned(i));
    first.into_iter().chain(rest).collect()
}

/// Row selection by stable id.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Selection {
    ids: BTreeSet<String>,
}

impl Selection {
    pub fn contains(&self, id: &str) -> bool {
        self.ids.contains(id)
    }

    pub fn toggle(&mut self, id: &str) {
        if !self.ids.remove(id) {
            self.ids.insert(id.to_owned());
        }
    }

    /// The header checkbox for the rows `ids`.
    pub fn state<'a>(&self, ids: impl IntoIterator<Item = &'a str>) -> CheckState {
        let (mut any, mut all, mut seen) = (false, true, false);
        for id in ids {
            seen = true;
            if self.ids.contains(id) {
                any = true;
            } else {
                all = false;
            }
        }
        CheckState::new(seen && all, any)
    }

    /// Select every row in `ids`, or clear them when all are selected.
    pub fn toggle_all<'a>(&mut self, ids: impl IntoIterator<Item = &'a str> + Clone) {
        if self.state(ids.clone()) == CheckState::On {
            for id in ids {
                self.ids.remove(id);
            }
        } else {
            self.ids.extend(ids.into_iter().map(str::to_owned));
        }
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.ids.iter().map(String::as_str)
    }
}

/// Pixel widths for one render of a table.
#[derive(Clone, Debug, PartialEq)]
pub struct ColumnLayout {
    /// Visible columns, as indices into the column list.
    pub visible: Vec<usize>,
    /// Width of each visible column.
    pub widths: Vec<f32>,
    /// The checkbox column (0 when not selectable).
    pub checkbox: f32,
    /// The trailing filler column that takes leftover space.
    pub filler: f32,
}

impl ColumnLayout {
    pub fn total(&self) -> f32 {
        self.checkbox + self.widths.iter().sum::<f32>() + self.filler
    }
}

/// Sizing inputs shared by every layout pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sizing {
    /// Width of "0" in the header font.
    pub ch: f32,
    pub min_column_width: f32,
    pub selectable: bool,
}

fn floor_px(c: &Column, s: &Sizing, table: f32) -> f32 {
    let floor = c.floor(s.ch, s.min_column_width);
    match c.width {
        ColumnWidth::Px(w) => w.max(floor),
        ColumnWidth::Rem(r) => (r * crate::theme::REM).max(floor),
        ColumnWidth::Percent(p) => (table * p / 100.0).max(floor),
        ColumnWidth::Auto | ColumnWidth::Fr(_) => floor,
    }
}

/// Lays out `columns` in a table `width` px wide (inside the frame), with resized
/// columns pinned to `overrides` and low-priority columns dropped while the floors
/// don't fit, as the site's `useCollapsibleColumns` and fixed table layout do.
pub fn layout_columns(
    columns: &[Column],
    width: f32,
    overrides: &dyn Fn(&str) -> Option<f32>,
    sizing: Sizing,
) -> ColumnLayout {
    let checkbox = if sizing.selectable {
        CHECKBOX_COLUMN_WIDTH
    } else {
        0.0
    };
    let mut collapse: Vec<usize> = (0..columns.len())
        .filter(|&i| columns[i].collapse_priority.is_some())
        .collect();
    collapse.sort_by_key(|&i| std::cmp::Reverse(columns[i].collapse_priority));
    let floor_of = |i: usize| {
        overrides(&columns[i].key).unwrap_or_else(|| floor_px(&columns[i], &sizing, width))
    };
    let mut hidden = 0;
    while hidden < collapse.len() {
        let floors: f32 = (0..columns.len())
            .filter(|i| !collapse[..hidden].contains(i))
            .map(floor_of)
            .sum();
        if checkbox + floors <= width {
            break;
        }
        hidden += 1;
    }
    let visible: Vec<usize> = (0..columns.len())
        .filter(|i| !collapse[..hidden].contains(i))
        .collect();
    let flexible = |i: usize| overrides(&columns[i].key).is_none() && columns[i].width.flexible();
    let fixed: f32 = visible
        .iter()
        .filter(|&&i| !flexible(i))
        .map(|&i| floor_of(i))
        .sum();
    let any_fr = visible
        .iter()
        .any(|&i| matches!(columns[i].width, ColumnWidth::Fr(_)));
    let weight = |i: usize| match columns[i].width {
        ColumnWidth::Fr(f) => f,
        _ if any_fr || flexible(i) => 1.0,
        _ => 0.0,
    };
    let flex: Vec<usize> = visible.iter().copied().filter(|&i| flexible(i)).collect();
    let mut remaining = (width - checkbox - fixed).max(0.0);
    let mut share: Vec<Option<f32>> = vec![None; columns.len()];
    let mut open: Vec<usize> = flex.clone();
    loop {
        let total: f32 = open.iter().map(|&i| weight(i)).sum();
        let pinned: Vec<usize> = open
            .iter()
            .copied()
            .filter(|&i| total > 0.0 && remaining * weight(i) / total < floor_of(i))
            .collect();
        if pinned.is_empty() {
            for &i in &open {
                share[i] = Some(if total > 0.0 {
                    remaining * weight(i) / total
                } else {
                    floor_of(i)
                });
            }
            break;
        }
        for i in pinned {
            share[i] = Some(floor_of(i));
            remaining = (remaining - floor_of(i)).max(0.0);
            open.retain(|&o| o != i);
        }
    }
    let widths: Vec<f32> = visible
        .iter()
        .map(|&i| share[i].unwrap_or_else(|| floor_of(i)))
        .collect();
    let used = checkbox + widths.iter().sum::<f32>();
    ColumnLayout {
        visible,
        widths,
        checkbox,
        filler: (width - used).max(0.0),
    }
}

/// How rows load.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadingState {
    /// Rows already shown dim in place.
    Dimmed,
    /// Skeleton rows follow the loaded rows (an infinite table fetching its next page).
    More,
    /// An empty table fills with skeleton rows.
    Skeleton,
}

/// `tableLoadingOverlay`: what loading looks like for `row_count` rows.
pub fn loading_state(
    loading: bool,
    row_count: usize,
    loading_more: Option<bool>,
    infinite: bool,
) -> Option<LoadingState> {
    let has_rows = row_count > 0;
    let more = loading && has_rows && loading_more.unwrap_or(infinite);
    match (loading, has_rows, more) {
        (true, true, false) => Some(LoadingState::Dimmed),
        (true, _, true) => Some(LoadingState::More),
        (true, false, _) => Some(LoadingState::Skeleton),
        _ => None,
    }
}

/// `getTableViewportLayout` for fixed rows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    /// The tallest body that fits, in whole rows.
    pub body_height: f32,
    /// The body scrolls (more rows than fit).
    pub scrolls: bool,
    /// The body's drawn height.
    pub height: f32,
}

impl Viewport {
    pub fn new(
        row_count: usize,
        row_height: f32,
        header_height: f32,
        height: f32,
        min_height: Option<f32>,
    ) -> Self {
        let resolved = height.max(min_height.unwrap_or(0.0));
        let rows_fit =
            |h: f32| ((h - header_height).max(row_height) / row_height).floor() * row_height;
        let body_height = rows_fit(resolved);
        let min_body = min_height.map_or(0.0, rows_fit);
        let content = (row_count as f32 * row_height)
            .min(body_height)
            .max(row_height);
        let scrolls = row_count as f32 * row_height > body_height;
        let height = if scrolls {
            body_height
        } else if row_count == 0 {
            body_height.max(min_body)
        } else {
            content.max(min_body)
        };
        Self {
            body_height,
            scrolls,
            height,
        }
    }

    /// Rows to mount at `scroll_top`, with `overscan` rows either side.
    pub fn window(
        &self,
        scroll_top: f32,
        row_height: f32,
        count: usize,
        overscan: usize,
    ) -> Range<usize> {
        if !self.scrolls || row_height <= 0.0 {
            return 0..count;
        }
        let first = (scroll_top / row_height).floor().max(0.0) as usize;
        let last = ((scroll_top + self.height) / row_height).ceil() as usize;
        first.saturating_sub(overscan)..(last + overscan).min(count)
    }
}

/// An infinite table asks for more once fewer than four rows' worth remain below.
pub fn near_end(scroll_top: f32, scroll_height: f32, client_height: f32, row_height: f32) -> bool {
    scroll_height - scroll_top - client_height < row_height * END_THRESHOLD_ROWS
}

/// The corner scrollbar: a straight thumb along the body's right edge that tightens
/// the card's corner when it runs into it.
pub mod corner {
    use super::*;

    pub const GAP: f32 = 3.0;
    pub const WIDTH: f32 = 4.0;
    pub const MIN_THUMB: f32 = 28.0;
    pub const IDLE: Duration = Duration::from_millis(900);
    /// The card's tightened corner, concentric with the thumb's round end.
    pub const CARD_RADIUS: f32 = GAP + WIDTH / 2.0;
    /// The shell's, one shell inset further out.
    pub const SHELL_RADIUS: f32 = CARD_RADIUS + 3.0;
    /// Thumb fill (foreground at 22 %), hovered (35 %) and pressed (45 %).
    pub const ALPHAS: [f32; 3] = [0.22, 0.35, 0.45];

    /// Where the thumb sits and which corners it reaches.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Thumb {
        pub top: f32,
        pub length: f32,
        /// The scrollable distance and the thumb's travel, for drags.
        pub scrollable: f32,
        pub travel: f32,
        pub at_top_corner: bool,
        pub at_bottom_corner: bool,
    }

    /// `None` when nothing scrolls.
    pub fn thumb(scroll_top: f32, content: f32, viewport: f32) -> Option<Thumb> {
        let scrollable = (content - viewport).max(0.0);
        if scrollable <= 1.0 {
            return None;
        }
        let track = viewport - GAP * 2.0;
        let length = track.min(MIN_THUMB.max(viewport / content * track));
        let travel = track - length;
        let progress = scroll_top.clamp(0.0, scrollable) / scrollable;
        let top = GAP + progress * travel;
        Some(Thumb {
            top,
            length,
            scrollable,
            travel,
            at_top_corner: top < BODY_RADIUS,
            at_bottom_corner: top + length > viewport - BODY_RADIUS,
        })
    }

    /// The scroll offset after dragging the thumb `dy` px from `start`.
    pub fn drag(thumb: &Thumb, start: f32, dy: f32) -> f32 {
        if thumb.travel <= 0.0 {
            return start;
        }
        (start + dy / thumb.travel * thumb.scrollable).clamp(0.0, thumb.scrollable)
    }
}

/// How the table pages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PagingMode {
    /// Every row is loaded; the table slices them.
    Client,
    /// The rows are the current page; the server knows the total.
    Server { total: usize },
    /// The rows are the current page; only whether another follows is known.
    Cursor { has_next: bool },
}

/// Paging inputs (React's `pagination` prop).
#[derive(Clone, Debug, PartialEq)]
pub struct Paging {
    /// 1-based.
    pub page: usize,
    pub page_size: usize,
    pub mode: PagingMode,
    pub item_label: Option<String>,
    /// Offer the page-size select.
    pub size_select: bool,
    pub size_options: Vec<usize>,
}

impl Paging {
    pub fn client(page: usize, page_size: usize) -> Self {
        Self {
            page,
            page_size,
            mode: PagingMode::Client,
            item_label: None,
            size_select: true,
            size_options: PAGE_SIZE_OPTIONS.to_vec(),
        }
    }

    pub fn mode(mut self, mode: PagingMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn item_label(mut self, label: impl Into<String>) -> Self {
        self.item_label = Some(label.into());
        self
    }

    pub fn size_select(mut self, on: bool) -> Self {
        self.size_select = on;
        self
    }

    /// Rows of `rows` (all loaded rows for client paging) on the current page.
    pub fn slice(&self, rows: usize) -> Range<usize> {
        match self.mode {
            PagingMode::Client => {
                let page = self.page.clamp(1, self.page_count(rows).unwrap_or(1));
                let start = (page - 1) * self.page_size;
                start.min(rows)..(start + self.page_size).min(rows)
            }
            _ => 0..rows,
        }
    }

    fn page_count(&self, rows: usize) -> Option<usize> {
        let total = match self.mode {
            PagingMode::Client => rows,
            PagingMode::Server { total } => total,
            PagingMode::Cursor { .. } => return None,
        };
        Some(total.div_ceil(self.page_size.max(1)).max(1))
    }

    /// The footer for `rows` loaded rows, or `None` when there is nothing to page.
    pub fn pager(&self, rows: usize, labels: &Labels) -> Option<Pager> {
        let size = self.page_size.max(1);
        let (page, page_count, page_rows, total) = match self.mode {
            PagingMode::Cursor { .. } => {
                if rows == 0 && self.page <= 1 {
                    return None;
                }
                (self.page.max(1), None, rows, None)
            }
            _ => {
                let total = match self.mode {
                    PagingMode::Server { total } => total,
                    _ => rows,
                };
                if total == 0 {
                    return None;
                }
                let count = total.div_ceil(size).max(1);
                let page = self.page.clamp(1, count);
                (
                    page,
                    Some(count),
                    size.min(total - (page - 1) * size),
                    Some(total),
                )
            }
        };
        let offset = (page - 1) * size;
        let start = if page_rows == 0 || total == Some(0) {
            0
        } else {
            offset + 1
        };
        let end = total.map_or(offset + page_rows, |t| t.min(page * size));
        let range = match total {
            Some(t) => (labels.range)(&group(start), &group(end), &group(t)),
            None => (labels.range_open)(&group(start), &group(end)),
        };
        let range = match &self.item_label {
            Some(label) => format!("{range} {label}"),
            None => range,
        };
        let is_first = page <= 1;
        let is_last = match (page_count, &self.mode) {
            (Some(count), _) => page >= count,
            (None, PagingMode::Cursor { has_next }) => !has_next,
            (None, _) => true,
        };
        let smallest = self
            .size_options
            .iter()
            .copied()
            .chain([size])
            .min()
            .unwrap_or(size);
        let more_than_smallest = match total {
            Some(t) => t > smallest,
            None => !(is_first && is_last) || page_rows > smallest,
        };
        let mut sizes: Vec<usize> = self.size_options.iter().copied().chain([size]).collect();
        sizes.sort_unstable();
        sizes.dedup();
        Some(Pager {
            page,
            page_size: size,
            page_text: match page_count {
                Some(count) => (labels.page_of)(&group(page), &group(count)),
                None => (labels.page_number)(&group(page)),
            },
            range,
            size_text: (labels.show_rows)(&group(size)),
            show_size_select: self.size_select && more_than_smallest,
            sizes,
            has_pages: !(is_first && is_last),
            prev: (!is_first).then(|| page - 1),
            next: (!is_last).then(|| page + 1),
        })
    }
}

/// The footer pager's content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pager {
    pub page: usize,
    pub page_size: usize,
    /// "Page 1 of 4".
    pub page_text: String,
    /// "1-10 of 36 prompts", shown when there is no page-size select.
    pub range: String,
    /// "Show 10".
    pub size_text: String,
    pub show_size_select: bool,
    pub sizes: Vec<usize>,
    /// Hide the page controls on a single page.
    pub has_pages: bool,
    pub prev: Option<usize>,
    pub next: Option<usize>,
}

/// Every string the table renders; override fields to translate.
#[derive(Clone, Copy, Debug)]
pub struct Labels {
    pub no_data: &'static str,
    pub select_all_rows: &'static str,
    pub select_row: fn(&str) -> String,
    pub resize_column: fn(&str) -> String,
    pub go_to_previous_page: &'static str,
    pub go_to_next_page: &'static str,
    pub range: fn(&str, &str, &str) -> String,
    pub range_open: fn(&str, &str) -> String,
    pub page_of: fn(&str, &str) -> String,
    pub page_number: fn(&str) -> String,
    pub show_rows: fn(&str) -> String,
    pub rows_per_page: &'static str,
}

impl Default for Labels {
    fn default() -> Self {
        Self {
            no_data: "No data",
            select_all_rows: "Select all rows",
            select_row: |row| format!("Select row {row}"),
            resize_column: |column| format!("Resize {column} column"),
            go_to_previous_page: "Go to previous page",
            go_to_next_page: "Go to next page",
            range: |start, end, total| format!("{start}-{end} of {total}"),
            range_open: |start, end| format!("{start}-{end}"),
            page_of: |page, total| format!("Page {page} of {total}"),
            page_number: |page| format!("Page {page}"),
            show_rows: |count| format!("Show {count}"),
            rows_per_page: "Rows per page",
        }
    }
}

/// `n` with en-US thousands separators.
pub fn group(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Resolved table paint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableSkin {
    pub shell: Color,
    pub shell_border: Color,
    pub card: Color,
    pub card_border: Color,
    pub row_border: Color,
    pub header_text: Color,
    pub text: Color,
    pub row_hover: Color,
    pub row_selected: Color,
    pub resize_hover: Color,
    pub thumb: [Color; 3],
}

impl TableSkin {
    pub fn new(theme: &Theme) -> Self {
        Self {
            shell: theme.shell,
            shell_border: theme.shell_border,
            card: theme.table_card,
            card_border: theme.table_card_border,
            row_border: theme.table_row_border,
            header_text: theme.muted_foreground,
            text: theme.foreground,
            row_hover: theme.muted,
            row_selected: theme.primary.fade(0.05),
            resize_hover: theme.primary.fade(0.4),
            thumb: corner::ALPHAS.map(|a| theme.foreground.fade(a)),
        }
    }
}

/// The site's demo data.
pub mod demo {
    use super::*;

    /// A prompt row of the data-table demo.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Prompt {
        pub id: String,
        pub prompt: &'static str,
        pub engine: &'static str,
        pub mentioned: bool,
        pub position: Option<u32>,
    }

    const ENGINES: [&str; 4] = ["ChatGPT", "Claude", "Gemini", "Perplexity"];
    const TOPICS: [&str; 6] = [
        "best AI meeting notes app for remote teams",
        "alternatives to Quillboard for startups",
        "privacy-friendly note taker for EU companies",
        "tools that summarize Zoom calls",
        "how to build a team knowledge base",
        "AI notes that integrate with Slack and Linear",
    ];

    pub fn prompts() -> Vec<Prompt> {
        (0..36)
            .map(|i| {
                let mentioned = i % 3 != 1;
                Prompt {
                    id: format!("prompt-{i}"),
                    prompt: TOPICS[i % TOPICS.len()],
                    engine: ENGINES[i % ENGINES.len()],
                    mentioned,
                    position: mentioned.then_some((i % 5) as u32 + 1),
                }
            })
            .collect()
    }

    pub fn prompt_columns() -> Vec<Column> {
        vec![
            Column::new("prompt", "Prompt").sortable(),
            Column::new("engine", "Engine")
                .width(ColumnWidth::Rem(7.5))
                .sortable()
                .collapse_priority(1),
            Column::new("status", "Mention").width(ColumnWidth::Rem(9.0)),
            Column::new("position", "Position")
                .width(ColumnWidth::Rem(7.0))
                .align(Align::Right)
                .sortable()
                .hint("Where the brand first shows up in the answer."),
        ]
    }

    impl Prompt {
        pub fn sort_value(&self, key: &str) -> SortValue {
            match key {
                "engine" => SortValue::Text(self.engine.into()),
                "position" => SortValue::Number(self.position.map_or(f64::INFINITY, f64::from)),
                "status" => SortValue::Text(
                    if self.mentioned {
                        "mentioned"
                    } else {
                        "missing"
                    }
                    .into(),
                ),
                _ => SortValue::Text(self.prompt.into()),
            }
        }

        pub fn position_text(&self) -> String {
            self.position.map_or("–".into(), |p| format!("#{p}"))
        }
    }

    /// A crawler visit (the infinite and empty examples).
    #[derive(Clone, Debug, PartialEq)]
    pub struct Visit {
        pub id: String,
        pub bot: &'static str,
        pub provider: crate::icons::Icon,
        pub path: &'static str,
        pub status: u16,
    }

    const BOTS: [(&str, crate::icons::Icon); 4] = [
        ("GPTBot", crate::icons::Icon::LogoOpenai),
        ("ClaudeBot", crate::icons::Icon::LogoAnthropic),
        ("PerplexityBot", crate::icons::Icon::LogoPerplexity),
        ("Googlebot", crate::icons::Icon::LogoGoogle),
    ];
    const PATHS: [&str; 5] = ["/", "/pricing", "/blog/ai-notes", "/docs", "/changelog"];
    pub const VISIT_PAGE: usize = 25;
    pub const MAX_VISITS: usize = 200;
    pub const LOAD_DELAY: Duration = Duration::from_millis(600);

    /// Visits `from..from + 25`.
    pub fn visits(from: usize) -> Vec<Visit> {
        (from..from + VISIT_PAGE)
            .map(|i| Visit {
                id: format!("visit-{i}"),
                bot: BOTS[i % BOTS.len()].0,
                provider: BOTS[i % BOTS.len()].1,
                path: PATHS[i % PATHS.len()],
                status: if i % 11 == 0 { 404 } else { 200 },
            })
            .collect()
    }

    /// The sample rows the empty example adds.
    pub fn sample_visits() -> Vec<Visit> {
        let row = |i: usize, bot: usize, path: &'static str, status| Visit {
            id: format!("visit-{i}"),
            bot: BOTS[bot].0,
            provider: BOTS[bot].1,
            path,
            status,
        };
        vec![
            row(1, 0, "/pricing", 200),
            row(2, 1, "/docs", 200),
            row(3, 2, "/blog/ai-notes", 200),
            row(4, 3, "/changelog", 404),
        ]
    }

    pub fn visit_columns() -> Vec<Column> {
        vec![
            Column::new("bot", "Crawler").width(ColumnWidth::Rem(11.0)),
            Column::new("path", "Page"),
            Column::new("status", "Status")
                .width(ColumnWidth::Rem(6.0))
                .align(Align::Right),
        ]
    }

    /// A change of the toolbar example.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Change {
        pub id: &'static str,
        pub kind: ChangeKind,
        pub engine: &'static str,
        pub prompt: &'static str,
        pub position: &'static str,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ChangeKind {
        Gained,
        Lost,
        Improved,
    }

    impl ChangeKind {
        pub fn label(self) -> &'static str {
            match self {
                ChangeKind::Gained => "Mention gained",
                ChangeKind::Lost => "Mention lost",
                ChangeKind::Improved => "Position up",
            }
        }

        /// `text-emerald-600 dark:text-emerald-400` or the red pair.
        pub fn color(self, theme: &Theme) -> Color {
            match (self, theme.is_dark()) {
                (ChangeKind::Lost, false) => Color::rgb(0xe7000b),
                (ChangeKind::Lost, true) => Color::rgb(0xff6467),
                (_, false) => Color::rgb(0x009966),
                (_, true) => Color::rgb(0x00d492),
            }
        }
    }

    pub const CHANGES: [Change; 4] = [
        Change {
            id: "change-1",
            kind: ChangeKind::Gained,
            engine: "ChatGPT",
            prompt: "best AI meeting notes app for remote teams",
            position: "#3",
        },
        Change {
            id: "change-2",
            kind: ChangeKind::Lost,
            engine: "Gemini",
            prompt: "alternatives to Quillboard for startups",
            position: "–",
        },
        Change {
            id: "change-3",
            kind: ChangeKind::Improved,
            engine: "Perplexity",
            prompt: "tools that summarize Zoom calls",
            position: "#5 → #2",
        },
        Change {
            id: "change-4",
            kind: ChangeKind::Lost,
            engine: "Claude",
            prompt: "privacy-friendly note taker for EU companies",
            position: "–",
        },
    ];

    pub fn change_columns() -> Vec<Column> {
        vec![
            Column::new("kind", "Change").width(ColumnWidth::Rem(10.0)),
            Column::new("engine", "Engine")
                .width(ColumnWidth::Rem(8.0))
                .collapse_priority(1),
            Column::new("prompt", "Prompt").width(ColumnWidth::Fr(2.0)),
            Column::new("position", "Position")
                .width(ColumnWidth::Rem(7.0))
                .align(Align::Right),
        ]
    }

    /// The toolbar's summary: label, ups, downs.
    pub const SUMMARY: [(&str, u32, u32); 2] = [("Mentions", 1, 2), ("Position", 3, 0)];

    /// Summary chip colours (emerald or red at 10 %, text 700 / 400).
    pub fn chip(theme: &Theme, up: bool) -> (Color, Color) {
        let dark = theme.is_dark();
        match (up, dark) {
            (true, false) => (Color::rgb(0x00bc7d).fade(0.1), Color::rgb(0x007a55)),
            (true, true) => (Color::rgb(0x00bc7d).fade(0.1), Color::rgb(0x00d492)),
            (false, false) => (Color::rgb(0xfb2c36).fade(0.1), Color::rgb(0xc10007)),
            (false, true) => (Color::rgb(0xfb2c36).fade(0.1), Color::rgb(0xff6467)),
        }
    }

    /// The primitives example.
    pub const INVOICES: [[&str; 4]; 3] = [
        ["INV-1042", "Oct 1, 2026", "Growth", "$250.00"],
        ["INV-1018", "Sep 1, 2026", "Growth", "$250.00"],
        ["INV-0995", "Aug 1, 2026", "Starter", "$100.00"],
    ];
    pub const INVOICE_HEADERS: [&str; 4] = ["Invoice", "Date", "Plan", "Amount"];
}

#[cfg(test)]
mod tests {
    use super::*;

    const CH: f32 = 8.0125;

    fn sizing(selectable: bool) -> Sizing {
        Sizing {
            ch: CH,
            min_column_width: DEFAULT_MIN_COLUMN_WIDTH,
            selectable,
        }
    }

    #[test]
    fn demo_columns_match_the_site() {
        let cols = demo::prompt_columns();
        let l = layout_columns(&cols, 840.0, &|_| None, sizing(true));
        assert_eq!(l.visible, vec![0, 1, 2, 3]);
        assert!((l.widths[3] - 140.1).abs() < 0.2, "{:?}", l.widths);
        assert!((l.widths[0] - 387.9).abs() < 0.2, "{:?}", l.widths);
        assert_eq!(l.widths[1], 120.0);
        assert!(l.filler < 0.5);
    }

    #[test]
    fn narrow_tables_drop_low_priority_columns() {
        let cols = demo::prompt_columns();
        let l = layout_columns(&cols, 420.0, &|_| None, sizing(true));
        assert_eq!(l.visible, vec![0, 2, 3]);
    }

    #[test]
    fn fixed_only_tables_leave_a_filler() {
        let cols = vec![Column::new("a", "A").width(ColumnWidth::Px(100.0))];
        let l = layout_columns(&cols, 400.0, &|_| None, sizing(false));
        assert_eq!(l.widths, vec![100.0]);
        assert_eq!(l.filler, 300.0);
        let resized = layout_columns(
            &demo::prompt_columns(),
            840.0,
            &|k| (k == "prompt").then_some(200.0),
            sizing(true),
        );
        assert_eq!(resized.widths[0], 200.0);
        assert!(resized.filler > 180.0);
    }

    #[test]
    fn sort_cycles_and_orders() {
        let s = Sort::toggle(None, "a");
        assert_eq!(s.as_ref().map(|s| s.direction), Some(SortDirection::Asc));
        let s = Sort::toggle(s.as_ref(), "a");
        assert_eq!(s.as_ref().map(|s| s.direction), Some(SortDirection::Desc));
        assert_eq!(Sort::toggle(s.as_ref(), "a"), None);
        let values = [3.0, 1.0, 2.0];
        let sort = Sort {
            key: "n".into(),
            direction: SortDirection::Desc,
        };
        let order = display_order(
            3,
            Some(&sort),
            |i, _| SortValue::Number(values[i]),
            |_| false,
        );
        assert_eq!(order, vec![0, 2, 1]);
        let pinned = display_order(3, None, |_, _| SortValue::Number(0.0), |i| i == 2);
        assert_eq!(pinned, vec![2, 0, 1]);
    }

    #[test]
    fn selection_tracks_mixed_and_all() {
        let ids = ["a", "b"];
        let mut s = Selection::default();
        assert_eq!(s.state(ids), CheckState::Off);
        s.toggle("a");
        assert_eq!(s.state(ids), CheckState::Mixed);
        s.toggle_all(ids);
        assert_eq!(s.state(ids), CheckState::On);
        s.toggle_all(ids);
        assert_eq!(s.state(ids), CheckState::Off);
    }

    #[test]
    fn pager_reads_like_the_site() {
        let labels = Labels::default();
        let p = Paging::client(1, 10).item_label("prompts");
        let pager = p.pager(36, &labels).expect("rows");
        assert_eq!(pager.page_text, "Page 1 of 4");
        assert_eq!(pager.size_text, "Show 10");
        assert!(pager.show_size_select && pager.has_pages);
        assert_eq!((pager.prev, pager.next), (None, Some(2)));
        assert_eq!(pager.range, "1-10 of 36 prompts");
        assert_eq!(p.slice(36), 0..10);
        let last = Paging::client(9, 10).pager(36, &labels).expect("rows");
        assert_eq!(last.page, 4);
        assert_eq!(last.range, "31-36 of 36");
        let cursor = Paging::client(2, 10).mode(PagingMode::Cursor { has_next: false });
        assert_eq!(
            cursor.pager(10, &labels).map(|p| p.page_text),
            Some("Page 2".into())
        );
        assert_eq!(group(1234567), "1,234,567");
    }

    #[test]
    fn viewport_and_window() {
        let v = Viewport::new(36, 48.0, 36.0, 400.0, None);
        assert!(v.scrolls);
        assert_eq!(v.body_height, 336.0);
        assert_eq!(v.window(480.0, 48.0, 36, 2), 8..19);
        let short = Viewport::new(4, 48.0, 36.0, 440.0, None);
        assert!(!short.scrolls);
        assert_eq!(short.height, 192.0);
        let empty = Viewport::new(0, 48.0, 36.0, 300.0, None);
        assert_eq!(empty.height, 240.0);
        assert!(near_end(1000.0, 1300.0, 200.0, 48.0));
    }

    #[test]
    fn loading_states() {
        assert_eq!(
            loading_state(true, 5, None, false),
            Some(LoadingState::Dimmed)
        );
        assert_eq!(
            loading_state(true, 5, Some(true), true),
            Some(LoadingState::More)
        );
        assert_eq!(
            loading_state(true, 0, None, false),
            Some(LoadingState::Skeleton)
        );
        assert_eq!(loading_state(false, 0, None, false), None);
    }

    #[test]
    fn corner_thumb_reaches_the_corners() {
        let t = corner::thumb(0.0, 1200.0, 336.0).expect("scrolls");
        assert_eq!(t.top, corner::GAP);
        assert!(t.at_top_corner && !t.at_bottom_corner);
        let end = corner::thumb(864.0, 1200.0, 336.0).expect("scrolls");
        assert!(end.at_bottom_corner);
        assert!(corner::thumb(0.0, 300.0, 336.0).is_none());
        assert_eq!(corner::drag(&t, 0.0, t.travel), t.scrollable);
    }
}
