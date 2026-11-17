//! GPUI rendering of the Notra tables: the primitives and the data table.

use std::collections::HashMap;
use std::f32::consts::PI;
use std::rc::Rc;
use std::time::Instant;

use ::gpui::{
    AnimationExt as _, AnyElement, App, Bounds, ElementId, Entity, FontWeight, IntoElement,
    MouseButton, MouseDownEvent, MouseMoveEvent, ParentElement, Pixels, RenderOnce, Role,
    ScrollWheelEvent, SharedString, TextRun, Transformation, UniformListScrollHandle, Window,
    anchored, canvas, deferred, div, font, point, prelude::*, px, radians, uniform_list,
};

use super::data_parts::{
    Checkbox, Skeleton, Tooltip, ease_on_hover, focus_ring, glyph, use_eased, use_target,
};
use super::{ActiveTheme, Button, ButtonSize, ButtonVariant, gradient, shadows};
use crate::components::data_parts::{checkbox::CheckState, lerp_color, select};
use crate::components::table::{
    self, Align, Column, LoadingState, Paging, Selection, Sizing, Sort, SortDirection, SortValue,
    TableSkin, Viewport, corner, display_order, layout_columns, loading_state, motion,
};
use crate::fonts;
use crate::icons::Icon;

/// The page-size trigger's height (a small ghost button).
const PAGE_SIZE_TRIGGER: f32 = 28.0;

type CellFn = Rc<dyn Fn(usize, &str, &mut Window, &mut App) -> AnyElement>;
type RowIdFn = Rc<dyn Fn(usize) -> SharedString>;
type SortFn = Rc<dyn Fn(usize, &str) -> SortValue>;
type RowFn = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

fn justify<E: Styled>(e: E, align: Align) -> E {
    match align {
        Align::Left => e.justify_start(),
        Align::Center => e.justify_center(),
        Align::Right => e.justify_end(),
    }
}

/// The dualtone shell around the header band and the lifted body.
fn frame(skin: &TableSkin) -> ::gpui::Div {
    div()
        .flex()
        .flex_col()
        .w_full()
        .p(px(table::RIM))
        .rounded(px(table::SHELL_RADIUS))
        .border_1()
        .border_color(skin.shell_border)
        .bg(skin.shell)
}

/// A header cell of [`Table`].
#[derive(Clone)]
pub struct TableHead {
    label: SharedString,
    align: Align,
    width: Option<f32>,
}

impl TableHead {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            align: Align::Left,
            width: None,
        }
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// A fixed width; columns without one share the rest.
    pub fn width(mut self, px: f32) -> Self {
        self.width = Some(px);
        self
    }
}

/// The `notra-table` primitives: headers on the grey shell, rows in the lifted card.
///
/// ```ignore
/// Table::new("invoices", vec![TableHead::new("Invoice"), TableHead::new("Amount").align(Align::Right)])
///     .row(vec![div().child("INV-1042"), div().child("$250.00")])
/// ```
#[derive(IntoElement)]
pub struct Table {
    id: ElementId,
    heads: Vec<TableHead>,
    rows: Vec<Vec<AnyElement>>,
}

impl Table {
    pub fn new(id: impl Into<ElementId>, heads: Vec<TableHead>) -> Self {
        Self {
            id: id.into(),
            heads,
            rows: Vec::new(),
        }
    }

    pub fn row<E: IntoElement>(mut self, cells: impl IntoIterator<Item = E>) -> Self {
        self.rows.push(
            cells
                .into_iter()
                .map(IntoElement::into_any_element)
                .collect(),
        );
        self
    }
}

impl RenderOnce for Table {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let skin = TableSkin::new(cx.notra());
        let heads = self.heads;
        let cell = |head: &TableHead| {
            let d = div()
                .flex()
                .items_center()
                .min_w_0()
                .px(px(table::CELL_PADDING_X));
            let d = justify(d, head.align);
            match head.width {
                Some(w) => d.flex_none().w(px(w)),
                None => d.flex_1(),
            }
        };
        let count = self.rows.len();
        let rows = self.rows.into_iter().enumerate().map(|(i, cells)| {
            let (hover_state, hover) = use_eased(
                SharedString::from(format!("{}-row-{i}", self.id)),
                motion::ROW_HOVER,
                window,
                cx,
            );
            div()
                .id(SharedString::from(format!("{}-row-{i}-hit", self.id)))
                .flex()
                .h(px(table::ROW_HEIGHT))
                .bg(skin.row_hover.fade(hover))
                .when(i + 1 < count, |d| {
                    d.border_b_1().border_color(skin.row_border)
                })
                .on_hover(ease_on_hover(hover_state))
                .children(
                    cells
                        .into_iter()
                        .zip(heads.iter())
                        .map(|(c, head)| cell(head).text_color(skin.text).child(c)),
                )
        });
        frame(&skin)
            .id(self.id.clone())
            .role(Role::Table)
            .text_size(px(14.0))
            .line_height(px(20.0))
            .child(
                div()
                    .flex()
                    .h(px(table::PRIMITIVE_HEADER_HEIGHT))
                    .text_color(skin.header_text)
                    .font_weight(FontWeight(500.0))
                    .children(heads.iter().map(|h| cell(h).child(h.label.clone()))),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .rounded(px(table::BODY_RADIUS))
                    .border_1()
                    .border_color(skin.card_border)
                    .bg(skin.card)
                    .overflow_hidden()
                    .children(rows),
            )
    }
}

