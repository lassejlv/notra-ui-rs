//! Text entry for GPUI: an editable [`TextField`] (caret, selection, IME, clipboard) and
//! the shadcn chrome around it: [`Input`], [`Textarea`] and [`InputGroup`].

use std::ops::Range;
use std::time::Instant;

use ::gpui::{
    AnyElement, App, Bounds, ClickEvent, ClipboardItem, ContentMask, Context, CursorStyle,
    ElementId, ElementInputHandler, Entity, EntityInputHandler, EventEmitter, FocusHandle,
    Focusable, FontWeight, GlobalElementId, Hsla, InspectorElementId, IntoElement, KeyBinding,
    LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels,
    Point, RenderOnce, Role, ScrollWheelEvent, SharedString, Style, Styled, Task, TextAlign,
    TextRun, UTF16Selection, UnderlineStyle, Window, WrappedLine, actions, div, fill, point,
    prelude::*, px, relative, size, svg,
};

use super::{ActiveTheme, shadows};
use crate::components::input::{
    AddonAlign, CARET_BLINK, CARET_WIDTH, DISABLED_OPACITY, Editing, FieldMetrics, FieldSkin,
    INPUT, Motion, TEXTAREA, TRANSITION, group,
};
use crate::fonts;
use crate::icons::Icon;
use crate::motion::{Animated, Tween};
use crate::theme::{Color, Shadow};

actions!(
    notra_text_field,
    [
        Backspace,
        Delete,
        DeleteWordLeft,
        DeleteWordRight,
        Left,
        Right,
        Up,
        Down,
        WordLeft,
        WordRight,
        Home,
        End,
        DocStart,
        DocEnd,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectWordLeft,
        SelectWordRight,
        SelectHome,
        SelectEnd,
        SelectDocStart,
        SelectDocEnd,
        SelectAll,
        Copy,
        Cut,
        Paste,
        Enter,
        Newline,
        Submit,
        ShowCharacterPalette,
    ]
);

const CONTEXT: &str = "NotraTextField";

/// Binds the editing keys every [`TextField`] listens for. [`super::init`] calls it.
pub fn bind_keys(cx: &mut App) {
    let ctx = Some(CONTEXT);
    let word = if cfg!(target_os = "macos") {
        "alt"
    } else {
        "ctrl"
    };
    let mut bindings = vec![
        KeyBinding::new("backspace", Backspace, ctx),
        KeyBinding::new("shift-backspace", Backspace, ctx),
        KeyBinding::new("delete", Delete, ctx),
        KeyBinding::new(&format!("{word}-backspace"), DeleteWordLeft, ctx),
        KeyBinding::new(&format!("{word}-delete"), DeleteWordRight, ctx),
        KeyBinding::new("left", Left, ctx),
        KeyBinding::new("right", Right, ctx),
        KeyBinding::new("up", Up, ctx),
        KeyBinding::new("down", Down, ctx),
        KeyBinding::new(&format!("{word}-left"), WordLeft, ctx),
        KeyBinding::new(&format!("{word}-right"), WordRight, ctx),
        KeyBinding::new("home", Home, ctx),
        KeyBinding::new("end", End, ctx),
        KeyBinding::new("secondary-home", DocStart, ctx),
        KeyBinding::new("secondary-end", DocEnd, ctx),
        KeyBinding::new("shift-left", SelectLeft, ctx),
        KeyBinding::new("shift-right", SelectRight, ctx),
        KeyBinding::new("shift-up", SelectUp, ctx),
        KeyBinding::new("shift-down", SelectDown, ctx),
        KeyBinding::new(&format!("{word}-shift-left"), SelectWordLeft, ctx),
        KeyBinding::new(&format!("{word}-shift-right"), SelectWordRight, ctx),
        KeyBinding::new("shift-home", SelectHome, ctx),
        KeyBinding::new("shift-end", SelectEnd, ctx),
        KeyBinding::new("secondary-shift-home", SelectDocStart, ctx),
        KeyBinding::new("secondary-shift-end", SelectDocEnd, ctx),
        KeyBinding::new("secondary-a", SelectAll, ctx),
        KeyBinding::new("secondary-c", Copy, ctx),
        KeyBinding::new("secondary-x", Cut, ctx),
        KeyBinding::new("secondary-v", Paste, ctx),
        KeyBinding::new("enter", Enter, ctx),
        KeyBinding::new("shift-enter", Newline, ctx),
        KeyBinding::new("secondary-enter", Submit, ctx),
    ];
    if cfg!(target_os = "macos") {
        bindings.extend([
            KeyBinding::new("cmd-left", Home, ctx),
            KeyBinding::new("cmd-right", End, ctx),
            KeyBinding::new("cmd-up", DocStart, ctx),
            KeyBinding::new("cmd-down", DocEnd, ctx),
            KeyBinding::new("cmd-shift-left", SelectHome, ctx),
            KeyBinding::new("cmd-shift-right", SelectEnd, ctx),
            KeyBinding::new("cmd-shift-up", SelectDocStart, ctx),
            KeyBinding::new("cmd-shift-down", SelectDocEnd, ctx),
            KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, ctx),
        ]);
    }
    cx.bind_keys(bindings);
}

/// What a [`TextField`] tells its owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextFieldEvent {
    /// The text changed.
    Change,
    /// Return in a single-line field (or one set to submit on Return), or
    /// Cmd/Ctrl+Return anywhere.
    Submit,
}