#[derive(Clone)]
struct TableState {
    sort: Option<Sort>,
    selection: Selection,
    page: usize,
    page_size: Option<usize>,
    widths: HashMap<String, f32>,
    width: f32,
    scroll: UniformListScrollHandle,
    resize: Option<(String, f32, f32)>,
    select_open: bool,
    scrolled_at: Option<Instant>,
    last_offset: f32,
    hovered: bool,
    thumb_drag: Option<(f32, f32)>,
    requested_at: Option<usize>,
}

/// Notra's data table.
///
/// Rows are indices; the table asks `cell` for the visible ones only (virtualised
/// when the body scrolls). Sort, selection, page, page size and column widths live in
/// the table's own state.
///
/// ```ignore
/// DataTable::new("prompts", columns, prompts.len())
///     .row_id(move |i| prompts[i].id.clone().into())
///     .sort_value(move |i, key| prompts[i].sort_value(key))
///     .cell(move |i, key, _, _| div().child(prompts[i].text(key)).into_any_element())
///     .selectable(true)
///     .paging(Paging::client(1, 10).item_label("prompts"))
/// ```
#[derive(IntoElement)]
pub struct DataTable {
    id: ElementId,
    columns: Vec<Column>,
    rows: usize,
    row_id: Option<RowIdFn>,
    cell: Option<CellFn>,
    sort_value: Option<SortFn>,
    selectable: bool,
    resizable: bool,
    row_height: f32,
    header_height: f32,
    height: Option<f32>,
    min_height: Option<f32>,
    min_column_width: f32,
    paging: Option<Paging>,
    loading: bool,
    loading_more: Option<bool>,
    skeleton_rows: usize,
    on_end_reached: Option<Handler>,
    on_row_click: Option<RowFn>,
    empty_state: Option<AnyElement>,
    toolbar: Option<AnyElement>,
    footer: Option<AnyElement>,
    scroll_fade: bool,
    labels: table::Labels,
    skeleton_headers: bool,
}

impl DataTable {
    pub fn new(id: impl Into<ElementId>, columns: Vec<Column>, rows: usize) -> Self {
        Self {
            id: id.into(),
            columns,
            rows,
            row_id: None,
            cell: None,
            sort_value: None,
            selectable: false,
            resizable: false,
            row_height: table::ROW_HEIGHT,
            header_height: table::HEADER_HEIGHT,
            height: None,
            min_height: None,
            min_column_width: table::DEFAULT_MIN_COLUMN_WIDTH,
            paging: None,
            loading: false,
            loading_more: None,
            skeleton_rows: table::SKELETON_ROWS,
            on_end_reached: None,
            on_row_click: None,
            empty_state: None,
            toolbar: None,
            footer: None,
            scroll_fade: true,
            labels: table::Labels::default(),
            skeleton_headers: false,
        }
    }

    /// The loading placeholder with the same chrome (`DataTableSkeleton`).
    pub fn skeleton(id: impl Into<ElementId>, columns: Vec<Column>, rows: usize) -> Self {
        let mut t = Self::new(id, columns, 0)
            .loading(true)
            .row_height(52.0)
            .scroll_fade(false);
        t.height = Some((rows as f32 + 1.0) * 52.0);
        t.skeleton_headers = true;
        t
    }

    /// A stable id per row, so selection survives sorting.
    pub fn row_id(mut self, f: impl Fn(usize) -> SharedString + 'static) -> Self {
        self.row_id = Some(Rc::new(f));
        self
    }

    pub fn cell(
        mut self,
        f: impl Fn(usize, &str, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.cell = Some(Rc::new(f));
        self
    }

    pub fn sort_value(mut self, f: impl Fn(usize, &str) -> SortValue + 'static) -> Self {
        self.sort_value = Some(Rc::new(f));
        self
    }

    /// A leading checkbox column with select-all in the header.
    pub fn selectable(mut self, on: bool) -> Self {
        self.selectable = on;
        self
    }

    /// Drag a header's right edge to resize its column.
    pub fn resizable(mut self, on: bool) -> Self {
        self.resizable = on;
        self
    }

    pub fn row_height(mut self, px: f32) -> Self {
        self.row_height = px;
        self
    }

    pub fn header_height(mut self, px: f32) -> Self {
        self.header_height = px;
        self
    }

    /// The table's height (header plus body); the body fits whole rows.
    pub fn height(mut self, px: f32) -> Self {
        self.height = Some(px);
        self
    }

    pub fn min_height(mut self, px: f32) -> Self {
        self.min_height = Some(px);
        self
    }

    pub fn min_column_width(mut self, px: f32) -> Self {
        self.min_column_width = px;
        self
    }

    pub fn paging(mut self, paging: Paging) -> Self {
        self.paging = Some(paging);
        self
    }

    /// Empty tables fill with skeleton rows; tables with rows dim.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Append skeleton rows instead of dimming (an infinite table fetching more).
    pub fn loading_more(mut self, more: bool) -> Self {
        self.loading_more = Some(more);
        self
    }

    pub fn skeleton_rows(mut self, rows: usize) -> Self {
        self.skeleton_rows = rows;
        self
    }

    /// Called once the body scrolls within four rows of its end.
    pub fn on_end_reached(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_end_reached = Some(Rc::new(f));
        self
    }

    pub fn on_row_click(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_row_click = Some(Rc::new(f));
        self
    }

    pub fn empty_state(mut self, empty: impl IntoElement) -> Self {
        self.empty_state = Some(empty.into_any_element());
        self
    }

    /// Content in a card above the column headers.
    pub fn toolbar(mut self, toolbar: impl IntoElement) -> Self {
        self.toolbar = Some(toolbar.into_any_element());
        self
    }

    /// Content under the body on the shell, before the pager.
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    /// Fade the edges while rows are scrolled out of view (on by default).
    pub fn scroll_fade(mut self, on: bool) -> Self {
        self.scroll_fade = on;
        self
    }

    pub fn labels(mut self, labels: table::Labels) -> Self {
        self.labels = labels;
        self
    }
}

impl RenderOnce for DataTable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let skin = TableSkin::new(&theme);
        let id = self.id.clone();
        let key = move |part: &str| SharedString::from(format!("{id}-{part}"));
        let initial_page = self.paging.as_ref().map_or(1, |p| p.page);
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| TableState {
            sort: None,
            selection: Selection::default(),
            page: initial_page,
            page_size: None,
            widths: HashMap::new(),
            width: 0.0,
            scroll: UniformListScrollHandle::new(),
            resize: None,
            select_open: false,
            scrolled_at: None,
            last_offset: 0.0,
            hovered: false,
            thumb_drag: None,
            requested_at: None,
        });
        let now = Instant::now();
        let ch = {
            let run = TextRun {
                len: 1,
                font: font(fonts::SANS),
                color: theme.foreground.into(),
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            f32::from(
                window
                    .text_system()
                    .shape_line("0".into(), px(14.0), &[run], None)
                    .width,
            )
        };
        let s = state.read(cx).clone();
        let row_id = self.row_id.clone();
        let id_of = move |i: usize| -> SharedString {
            row_id
                .as_ref()
                .map_or_else(|| SharedString::from(i.to_string()), |f| f(i))
        };
        let sort_value = self.sort_value.clone();
        let order = display_order(
            self.rows,
            s.sort.as_ref().filter(|_| sort_value.is_some()),
            |i, key| {
                sort_value
                    .as_ref()
                    .map_or(SortValue::Number(0.0), |f| f(i, key))
            },
            |_| false,
        );
        let paging = self.paging.clone().map(|mut p| {
            p.page = s.page;
            if let Some(size) = s.page_size {
                p.page_size = size;
            }
            p
        });
        let slice = paging
            .as_ref()
            .map_or(0..order.len(), |p| p.slice(order.len()));
        let rows: Rc<Vec<usize>> = Rc::new(order[slice].to_vec());
        let pager = paging
            .as_ref()
            .and_then(|p| p.pager(order.len(), &self.labels));
        let row_ids: Vec<SharedString> = rows.iter().map(|&i| id_of(i)).collect();
        let all_ids: Vec<SharedString> = order.iter().map(|&i| id_of(i)).collect();
        let header_check = s.selection.state(all_ids.iter().map(|s| s.as_ref()));
        let width = if s.width > 0.0 { s.width } else { 800.0 };
        if s.width <= 0.0 {
            window.request_animation_frame();
        }
        let widths = s.widths.clone();
        let layout = layout_columns(
            &self.columns,
            (width - table::FRAME_INSET).max(0.0),
            &|k| widths.get(k).copied(),
            Sizing {
                ch,
                min_column_width: self.min_column_width,
                selectable: self.selectable,
            },
        );
        let height = self.height.unwrap_or(if paging.is_some() {
            rows.len().max(1) as f32 * self.row_height + self.header_height
        } else {
            table::DEFAULT_HEIGHT
        });
        let infinite = self.on_end_reached.is_some();
        let load = loading_state(self.loading, rows.len(), self.loading_more, infinite);
        let extra = if load == Some(LoadingState::More) {
            self.skeleton_rows
        } else {
            0
        };
        let viewport = Viewport::new(
            rows.len() + extra,
            self.row_height,
            self.header_height,
            height,
            self.min_height,
        );
        let offset = -f32::from(s.scroll.0.borrow().base_handle.offset().y);
        let content = (rows.len() + extra) as f32 * self.row_height;
        let at_start = offset <= 1.0;
        let at_end = offset + viewport.height + 1.0 >= content;
        let scrolling = s
            .scrolled_at
            .is_some_and(|at| now.duration_since(at) < corner::IDLE);
        let thumb = corner::thumb(offset, content, viewport.height)
            .filter(|_| viewport.scrolls && !rows.is_empty());
        let thumb_visible = thumb.is_some() && (s.hovered || scrolling || s.thumb_drag.is_some());
        let select_open = s.select_open;
        let resizing = s.resize.is_some();

        if scrolling {
            window.request_animation_frame();
        }

        let corner_end = thumb_visible && thumb.is_some_and(|t| t.at_bottom_corner);
        let corner_start = thumb_visible && thumb.is_some_and(|t| t.at_top_corner);
        let card_br = use_target(
            key("corner-end"),
            motion::EDGE,
            if corner_end { 1.0 } else { 0.0 },
            window,
            cx,
        );
        let card_tr = use_target(
            key("corner-start"),
            motion::EDGE,
            if corner_start { 1.0 } else { 0.0 },
            window,
            cx,
        );
        let thumb_alpha = use_target(
            key("thumb"),
            motion::EDGE,
            if thumb_visible { 1.0 } else { 0.0 },
            window,
            cx,
        );
        let fade_top = use_target(
            key("fade-top"),
            motion::EDGE,
            if at_start { 0.0 } else { 1.0 },
            window,
            cx,
        );
        let fade_bottom = use_target(
            key("fade-bottom"),
            motion::EDGE,
            if at_end || !viewport.scrolls {
                0.0
            } else {
                1.0
            },
            window,
            cx,
        );
        let dim = use_target(
            key("dim"),
            motion::DIM,
            if load == Some(LoadingState::Dimmed) {
                table::DIM_OPACITY
            } else {
                1.0
            },
            window,
            cx,
        );

        let checkbox_cell = || {
            div()
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .w(px(layout.checkbox))
        };

        let header_cells = layout.visible.iter().zip(&layout.widths).map(|(&ci, &w)| {
            let column = &self.columns[ci];
            let active = s.sort.as_ref().filter(|s| s.key == column.key);
            let desc = active.is_some_and(|s| s.direction == SortDirection::Desc);
            let arrow_opacity = use_target(
                key(&format!("sort-{}", column.key)),
                motion::SORT,
                if active.is_some() {
                    1.0
                } else {
                    table::SORT_IDLE_OPACITY
                },
                window,
                cx,
            );
            let turn = use_target(
                key(&format!("turn-{}", column.key)),
                motion::SORT,
                if desc { 1.0 } else { 0.0 },
                window,
                cx,
            );
            let (hover_state, hover) = use_eased(
                key(&format!("head-{}", column.key)),
                motion::ROW_HOVER,
                window,
                cx,
            );
            let ink = if active.is_some() || self.skeleton_headers {
                skin.text
            } else {
                lerp_color(skin.header_text, skin.text, hover)
            };
            let label = div()
                .flex()
                .items_center()
                .gap(px(table::ICON_GAP))
                .whitespace_nowrap()
                .child(column.header.clone())
                .when(column.hint.is_some(), |d| {
                    d.child(div().opacity(table::HINT_OPACITY).child(glyph(
                        Icon::HugeInformationCircle,
                        table::HINT_ICON,
                        ink,
                    )))
                });
            let arrow = column.sortable.then(|| {
                div().flex_none().opacity(arrow_opacity).child(
                    glyph(Icon::HugeArrowUp01, table::SORT_ICON, ink)
                        .with_transformation(Transformation::rotate(radians(turn * PI))),
                )
            });
            let right = column.align == Align::Right;
            let sort_key = column.key.clone();
            let sortable = column.sortable;
            let mut content = div()
                .id(key(&format!("head-{}-hit", column.key)))
                .flex()
                .flex_1()
                .h_full()
                .min_w_0()
                .items_center()
                .gap(px(table::ICON_GAP))
                .px(px(table::CELL_PADDING_X))
                .rounded(px(6.0))
                .text_color(ink)
                .on_hover(ease_on_hover(hover_state));
            content = justify(content, column.align);
            if self.skeleton_headers {
                content = content.child(Skeleton::new(14.0).w(px(64.0)));
            } else {
                content = if right {
                    content.children(arrow).child(label)
                } else {
                    content.child(label).children(arrow)
                };
            }
            if let Some(hint) = column.hint.clone() {
                content = content.tooltip(Tooltip::text(hint));
            }
            if sortable {
                let sort_state = state.clone();
                content = content
                    .role(Role::ColumnHeader)
                    .cursor_pointer()
                    .focus_visible(move |s| s.shadow(focus_ring(&theme)))
                    .on_click(move |_, _, cx| {
                        sort_state.update(cx, |s, cx| {
                            s.sort = Sort::toggle(s.sort.as_ref(), &sort_key);
                            cx.notify();
                        });
                    });
            }
            let resize = self.resizable.then(|| {
                let resize_key = column.key.clone();
                let resize_state = state.clone();
                let snapshot: Vec<(String, f32)> = layout
                    .visible
                    .iter()
                    .zip(&layout.widths)
                    .map(|(&i, &w)| (self.columns[i].key.clone(), w))
                    .collect();
                div()
                    .id(key(&format!("resize-{}", column.key)))
                    .absolute()
                    .top_0()
                    .right_0()
                    .h_full()
                    .w(px(table::RESIZE_HANDLE_WIDTH))
                    .cursor_col_resize()
                    .hover(move |s| s.bg(skin.resize_hover))
                    .on_mouse_down(MouseButton::Left, move |e: &MouseDownEvent, _, cx| {
                        cx.stop_propagation();
                        let x = f32::from(e.position.x);
                        resize_state.update(cx, |s, cx| {
                            for (k, w) in &snapshot {
                                s.widths.entry(k.clone()).or_insert(*w);
                            }
                            let start = s.widths.get(&resize_key).copied().unwrap_or(0.0);
                            s.resize = Some((resize_key.clone(), x, start));
                            cx.notify();
                        });
                    })
            });
            div()
                .relative()
                .flex()
                .flex_none()
                .w(px(w))
                .h_full()
                .children(Some(content))
                .children(resize)
        });

        let header = div()
            .flex()
            .h(px(self.header_height))
            .text_color(skin.header_text)
            .font_weight(FontWeight(500.0))
            .px(px(1.0))
            .when(self.selectable, |d| {
                let toggle_state = state.clone();
                let ids = all_ids.clone();
                d.child(
                    checkbox_cell().child(
                        Checkbox::new(key("select-all"), header_check)
                            .ink(skin.header_text)
                            .label(self.labels.select_all_rows)
                            .on_toggle(move |_, cx| {
                                toggle_state.update(cx, |s, cx| {
                                    s.selection.toggle_all(ids.iter().map(|s| s.as_ref()));
                                    cx.notify();
                                });
                            }),
                    ),
                )
            })
            .children(header_cells)
            .child(div().flex_1());

        let skeleton_row = |i: usize, last: bool| {
            div()
                .flex()
                .flex_none()
                .h(px(self.row_height))
                .when(!last, |d| d.border_b_1().border_color(skin.row_border))
                .when(self.selectable, |d| d.child(checkbox_cell()))
                .children(layout.visible.iter().zip(&layout.widths).map(|(&ci, &w)| {
                    let right = self.columns[ci].align == Align::Right;
                    let bar = Skeleton::new(12.0).radius(999.0);
                    let bar = if right {
                        bar.w(px(40.0))
                    } else {
                        bar.w(::gpui::relative(2.0 / 3.0))
                    };
                    justify(
                        div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .w(px(w))
                            .px(px(table::CELL_PADDING_X)),
                        self.columns[ci].align,
                    )
                    .child(bar)
                }))
                .child(div().flex_1())
                .id(key(&format!("skeleton-{i}")))
        };

        let body_content: AnyElement = if rows.is_empty() && load == Some(LoadingState::Skeleton) {
            let count = ((viewport.body_height / self.row_height).ceil() as usize).max(1);
            div()
                .flex()
                .flex_col()
                .children((0..count).map(|i| skeleton_row(i, i + 1 == count)))
                .into_any_element()
        } else if rows.is_empty() {
            let empty = self.empty_state.unwrap_or_else(|| {
                div()
                    .text_color(theme.muted_foreground)
                    .child(self.labels.no_data)
                    .into_any_element()
            });
            div()
                .flex()
                .items_center()
                .justify_center()
                .h(px(viewport.body_height))
                .px(px(24.0))
                .text_center()
                .child(empty)
                .into_any_element()
        } else {
            let cell = self.cell.clone();
            let columns = Rc::new(self.columns.clone());
            let visible = Rc::new(
                layout
                    .visible
                    .iter()
                    .copied()
                    .zip(layout.widths.iter().copied())
                    .collect::<Vec<_>>(),
            );
            let checkbox = layout.checkbox;
            let row_height = self.row_height;
            let ids = Rc::new(row_ids.clone());
            let list_state = state.clone();
            let rows_for_list = rows.clone();
            let on_row_click = self.on_row_click.clone();
            let selectable = self.selectable;
            let labels = self.labels;
            let total = rows.len() + extra;
            let table_id = self.id.clone();
            let skeleton_rows: Vec<AnyElement> = (rows.len()..total)
                .map(|i| skeleton_row(i, i + 1 == total).into_any_element())
                .collect();
            let skeletons = Rc::new(std::cell::RefCell::new(
                skeleton_rows.into_iter().map(Some).collect::<Vec<_>>(),
            ));
            let end_reached = self.on_end_reached.clone().filter(|_| !self.loading);
            let loaded = rows.len();
            uniform_list(key("rows"), total, move |range, window, cx| {
                let watch = list_state.clone();
                let near_end = range.end + table::END_THRESHOLD_ROWS as usize >= loaded;
                let request = end_reached
                    .clone()
                    .filter(|_| near_end && watch.read(cx).requested_at != Some(loaded));
                window.defer(cx, move |window, cx| {
                    let offset =
                        -f32::from(watch.read(cx).scroll.0.borrow().base_handle.offset().y);
                    watch.update(cx, |s, cx| {
                        if (offset - s.last_offset).abs() > 0.5 {
                            s.last_offset = offset;
                            s.scrolled_at = Some(Instant::now());
                            cx.notify();
                        }
                        if request.is_some() {
                            s.requested_at = Some(loaded);
                        }
                    });
                    if let Some(end) = request {
                        end(window, cx);
                    }
                });
                let theme = *cx.notra();
                let skin = TableSkin::new(&theme);
                range
                    .map(|pos| {
                        if pos >= rows_for_list.len() {
                            return skeletons
                                .borrow_mut()
                                .get_mut(pos - rows_for_list.len())
                                .and_then(Option::take)
                                .unwrap_or_else(|| div().into_any_element());
                        }
                        let row = rows_for_list[pos];
                        let row_key = ids[pos].clone();
                        let selected = list_state.read(cx).selection.contains(&row_key);
                        let (hover_state, hover) = use_eased(
                            SharedString::from(format!("{table_id}-row-{row_key}")),
                            motion::ROW_HOVER,
                            window,
                            cx,
                        );
                        let fill = if selected {
                            skin.row_selected
                        } else {
                            skin.row_hover.fade(hover)
                        };
                        let toggle_state = list_state.clone();
                        let toggle_key = row_key.clone();
                        let cells = visible.iter().map(|&(ci, w)| {
                            let column = &columns[ci];
                            let content = cell
                                .as_ref()
                                .map(|f| f(row, &column.key, window, cx))
                                .unwrap_or_else(|| div().into_any_element());
                            justify(
                                div()
                                    .flex()
                                    .flex_none()
                                    .items_center()
                                    .w(px(w))
                                    .min_w_0()
                                    .overflow_hidden()
                                    .px(px(table::CELL_PADDING_X))
                                    .whitespace_nowrap(),
                                column.align,
                            )
                            .child(div().min_w_0().truncate().child(content))
                        });
                        div()
                            .id(SharedString::from(format!("{table_id}-row-{row_key}-hit")))
                            .role(Role::Row)
                            .aria_selected(selected)
                            .flex()
                            .h(px(row_height))
                            .bg(fill)
                            .text_color(skin.text)
                            .when(pos + 1 < total, |d| {
                                d.border_b_1().border_color(skin.row_border)
                            })
                            .on_hover(ease_on_hover(hover_state))
                            .when(selectable, |d| {
                                d.child(
                                    div()
                                        .flex()
                                        .flex_none()
                                        .items_center()
                                        .justify_center()
                                        .w(px(checkbox))
                                        .child(
                                            Checkbox::new(
                                                SharedString::from(format!(
                                                    "{table_id}-check-{row_key}"
                                                )),
                                                if selected {
                                                    CheckState::On
                                                } else {
                                                    CheckState::Off
                                                },
                                            )
                                            .on_toggle(move |_, cx| {
                                                toggle_state.update(cx, |s, cx| {
                                                    s.selection.toggle(&toggle_key);
                                                    cx.notify();
                                                });
                                            })
                                            .label((labels.select_row)(&(pos + 1).to_string())),
                                        ),
                                )
                            })
                            .children(cells)
                            .child(div().flex_1())
                            .when_some(on_row_click.clone(), |d, f| {
                                d.cursor_pointer()
                                    .on_click(move |_, window, cx| f(row, window, cx))
                            })
                            .into_any_element()
                    })
                    .collect::<Vec<_>>()
            })
            .track_scroll(&s.scroll)
            .h(px(viewport.height))
            .into_any_element()
        };

        let scroll_state = state.clone();
        let hover_state = state.clone();
        let fade = |top: bool, alpha: f32| {
            div()
                .absolute()
                .left_0()
                .right_0()
                .h(px(table::FADE_HEIGHT))
                .map(|d| if top { d.top_0() } else { d.bottom_0() })
                .opacity(alpha)
                .bg(if top {
                    gradient(skin.card, skin.card.fade(0.0))
                } else {
                    gradient(skin.card.fade(0.0), skin.card)
                })
        };
        let thumb_el = thumb.map(|t| {
            let drag_state = state.clone();
            div()
                .id(key("thumb"))
                .absolute()
                .top(px(t.top))
                .right(px(corner::GAP))
                .w(px(corner::WIDTH))
                .h(px(t.length))
                .rounded_full()
                .bg(skin.thumb[0])
                .hover(move |s| s.bg(skin.thumb[1]))
                .active(move |s| s.bg(skin.thumb[2]))
                .opacity(thumb_alpha)
                .role(Role::ScrollBar)
                .on_mouse_down(MouseButton::Left, move |e: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    let y = f32::from(e.position.y);
                    drag_state.update(cx, |s, cx| {
                        s.thumb_drag = Some((y, offset));
                        cx.notify();
                    });
                })
        });
        let body_radius = px(table::BODY_RADIUS);
        let br = px(table::BODY_RADIUS + (corner::CARD_RADIUS - table::BODY_RADIUS) * card_br);
        let tr = px(table::BODY_RADIUS + (corner::CARD_RADIUS - table::BODY_RADIUS) * card_tr);
        let body = div()
            .id(key("body"))
            .relative()
            .rounded_tl(body_radius)
            .rounded_bl(body_radius)
            .rounded_tr(tr)
            .rounded_br(br)
            .border_1()
            .border_color(skin.card_border)
            .bg(skin.card)
            .shadow(shadows(theme.table_card_shadow))
            .overflow_hidden()
            .h(px(viewport.height + 2.0))
            .on_hover(move |hovered, _, cx| {
                let hovered = *hovered;
                hover_state.update(cx, |s, cx| {
                    s.hovered = hovered;
                    cx.notify();
                });
            })
            .on_scroll_wheel(move |e: &ScrollWheelEvent, _, cx| {
                let handle = scroll_state.read(cx).scroll.0.borrow().base_handle.clone();
                let top = -f32::from(handle.offset().y);
                let max = f32::from(handle.max_offset().y);
                let up = f32::from(e.delta.pixel_delta(px(20.0)).y) > 0.0;
                if (up && top > 0.5) || (!up && top < max - 0.5) {
                    cx.stop_propagation();
                }
                scroll_state.update(cx, |s, cx| {
                    s.scrolled_at = Some(Instant::now());
                    cx.notify();
                });
            })
            .child(div().opacity(dim).child(body_content))
            .when(self.scroll_fade && viewport.scrolls, |d| {
                d.child(fade(true, fade_top))
                    .child(fade(false, fade_bottom))
            })
            .children(thumb_el);

        let toolbar = self.toolbar.map(|t| {
            div()
                .rounded(px(table::BODY_RADIUS))
                .border_1()
                .border_color(skin.card_border)
                .bg(skin.card)
                .shadow(shadows(theme.table_card_shadow))
                .child(t)
        });

        let footer = pager.map(|pager| {
            footer_pager(
                &self.id,
                &self.labels,
                pager,
                &state,
                select_open,
                window,
                cx,
            )
        });
        let shell_br =
            px(table::SHELL_RADIUS + (corner::SHELL_RADIUS - table::SHELL_RADIUS) * card_br);
        let has_footer = footer.is_some() || self.footer.is_some();
        let root_state = state.clone();
        let move_state = state.clone();
        div()
            .id(self.id)
            .relative()
            .w_full()
            .min_w_0()
            .text_size(px(14.0))
            .line_height(px(20.0))
            .font_family(fonts::SANS)
            .when(resizing, |d| d.cursor_col_resize())
            .on_mouse_move(move |e: &MouseMoveEvent, _, cx| {
                let x = f32::from(e.position.x);
                let y = f32::from(e.position.y);
                move_state.update(cx, |s, cx| {
                    if let Some((key, start, from)) = s.resize.clone() {
                        s.widths
                            .insert(key, (from + x - start).max(table::DEFAULT_MIN_COLUMN_WIDTH));
                        cx.notify();
                    }
                    if let Some((start_y, start)) = s.thumb_drag {
                        let handle = s.scroll.0.borrow().base_handle.clone();
                        if let Some(t) = corner::thumb(start, content, viewport.height) {
                            let next = corner::drag(&t, start, y - start_y);
                            handle.set_offset(point(px(0.0), px(-next)));
                            s.scrolled_at = Some(Instant::now());
                            cx.notify();
                        }
                    }
                });
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                root_state.update(cx, |s, cx| {
                    if s.resize.take().is_some() || s.thumb_drag.take().is_some() {
                        cx.notify();
                    }
                });
            })
            .child(measure(state.clone()))
            .child(
                frame(&skin)
                    .rounded_br(if has_footer {
                        px(table::SHELL_RADIUS)
                    } else {
                        shell_br
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .rounded_t(px(table::BODY_RADIUS))
                            .children(toolbar)
                            .child(header),
                    )
                    .child(body)
                    .children(self.footer)
                    .children(footer),
            )
    }
}