/// An editable text field: one line that scrolls sideways, or several that wrap and grow
/// between the metrics' bounds before scrolling. It paints only its text and padding; wrap it
/// in [`Input`], [`Textarea`], [`InputGroup`] or a composer for the box.
///
/// ```ignore
/// let field = cx.new(|cx| TextField::new(cx).placeholder("Email"));
/// cx.subscribe(&field, |this, field, event, cx| { .. }).detach();
/// Input::new(field.clone())
/// ```
pub struct TextField {
    focus: FocusHandle,
    editing: Editing,
    placeholder: SharedString,
    metrics: FieldMetrics,
    padding: [f32; 4],
    multiline: bool,
    submit_on_enter: bool,
    disabled: bool,
    label: Option<SharedString>,
    scroll: Point<Pixels>,
    reveal_caret: bool,
    selecting: bool,
    caret_on: bool,
    blink: Option<Task<()>>,
    layout: Option<Layout>,
}

impl EventEmitter<TextFieldEvent> for TextField {}

impl Focusable for TextField {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl TextField {
    /// A single-line field with shadcn `Input` metrics.
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            editing: Editing::default(),
            placeholder: SharedString::default(),
            metrics: INPUT,
            padding: padding(&INPUT),
            multiline: false,
            submit_on_enter: true,
            disabled: false,
            label: None,
            scroll: Point::default(),
            reveal_caret: false,
            selecting: false,
            caret_on: true,
            blink: None,
            layout: None,
        }
    }

    /// A wrapping field that grows with its text (`field-sizing-content`), shadcn `Textarea`
    /// metrics unless `metrics` follows. Return inserts a line break.
    pub fn multiline(mut self) -> Self {
        self.multiline = true;
        self.submit_on_enter = false;
        self.metrics(TEXTAREA)
    }

    /// Size, padding and type; a multi-line field grows between `min_height` and `max_height`.
    pub fn metrics(mut self, metrics: FieldMetrics) -> Self {
        self.metrics = metrics;
        self.padding = padding(&metrics);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// The text the field starts with (React's `defaultValue`).
    pub fn default_value(mut self, text: impl Into<String>) -> Self {
        self.editing = Editing::new(text);
        self
    }

    /// Whether Return submits a multi-line field instead of breaking the line.
    pub fn submit_on_enter(mut self, submit: bool) -> Self {
        self.submit_on_enter = submit;
        self
    }

    /// The accessible name (`aria-label`), used when there is no visible label.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn text(&self) -> &str {
        self.editing.text()
    }

    pub fn is_empty(&self) -> bool {
        self.editing.is_empty()
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn is_multiline(&self) -> bool {
        self.multiline
    }

    pub fn field_metrics(&self) -> FieldMetrics {
        self.metrics
    }

    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus.is_focused(window)
    }

    /// Replaces the text, leaving the caret at its end.
    pub fn set_text(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        self.editing.set_text(text);
        self.reveal_caret = true;
        cx.emit(TextFieldEvent::Change);
        cx.notify();
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        cx.notify();
    }

    /// Puts the keyboard here.
    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        if !self.disabled {
            window.focus(&self.focus, cx);
        }
    }

    /// Insets the text from the field's edges (top, right, bottom, left); chrome around the
    /// field uses it where an addon sits beside the text.
    pub fn set_padding(&mut self, padding: [f32; 4]) {
        self.padding = padding;
    }

    fn edited(&mut self, cx: &mut Context<Self>) {
        self.reveal_caret = true;
        self.wake_caret();
        cx.emit(TextFieldEvent::Change);
        cx.notify();
    }

    fn moved(&mut self, cx: &mut Context<Self>) {
        self.reveal_caret = true;
        self.wake_caret();
        cx.notify();
    }

    fn wake_caret(&mut self) {
        self.caret_on = true;
        self.blink = None;
    }

    fn motion(&mut self, motion: Motion, extend: bool, cx: &mut Context<Self>) {
        self.editing.apply(motion, extend);
        self.moved(cx);
    }

    fn vertical(&mut self, rows: f32, extend: bool, cx: &mut Context<Self>) {
        let Some(layout) = self.layout.as_ref() else {
            return;
        };
        let caret = layout.position_for(self.editing.cursor());
        let y = caret.y + layout.line_height * (rows + 0.5);
        let target = if y < layout.origin.y {
            0
        } else if y >= layout.origin.y + layout.content_height {
            self.editing.text().len()
        } else {
            layout.index_for(point(caret.x, y))
        };
        if extend {
            self.editing.select_to(target);
        } else {
            self.editing.move_to(target);
        }
        self.moved(cx);
    }

    fn insert(&mut self, text: &str, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let text = if self.multiline {
            text.to_owned()
        } else {
            text.replace(['\r', '\n'], " ")
        };
        self.editing.insert(&text);
        self.edited(cx);
    }

    fn delete(&mut self, motion: Motion, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        if self.editing.delete_towards(motion) {
            self.edited(cx);
        } else {
            window.play_system_bell();
        }
    }

    fn enter(&mut self, newline: bool, cx: &mut Context<Self>) {
        if self.multiline && newline {
            self.insert("\n", cx);
        } else {
            cx.emit(TextFieldEvent::Submit);
        }
    }

    fn copy_selection(&self, cx: &mut Context<Self>) {
        let selected = self.editing.selected_text();
        if !selected.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(selected.to_owned()));
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        window.focus(&self.focus, cx);
        let Some(index) = self.layout.as_ref().map(|l| l.index_for(event.position)) else {
            return;
        };
        match event.click_count {
            2 => self.editing.select_word_at(index),
            n if n >= 3 => self.editing.select_all(),
            _ if event.modifiers.shift => self.editing.select_to(index),
            _ => self.editing.move_to(index),
        }
        self.selecting = event.click_count < 2;
        self.moved(cx);
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selecting {
            return;
        }
        if let Some(index) = self.layout.as_ref().map(|l| l.index_for(event.position)) {
            self.editing.select_to(index);
            self.moved(cx);
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.selecting = false;
    }

    fn on_scroll(&mut self, event: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let delta = event.delta.pixel_delta(px(self.metrics.line_height));
        if self.multiline {
            self.scroll.y -= delta.y;
        } else {
            self.scroll.x -= delta.x + delta.y;
        }
        cx.notify();
    }

    fn keep_blinking(&mut self, focused: bool, cx: &mut Context<Self>) {
        if !focused {
            self.blink = None;
            self.caret_on = true;
            return;
        }
        if self.blink.is_some() {
            return;
        }
        self.blink = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(CARET_BLINK).await;
                let alive = this.update(cx, |field, cx| {
                    field.caret_on = !field.caret_on;
                    cx.notify();
                });
                if alive.is_err() {
                    return;
                }
            }
        }));
    }
}

fn padding(metrics: &FieldMetrics) -> [f32; 4] {
    [
        metrics.padding_y,
        metrics.padding_x,
        metrics.padding_y,
        metrics.padding_x,
    ]
}

impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus.is_focused(window);
        self.keep_blinking(focused, cx);
        let [top, right, bottom, left] = self.padding;
        div()
            .id("text-field")
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .tab_stop(!self.disabled)
            .role(if self.multiline {
                Role::MultilineTextInput
            } else {
                Role::TextInput
            })
            .aria_label(
                self.label
                    .clone()
                    .unwrap_or_else(|| self.placeholder.clone()),
            )
            .w_full()
            .min_w_0()
            .pt(px(top))
            .pr(px(right))
            .pb(px(bottom))
            .pl(px(left))
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::IBeam
            })
            .on_action(cx.listener(|f, _: &Backspace, w, cx| f.delete(Motion::Left, w, cx)))
            .on_action(cx.listener(|f, _: &Delete, w, cx| f.delete(Motion::Right, w, cx)))
            .on_action(
                cx.listener(|f, _: &DeleteWordLeft, w, cx| f.delete(Motion::WordLeft, w, cx)),
            )
            .on_action(
                cx.listener(|f, _: &DeleteWordRight, w, cx| f.delete(Motion::WordRight, w, cx)),
            )
            .on_action(cx.listener(|f, _: &Left, _, cx| f.motion(Motion::Left, false, cx)))
            .on_action(cx.listener(|f, _: &Right, _, cx| f.motion(Motion::Right, false, cx)))
            .on_action(cx.listener(|f, _: &WordLeft, _, cx| f.motion(Motion::WordLeft, false, cx)))
            .on_action(
                cx.listener(|f, _: &WordRight, _, cx| f.motion(Motion::WordRight, false, cx)),
            )
            .on_action(cx.listener(|f, _: &Home, _, cx| f.motion(Motion::LineStart, false, cx)))
            .on_action(cx.listener(|f, _: &End, _, cx| f.motion(Motion::LineEnd, false, cx)))
            .on_action(cx.listener(|f, _: &DocStart, _, cx| f.motion(Motion::Start, false, cx)))
            .on_action(cx.listener(|f, _: &DocEnd, _, cx| f.motion(Motion::End, false, cx)))
            .on_action(cx.listener(|f, _: &SelectLeft, _, cx| f.motion(Motion::Left, true, cx)))
            .on_action(cx.listener(|f, _: &SelectRight, _, cx| f.motion(Motion::Right, true, cx)))
            .on_action(
                cx.listener(|f, _: &SelectWordLeft, _, cx| f.motion(Motion::WordLeft, true, cx)),
            )
            .on_action(
                cx.listener(|f, _: &SelectWordRight, _, cx| f.motion(Motion::WordRight, true, cx)),
            )
            .on_action(
                cx.listener(|f, _: &SelectHome, _, cx| f.motion(Motion::LineStart, true, cx)),
            )
            .on_action(cx.listener(|f, _: &SelectEnd, _, cx| f.motion(Motion::LineEnd, true, cx)))
            .on_action(
                cx.listener(|f, _: &SelectDocStart, _, cx| f.motion(Motion::Start, true, cx)),
            )
            .on_action(cx.listener(|f, _: &SelectDocEnd, _, cx| f.motion(Motion::End, true, cx)))
            .on_action(cx.listener(|f, _: &Up, _, cx| f.vertical(-1.0, false, cx)))
            .on_action(cx.listener(|f, _: &Down, _, cx| f.vertical(1.0, false, cx)))
            .on_action(cx.listener(|f, _: &SelectUp, _, cx| f.vertical(-1.0, true, cx)))
            .on_action(cx.listener(|f, _: &SelectDown, _, cx| f.vertical(1.0, true, cx)))
            .on_action(cx.listener(|f, _: &SelectAll, _, cx| {
                f.editing.select_all();
                f.moved(cx);
            }))
            .on_action(cx.listener(|f, _: &Copy, _, cx| f.copy_selection(cx)))
            .on_action(cx.listener(|f, _: &Cut, w, cx| {
                f.copy_selection(cx);
                if !f.editing.selection().is_empty() {
                    f.delete(Motion::Left, w, cx);
                }
            }))
            .on_action(cx.listener(|f, _: &Paste, _, cx| {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    f.insert(&text, cx);
                }
            }))
            .on_action(cx.listener(|f, _: &Enter, _, cx| {
                let newline = !f.submit_on_enter;
                f.enter(newline, cx);
            }))
            .on_action(cx.listener(|f, _: &Newline, _, cx| f.enter(true, cx)))
            .on_action(cx.listener(|_, _: &Submit, _, cx| cx.emit(TextFieldEvent::Submit)))
            .on_action(cx.listener(|_, _: &ShowCharacterPalette, w, _| w.show_character_palette()))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_scroll_wheel(cx.listener(Self::on_scroll))
            .child(TextElement { field: cx.entity() })
    }
}

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.editing.range_from_utf16(&range_utf16);
        actual_range.replace(self.editing.range_to_utf16(&range));
        self.editing.text().get(range).map(str::to_owned)
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.editing.range_to_utf16(&self.editing.selection()),
            reversed: self.editing.is_reversed(),
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.editing
            .marked()
            .map(|range| self.editing.range_to_utf16(&range))
    }

    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.editing.unmark();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        if let Some(range) = range_utf16 {
            let range = self.editing.range_from_utf16(&range);
            self.editing.set_selection(range, false);
        }
        self.insert(text, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        selected_utf16: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let range = range_utf16.map(|r| self.editing.range_from_utf16(&r));
        let selected =
            selected_utf16.map(|r| utf16_to_byte(text, r.start)..utf16_to_byte(text, r.end));
        self.editing.replace_and_mark(range, text, selected);
        self.edited(cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let layout = self.layout.as_ref()?;
        let range = self.editing.range_from_utf16(&range_utf16);
        let start = layout.position_for(range.start);
        let end = layout.position_for(range.end);
        let width = if end.y == start.y {
            end.x - start.x
        } else {
            px(0.)
        };
        Some(Bounds::new(start, size(width, layout.line_height)))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let index = self.layout.as_ref()?.index_for(point);
        Some(self.editing.offset_to_utf16(index))
    }
}