/// Records the table's laid-out width into its state for the next frame's layout.
fn measure(state: Entity<TableState>) -> impl IntoElement {
    canvas(
        move |bounds: Bounds<Pixels>, _, cx| {
            let width = f32::from(bounds.size.width);
            state.update(cx, |s, cx| {
                if (s.width - width).abs() > 0.5 {
                    s.width = width;
                    cx.notify();
                }
            });
        },
        |_, _, _, _| {},
    )
    .absolute()
    .inset_0()
}

fn footer_pager(
    id: &ElementId,
    labels: &table::Labels,
    pager: table::Pager,
    state: &Entity<TableState>,
    open: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let theme = *cx.notra();
    let owner = id.clone();
    let key = move |part: &str| SharedString::from(format!("{owner}-{part}"));
    let (trigger_state, hover) = use_eased(key("size-trigger"), motion::ROW_HOVER, window, cx);
    let toggle = state.clone();
    let left: AnyElement = if pager.show_size_select {
        let popup = open.then(|| page_size_popup(id, labels, &pager, state, cx));
        div()
            .relative()
            .flex()
            .flex_col()
            .ml(px(-table::PAGE_SIZE_PULL))
            .child(
                div()
                    .id(key("size"))
                    .role(Role::ComboBox)
                    .aria_label(labels.rows_per_page)
                    .aria_expanded(open)
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .h(px(28.0))
                    .px(px(10.0))
                    .rounded(px(6.0))
                    .bg(theme.muted.fade(hover))
                    .text_color(lerp_color(theme.muted_foreground, theme.foreground, hover))
                    .cursor_pointer()
                    .on_hover(ease_on_hover(trigger_state))
                    .on_click(move |_, _, cx| {
                        toggle.update(cx, |s, cx| {
                            s.select_open = !s.select_open;
                            cx.notify();
                        });
                    })
                    .child(pager.size_text.clone())
                    .child(glyph(
                        Icon::ChevronsUpDown,
                        select::TRIGGER_ICON,
                        theme.muted_foreground,
                    )),
            )
            .children(popup)
            .into_any_element()
    } else {
        div()
            .truncate()
            .child(pager.range.clone())
            .into_any_element()
    };
    let nav = pager.has_pages.then(|| {
        let go = |target: Option<usize>, icon: Icon, part: &str| {
            let page_state = state.clone();
            Button::icon_button(key(part), icon)
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::IconSm)
                .disabled(target.is_none())
                .on_click(move |_, _, cx| {
                    if let Some(page) = target {
                        page_state.update(cx, |s, cx| {
                            s.page = page;
                            cx.notify();
                        });
                    }
                })
        };
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(table::FOOTER_GAP))
            .child(div().child(pager.page_text.clone()))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(table::PAGER_BUTTON_GAP))
                    .child(go(pager.prev, Icon::HugeArrowLeft01Bold, "prev"))
                    .child(go(pager.next, Icon::HugeArrowRight01Bold, "next")),
            )
    });
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap(px(table::FOOTER_GAP))
        .min_h(px(table::FOOTER_MIN_HEIGHT))
        .px(px(table::FOOTER_PADDING_X))
        .py(px(6.0))
        .text_size(px(12.0))
        .line_height(px(16.0))
        .text_color(theme.muted_foreground)
        .child(left)
        .children(nav)
        .into_any_element()
}