fn utf16_to_byte(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (i, ch) in text.char_indices() {
        if units >= utf16 {
            return i;
        }
        units += ch.len_utf16();
    }
    text.len()
}

/// Where the last frame put the text, for hit tests, IME and caret motion.
struct Layout {
    lines: Vec<WrappedLine>,
    starts: Vec<usize>,
    tops: Vec<Pixels>,
    origin: Point<Pixels>,
    line_height: Pixels,
    content_height: Pixels,
    /// Whether the lines show the placeholder rather than the text.
    placeholder: bool,
}

impl Layout {
    fn line_of(&self, offset: usize) -> usize {
        self.starts
            .partition_point(|&start| start <= offset)
            .saturating_sub(1)
    }

    fn position_for(&self, offset: usize) -> Point<Pixels> {
        if self.placeholder {
            return self.origin;
        }
        let i = self.line_of(offset);
        let (Some(line), Some(&start), Some(&top)) =
            (self.lines.get(i), self.starts.get(i), self.tops.get(i))
        else {
            return self.origin;
        };
        let local = offset.saturating_sub(start).min(line.len());
        let at = line
            .position_for_index(local, self.line_height)
            .unwrap_or(point(line.width(), px(0.)));
        self.origin + point(at.x, top + at.y)
    }

    fn index_for(&self, position: Point<Pixels>) -> usize {
        if self.placeholder {
            return 0;
        }
        let local = position - self.origin;
        let i = self
            .tops
            .partition_point(|&top| top <= local.y)
            .saturating_sub(1);
        let (Some(line), Some(&start), Some(&top)) =
            (self.lines.get(i), self.starts.get(i), self.tops.get(i))
        else {
            return 0;
        };
        let height = line.size(self.line_height).height;
        let y = (local.y - top).clamp(px(0.), (height - px(1.)).max(px(0.)));
        let x = local.x.max(px(0.));
        let (Ok(index) | Err(index)) =
            line.closest_index_for_position(point(x, y), self.line_height);
        start + index.min(line.len())
    }

    /// The highlight rectangles of a selection, one per visual row it touches.
    fn selection_rects(&self, range: Range<usize>, right: Pixels) -> Vec<Bounds<Pixels>> {
        let mut rects = Vec::new();
        if range.is_empty() || self.placeholder {
            return rects;
        }
        let lh = self.line_height;
        for (i, line) in self.lines.iter().enumerate() {
            let start = self.starts[i];
            let end = start + line.len();
            let (a, b) = (range.start.max(start), range.end.min(end));
            let newline = range.end > end && range.start <= end;
            if a > b || (a == b && !newline) {
                continue;
            }
            let p1 = self.position_for(a);
            let mut p2 = self.position_for(b);
            if newline {
                p2.x += px(4.);
            }
            if p1.y == p2.y {
                rects.push(Bounds::from_corners(p1, point(p2.x, p2.y + lh)));
                continue;
            }
            rects.push(Bounds::from_corners(p1, point(right, p1.y + lh)));
            if p2.y > p1.y + lh {
                rects.push(Bounds::from_corners(
                    point(self.origin.x, p1.y + lh),
                    point(right, p2.y),
                ));
            }
            rects.push(Bounds::from_corners(
                point(self.origin.x, p2.y),
                point(p2.x, p2.y + lh),
            ));
        }
        rects
    }
}

/// Shapes the visible text (or placeholder) into wrapped lines.
fn shape(
    field: &TextField,
    color: Hsla,
    placeholder_color: Hsla,
    wrap_width: Option<Pixels>,
    window: &Window,
) -> (Vec<WrappedLine>, bool) {
    let placeholder = field.editing.is_empty();
    let (text, color) = if placeholder {
        (field.placeholder.clone(), placeholder_color)
    } else {
        (SharedString::from(field.editing.text().to_owned()), color)
    };
    let mut font = ::gpui::font(fonts::SANS);
    font.weight = FontWeight::NORMAL;
    let run = TextRun {
        len: text.len(),
        font,
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let runs = match field.editing.marked().filter(|_| !placeholder) {
        Some(marked) => vec![
            TextRun {
                len: marked.start,
                ..run.clone()
            },
            TextRun {
                len: marked.len(),
                underline: Some(UnderlineStyle {
                    color: Some(color),
                    thickness: px(1.),
                    wavy: false,
                }),
                ..run.clone()
            },
            TextRun {
                len: text.len() - marked.end,
                ..run
            },
        ],
        None => vec![run],
    };
    let wrap = if field.multiline { wrap_width } else { None };
    let lines = window
        .text_system()
        .shape_text(text, px(field.metrics.font_size), &runs, wrap, None)
        .map(|lines| lines.into_vec())
        .unwrap_or_default();
    (lines, placeholder)
}

fn rows(lines: &[WrappedLine]) -> usize {
    lines
        .iter()
        .map(|line| line.wrap_boundaries().len() + 1)
        .sum::<usize>()
        .max(1)
}

struct TextElement {
    field: Entity<TextField>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

struct Prepainted {
    selection: Vec<Bounds<Pixels>>,
    caret: Option<Bounds<Pixels>>,
}

impl ::gpui::Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = Prepainted;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        _: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        let field = self.field.clone();
        let id = window.request_measured_layout(style, move |known, available, window, cx| {
            let field = field.read(cx);
            let width = known.width.unwrap_or(match available.width {
                ::gpui::AvailableSpace::Definite(width) => width,
                _ => px(0.),
            });
            let lines = if field.multiline {
                let (lines, _) =
                    shape(field, Hsla::default(), Hsla::default(), Some(width), window);
                rows(&lines)
            } else {
                1
            };
            size(width, px(field.metrics.content_height(lines)))
        });
        (id, ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Prepainted {
        let theme = *cx.notra();
        let skin = FieldSkin::new(&theme);
        let focused = self.field.read(cx).focus.is_focused(window);
        self.field.update(cx, |field, _| {
            let (lines, placeholder) = shape(
                field,
                skin.text.into(),
                skin.placeholder.into(),
                Some(bounds.size.width),
                window,
            );
            let lh = px(field.metrics.line_height);
            let mut starts = Vec::with_capacity(lines.len());
            let mut tops = Vec::with_capacity(lines.len());
            let (mut offset, mut top) = (0, px(0.));
            for line in &lines {
                starts.push(offset);
                tops.push(top);
                offset += line.len() + 1;
                top += line.size(lh).height.max(lh);
            }
            let content_height = top.max(lh);
            let width = lines.iter().map(|l| l.width()).fold(px(0.), Pixels::max);
            let mut layout = Layout {
                lines,
                starts,
                tops,
                origin: bounds.origin,
                line_height: lh,
                content_height,
                placeholder,
            };
            if field.multiline {
                if field.reveal_caret {
                    let caret = layout.position_for(field.editing.cursor()) - bounds.origin;
                    if caret.y < field.scroll.y {
                        field.scroll.y = caret.y;
                    } else if caret.y + lh > field.scroll.y + bounds.size.height {
                        field.scroll.y = caret.y + lh - bounds.size.height;
                    }
                }
                let max = (content_height - bounds.size.height).max(px(0.));
                field.scroll.y = field.scroll.y.clamp(px(0.), max);
                field.scroll.x = px(0.);
                layout.origin.y -= field.scroll.y;
            } else {
                if field.reveal_caret {
                    let caret = layout.position_for(field.editing.cursor()).x - bounds.origin.x;
                    let room = bounds.size.width - px(CARET_WIDTH);
                    if caret < field.scroll.x {
                        field.scroll.x = caret;
                    } else if caret > field.scroll.x + room {
                        field.scroll.x = caret - room;
                    }
                }
                let max = (width + px(CARET_WIDTH) - bounds.size.width).max(px(0.));
                field.scroll.x = field.scroll.x.clamp(px(0.), max);
                field.scroll.y = px(0.);
                layout.origin.x -= field.scroll.x;
                layout.origin.y += (bounds.size.height - lh) / 2.;
            }
            field.reveal_caret = false;

            let selection = layout.selection_rects(field.editing.selection(), bounds.right());
            let caret = (focused
                && field.editing.selection().is_empty()
                && field.caret_on
                && !field.disabled)
                .then(|| {
                    let at = layout.position_for(field.editing.cursor());
                    let height = px(field.metrics.font_size * 1.2).min(lh);
                    Bounds::new(
                        point(at.x.floor(), at.y + (lh - height) / 2.),
                        size(px(CARET_WIDTH), height),
                    )
                });
            field.layout = Some(layout);
            Prepainted { selection, caret }
        })
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        prepainted: &mut Prepainted,
        window: &mut Window,
        cx: &mut App,
    ) {
        let theme = *cx.notra();
        let skin = FieldSkin::new(&theme);
        let (focus, disabled) = {
            let field = self.field.read(cx);
            (field.focus.clone(), field.disabled)
        };
        if !disabled {
            window.handle_input(
                &focus,
                ElementInputHandler::new(bounds, self.field.clone()),
                cx,
            );
        }
        let layout = self.field.update(cx, |field, _| field.layout.take());
        let Some(layout) = layout else {
            return;
        };
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for rect in prepainted.selection.drain(..) {
                window.paint_quad(fill(rect, skin.selection));
            }
            for (line, top) in layout.lines.iter().zip(&layout.tops) {
                let origin = layout.origin + point(px(0.), *top);
                if line
                    .paint(
                        origin,
                        layout.line_height,
                        TextAlign::Left,
                        None,
                        window,
                        cx,
                    )
                    .is_err()
                {
                    break;
                }
            }
            if let Some(caret) = prepainted.caret.take() {
                window.paint_quad(fill(caret, skin.caret));
            }
        });
        self.field
            .update(cx, |field, _| field.layout = Some(layout));
    }
}