fn page_size_popup(
    id: &ElementId,
    labels: &table::Labels,
    pager: &table::Pager,
    state: &Entity<TableState>,
    cx: &mut App,
) -> AnyElement {
    let theme = *cx.notra();
    let skin = select::PopupSkin::new(&theme);
    let id = id.clone();
    let show_rows = labels.show_rows;
    let close = state.clone();
    let items = pager.sizes.iter().map(|&size| {
        let pick = state.clone();
        let checked = size == pager.page_size;
        div()
            .id(SharedString::from(format!("{id}-size-{size}")))
            .role(Role::ListBoxOption)
            .aria_selected(checked)
            .relative()
            .flex()
            .items_center()
            .h(px(select::ITEM_HEIGHT))
            .pl(px(select::ITEM_PADDING_LEFT))
            .pr(px(select::ITEM_PADDING_RIGHT))
            .rounded(px(select::ITEM_RADIUS))
            .text_size(px(select::ITEM_FONT_SIZE))
            .line_height(px(select::ITEM_LINE_HEIGHT))
            .text_color(skin.text)
            .cursor_pointer()
            .hover(move |s| {
                s.bg(skin.item_highlight)
                    .text_color(skin.item_highlight_text)
            })
            .on_click(move |_, _, cx| {
                pick.update(cx, |s, cx| {
                    s.page_size = Some(size);
                    s.page = 1;
                    s.select_open = false;
                    cx.notify();
                });
            })
            .child(show_rows(&table::group(size)))
            .when(checked, |d| {
                d.child(
                    div()
                        .absolute()
                        .right(px(select::INDICATOR_RIGHT))
                        .child(glyph(Icon::Check, select::INDICATOR_SIZE, skin.text)),
                )
            })
    });
    let enter = select::ENTER;
    let popup = div()
        .id(SharedString::from(format!("{}-size-popup", id)))
        .role(Role::ListBox)
        .occlude()
        .min_w(px(select::POPUP_MIN_WIDTH))
        .p(px(select::POPUP_PADDING))
        .rounded(px(select::POPUP_RADIUS))
        .border_1()
        .border_color(skin.border)
        .bg(gradient(skin.top, skin.bottom))
        .shadow(shadows(&skin.shadows))
        .font_family(fonts::SANS)
        .on_mouse_down_out(move |_, _, cx| {
            close.update(cx, |s, cx| {
                s.select_open = false;
                cx.notify();
            });
        })
        .children(items)
        .with_animation(
            SharedString::from(format!("{}-size-popup-enter", id)),
            ::gpui::Animation::new(enter.duration).with_easing(move |t| enter.easing.ease(t)),
            |d, t| d.opacity(t),
        );
    div()
        .absolute()
        .top_0()
        .left_0()
        .child(
            deferred(
                anchored().child(
                    div()
                        .pt(px(PAGE_SIZE_TRIGGER + select::POPUP_GAP))
                        .pb(px(select::POPUP_GAP))
                        .child(popup),
                ),
            )
            .with_priority(1),
        )
        .into_any_element()
}