/// Focus progress of a field's box, for the 150 ms border transition.
struct FocusState {
    focus: Animated,
}

fn field_box(
    id: ElementId,
    field: &Entity<TextField>,
    radius: f32,
    window: &mut Window,
    cx: &mut App,
) -> ::gpui::Stateful<::gpui::Div> {
    let theme = *cx.notra();
    let skin = FieldSkin::new(&theme);
    let (focused, disabled) = {
        let f = field.read(cx);
        (f.is_focused(window), f.disabled)
    };
    let now = Instant::now();
    let state = window.use_keyed_state(id.clone(), cx, |_, _| FocusState {
        focus: Animated::new(if focused { 1.0 } else { 0.0 }, TRANSITION),
    });
    state.update(cx, |s, _| s.focus.set(if focused { 1.0 } else { 0.0 }, now));
    let s = state.read(cx);
    let t = if cx.reduce_motion() {
        s.focus.target()
    } else {
        s.focus.value(now)
    };
    if s.focus.is_animating(now) && !cx.reduce_motion() {
        window.request_animation_frame();
    }
    let border = lerp_color(skin.border, skin.focus_border, t);
    div()
        .id(id)
        .relative()
        .w_full()
        .min_w_0()
        .rounded(px(radius))
        .border_1()
        .border_color(border)
        .bg(if disabled {
            skin.disabled_background
        } else {
            skin.background
        })
        .when(focused, |d| {
            let spread = skin.focus_ring.spread;
            d.child(
                div()
                    .absolute()
                    .top(px(-spread - 1.0))
                    .left(px(-spread - 1.0))
                    .right(px(-spread - 1.0))
                    .bottom(px(-spread - 1.0))
                    .rounded(px(radius + spread))
                    .border(px(spread))
                    .border_color(skin.focus_ring.color),
            )
        })
        .when(disabled, |d| {
            d.opacity(DISABLED_OPACITY)
                .cursor(CursorStyle::OperationNotAllowed)
        })
}

pub(crate) fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    if t <= 0.0 {
        a
    } else if t >= 1.0 {
        b
    } else {
        a.mix(b, t)
    }
}

/// shadcn's `Input`: a bordered box around a single-line [`TextField`].
#[derive(IntoElement)]
pub struct Input {
    field: Entity<TextField>,
}

impl Input {
    pub fn new(field: Entity<TextField>) -> Self {
        Self { field }
    }
}

impl RenderOnce for Input {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let radius = self.field.read(cx).metrics.radius;
        let id = ElementId::from(("notra-input", self.field.entity_id()));
        field_box(id, &self.field, radius, window, cx).child(self.field)
    }
}

/// shadcn's `Textarea`: a bordered box around a multi-line [`TextField`] that grows with its
/// text.
#[derive(IntoElement)]
pub struct Textarea {
    field: Entity<TextField>,
}

impl Textarea {
    pub fn new(field: Entity<TextField>) -> Self {
        Self { field }
    }
}

impl RenderOnce for Textarea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let radius = self.field.read(cx).metrics.radius;
        let id = ElementId::from(("notra-textarea", self.field.entity_id()));
        field_box(id, &self.field, radius, window, cx).child(self.field)
    }
}

/// A field with icons, text or buttons inside its box (shadcn's `InputGroup`).
///
/// ```ignore
/// InputGroup::new(field)
///     .addon(InputGroupAddon::new(AddonAlign::InlineStart).icon(Icon::Search))
///     .addon(InputGroupAddon::new(AddonAlign::InlineEnd).text("12 results"))
/// ```
#[derive(IntoElement)]
pub struct InputGroup {
    field: Entity<TextField>,
    addons: Vec<InputGroupAddon>,
}

impl InputGroup {
    pub fn new(field: Entity<TextField>) -> Self {
        Self {
            field,
            addons: Vec::new(),
        }
    }

    pub fn addon(mut self, addon: InputGroupAddon) -> Self {
        self.addons.push(addon);
        self
    }
}

impl RenderOnce for InputGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let has = |align: AddonAlign| self.addons.iter().any(|a| a.align == align);
        let multiline = self.field.read(cx).multiline;
        let column = multiline || has(AddonAlign::BlockStart) || has(AddonAlign::BlockEnd);
        let metrics = self.field.read(cx).metrics;
        let inset = group::INPUT_INSET_BESIDE_ADDON;
        let padding = [
            if has(AddonAlign::BlockStart) && !multiline {
                group::BLOCK_ADDON_PADDING_Y
            } else {
                metrics.padding_y
            },
            if has(AddonAlign::InlineEnd) {
                inset
            } else {
                metrics.padding_x
            },
            if has(AddonAlign::BlockEnd) && !multiline {
                group::BLOCK_ADDON_PADDING_Y + metrics.padding_y
            } else {
                metrics.padding_y
            },
            if has(AddonAlign::InlineStart) {
                inset
            } else {
                metrics.padding_x
            },
        ];
        self.field.update(cx, |f, _| f.set_padding(padding));
        let focus = self.field.read(cx).focus.clone();
        let id = ElementId::from(("notra-input-group", self.field.entity_id()));
        let mut addons = self.addons;
        addons.sort_by_key(|a| match a.align {
            AddonAlign::InlineStart | AddonAlign::BlockStart => 0,
            AddonAlign::InlineEnd | AddonAlign::BlockEnd => 2,
        });
        let (before, after): (Vec<_>, Vec<_>) = addons
            .into_iter()
            .partition(|a| matches!(a.align, AddonAlign::InlineStart | AddonAlign::BlockStart));
        let field = self.field;
        field_box(id, &field, group::RADIUS, window, cx)
            .flex()
            .when(column, |d| d.flex_col())
            .when(!column, |d| d.items_center().h(px(group::HEIGHT)))
            .children(before.into_iter().map(|a| a.focus_target(focus.clone())))
            .child(div().flex_1().min_w_0().w_full().child(field))
            .children(after.into_iter().map(|a| a.focus_target(focus.clone())))
    }
}

/// Text, icons or buttons beside the text of an [`InputGroup`]; clicking it focuses the field.
#[derive(IntoElement)]
pub struct InputGroupAddon {
    align: AddonAlign,
    icons: Vec<Icon>,
    children: Vec<AnyElement>,
    holds_button: bool,
    focus: Option<FocusHandle>,
}

impl InputGroupAddon {
    pub fn new(align: AddonAlign) -> Self {
        Self {
            align,
            icons: Vec::new(),
            children: Vec::new(),
            holds_button: false,
            focus: None,
        }
    }

    /// An icon before the addon's other contents.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icons.push(icon);
        self
    }

    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.children
            .push(div().child(text.into()).into_any_element());
        self
    }

    /// An [`InputGroupButton`]; the addon tucks into the group's edge to hold it.
    pub fn button(mut self, button: InputGroupButton) -> Self {
        self.holds_button = true;
        self.children.push(button.into_any_element());
        self
    }

    fn focus_target(mut self, focus: FocusHandle) -> Self {
        self.focus = Some(focus);
        self
    }
}

impl ParentElement for InputGroupAddon {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for InputGroupAddon {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let pull = if self.holds_button {
            group::BUTTON_PULL
        } else {
            0.0
        };
        let focus = self.focus;
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(group::ADDON_GAP))
            .py(px(6.))
            .text_size(px(14.))
            .line_height(px(20.))
            .font_weight(FontWeight(500.0))
            .text_color(theme.muted_foreground)
            .cursor(CursorStyle::IBeam)
            .map(|d| match self.align {
                AddonAlign::InlineStart => d.pl(px(group::ADDON_INSET)).ml(px(-pull)),
                AddonAlign::InlineEnd => d.pr(px(group::ADDON_INSET)).mr(px(-pull)),
                AddonAlign::BlockStart => d
                    .w_full()
                    .justify_start()
                    .px(px(group::BLOCK_ADDON_PADDING_X))
                    .pt(px(group::BLOCK_ADDON_PADDING_Y)),
                AddonAlign::BlockEnd => d
                    .w_full()
                    .justify_start()
                    .px(px(group::BLOCK_ADDON_PADDING_X))
                    .pb(px(group::BLOCK_ADDON_PADDING_Y)),
            })
            .when_some(focus, |d, focus| {
                d.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.focus(&focus, cx);
                })
            })
            .children(self.icons.into_iter().map(|icon| {
                svg()
                    .path(icon.path())
                    .size(px(group::ADDON_ICON))
                    .flex_none()
                    .text_color(theme.muted_foreground)
            }))
            .children(self.children)
    }
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Box metrics of a ghost button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GhostMetrics {
    pub height: f32,
    /// `None` for a square icon button.
    pub padding_x: Option<f32>,
    pub gap: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub icon: f32,
}

/// A ghost button with Notra's hover fill, sized by the caller: the toolbar buttons, chip
/// actions and input-group buttons all restyle the ghost `Button` with classes.
#[derive(IntoElement)]
pub(crate) struct GhostButton {
    id: ElementId,
    metrics: GhostMetrics,
    icon: Option<Icon>,
    label: Option<SharedString>,
    icon_end: Option<(Icon, f32)>,
    text: Color,
    hover_fill: Color,
    hover_text: Color,
    disabled: bool,
    aria_label: Option<SharedString>,
    on_click: Option<ClickHandler>,
}

impl GhostButton {
    pub fn new(
        id: impl Into<ElementId>,
        metrics: GhostMetrics,
        text: Color,
        hover: (Color, Color),
    ) -> Self {
        Self {
            id: id.into(),
            metrics,
            icon: None,
            label: None,
            icon_end: None,
            text,
            hover_fill: hover.0,
            hover_text: hover.1,
            disabled: false,
            aria_label: None,
            on_click: None,
        }
    }

    pub fn icon(mut self, icon: Option<Icon>) -> Self {
        self.icon = icon;
        self
    }

    pub fn label(mut self, label: Option<SharedString>) -> Self {
        self.label = label;
        self
    }

    /// A trailing icon at its own size (a `size-3` chevron).
    pub fn icon_end(mut self, icon: Option<(Icon, f32)>) -> Self {
        self.icon_end = icon;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The accessible name of an icon-only button.
    pub fn aria_label(mut self, label: Option<SharedString>) -> Self {
        self.aria_label = label;
        self
    }

    pub fn on_click(mut self, handler: Option<ClickHandler>) -> Self {
        self.on_click = handler;
        self
    }
}

/// A hover track that outlives a frame.
pub(crate) struct HoverState {
    hover: Animated,
}

/// Samples a hover track, requesting frames while it moves.
pub(crate) fn hover_value(
    id: ElementId,
    tween: Tween,
    window: &mut Window,
    cx: &mut App,
) -> (f32, Entity<HoverState>) {
    let now = Instant::now();
    let state = window.use_keyed_state(id, cx, |_, _| HoverState {
        hover: Animated::new(0.0, tween),
    });
    let s = state.read(cx);
    let value = if cx.reduce_motion() {
        s.hover.target()
    } else {
        s.hover.value(now)
    };
    if s.hover.is_animating(now) && !cx.reduce_motion() {
        window.request_animation_frame();
    }
    (value, state)
}

/// Wires `on_hover` of an element to a [`hover_value`] track.
pub(crate) fn track_hover(
    state: Entity<HoverState>,
) -> impl Fn(&bool, &mut Window, &mut App) + 'static {
    move |hovered, _, cx| {
        let target = if *hovered { 1.0 } else { 0.0 };
        state.update(cx, |s, cx| {
            s.hover.set(target, Instant::now());
            cx.notify();
        });
    }
}

impl RenderOnce for GhostButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let m = self.metrics;
        let interactive = !self.disabled;
        let (hover, state) = hover_value(self.id.clone(), TRANSITION, window, cx);
        let hover = if interactive { hover } else { 0.0 };
        let fill_color = Color::TRANSPARENT.mix(self.hover_fill, hover);
        let text: Hsla = lerp_color(self.text, self.hover_text, hover).into();
        let icon = |icon: Icon, size: f32| {
            svg()
                .path(icon.path())
                .size(px(size))
                .flex_none()
                .text_color(text)
        };
        let height = px(m.height);
        div()
            .id(self.id)
            .role(Role::Button)
            .when_some(self.aria_label.or(self.label.clone()), |d, label| {
                d.aria_label(label)
            })
            .tab_stop(interactive)
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(m.gap))
            .h(height)
            .map(|d| match m.padding_x {
                Some(p) => d.px(px(p)),
                None => d.w(height),
            })
            .rounded(px(m.radius))
            .border_1()
            .border_color(Color::TRANSPARENT)
            .bg(fill_color)
            .text_color(text)
            .text_size(px(m.font_size))
            .line_height(px(m.line_height))
            .font_weight(FontWeight(500.0))
            .whitespace_nowrap()
            .focus_visible(move |style| {
                style
                    .border_color(theme.ring)
                    .shadow(shadows(&[Shadow::drop(
                        0.0,
                        0.0,
                        3.0,
                        theme.ring.fade(0.5),
                    )]))
            })
            .when(self.disabled, |d| {
                d.opacity(DISABLED_OPACITY)
                    .cursor(CursorStyle::OperationNotAllowed)
            })
            .when(interactive, |d| {
                d.cursor_pointer()
                    .on_hover(track_hover(state))
                    .when_some(self.on_click, |d, handler| d.on_click(handler))
            })
            .when_some(self.icon, |d, i| d.child(icon(i, m.icon)))
            .when_some(self.label, |d, label| d.child(label))
            .when_some(self.icon_end, |d, (i, size)| d.child(icon(i, size)))
    }
}

/// A ghost button inside an [`InputGroupAddon`] (`InputGroupButton size="xs"`, or an icon).
#[derive(IntoElement)]
pub struct InputGroupButton {
    id: ElementId,
    icon: Option<Icon>,
    label: Option<SharedString>,
    aria_label: Option<SharedString>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl InputGroupButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            label: Some(label.into()),
            aria_label: None,
            disabled: false,
            on_click: None,
        }
    }

    /// A square `icon-xs` button; name it with `aria_label`.
    pub fn icon_button(
        id: impl Into<ElementId>,
        icon: Icon,
        aria_label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            icon: Some(icon),
            label: None,
            aria_label: Some(aria_label.into()),
            disabled: false,
            on_click: None,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for InputGroupButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.notra();
        let hover_fill = if theme.is_dark() {
            theme.muted.fade(0.5)
        } else {
            theme.muted
        };
        let metrics = GhostMetrics {
            height: group::BUTTON_HEIGHT,
            padding_x: self.label.as_ref().map(|_| group::BUTTON_PADDING_X),
            gap: 4.0,
            radius: group::BUTTON_RADIUS,
            font_size: 14.0,
            line_height: 20.0,
            icon: group::BUTTON_ICON,
        };
        GhostButton::new(
            self.id,
            metrics,
            theme.foreground,
            (hover_fill, theme.foreground),
        )
        .icon(self.icon)
        .label(self.label)
        .aria_label(self.aria_label)
        .disabled(self.disabled)
        .on_click(self.on_click)
    }
}
