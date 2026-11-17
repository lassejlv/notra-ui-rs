//! Shared GPUI pieces of the terminal blocks: styled monospace text, the window
//! chrome with its scroll area, glyph spinners and a keyboard-driven prompt input.

use std::sync::Arc;
use std::time::Instant;

use ::gpui::{
    AnyElement, App, Context, Div, ElementId, Entity, EventEmitter, FocusHandle, FontFeatures,
    FontStyle, FontWeight, HighlightStyle, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, RenderOnce, ScrollHandle, SharedString, Stateful, StatefulInteractiveElement,
    StrikethroughStyle, Styled, StyledText, UnderlineStyle, Window, div, prelude::*, px,
};

use super::{ActiveTheme, shadows};
use crate::components::terminal::{self, ChromePalette, FrameCycle, Rich, chrome};
use crate::fonts;
use crate::motion::{Animated, Tween};
use crate::theme::Shadow;

/// Monospace without ligatures or contextual alternates: `--` stays two hyphens.
pub fn code_features() -> FontFeatures {
    FontFeatures(Arc::new(vec![("liga".into(), 0), ("calt".into(), 0)]))
}

/// Paints [`Rich`] runs over the parent's monospace text style.
pub fn rich_text(rich: &Rich) -> StyledText {
    let highlights = rich
        .ranges()
        .filter(|(range, _)| !range.is_empty())
        .map(|(range, run)| {
            let color = run.color.into();
            let style = HighlightStyle {
                color: Some(color),
                font_weight: run.bold.then_some(FontWeight::BOLD),
                font_style: run.italic.then_some(FontStyle::Italic),
                strikethrough: run.strike.then_some(StrikethroughStyle {
                    thickness: px(1.0),
                    color: Some(color),
                }),
                underline: run.underline.then_some(UnderlineStyle {
                    thickness: px(1.0),
                    color: Some(color),
                    wavy: false,
                }),
                ..Default::default()
            };
            (range, style)
        })
        .collect::<Vec<_>>();
    StyledText::new(rich.text()).with_highlights(highlights)
}

/// The window Claude Code and Codex draw: title bar, scrolling transcript and a
/// footer pinned under it. The body's padding and gaps belong to the caller.
#[derive(IntoElement)]
pub struct TerminalWindow {
    id: ElementId,
    palette: ChromePalette,
    title: Option<SharedString>,
    height: Option<f32>,
    font_size: f32,
    line_height: f32,
    body: Vec<AnyElement>,
    footer: Option<AnyElement>,
    scroll: Option<ScrollHandle>,
}

impl TerminalWindow {
    pub fn new(id: impl Into<ElementId>, palette: ChromePalette) -> Self {
        Self {
            id: id.into(),
            palette,
            title: None,
            height: None,
            font_size: 13.0,
            line_height: 20.0,
            body: Vec::new(),
            footer: None,
            scroll: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// A fixed height; the transcript scrolls inside it.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// The text style of the body and footer.
    pub fn text(mut self, font_size: f32, line_height: f32) -> Self {
        self.font_size = font_size;
        self.line_height = line_height;
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    /// Drives the transcript's scroll position, for following new output.
    pub fn track_scroll(mut self, handle: &ScrollHandle) -> Self {
        self.scroll = Some(handle.clone());
        self
    }
}

impl ParentElement for TerminalWindow {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(elements);
    }
}

impl RenderOnce for TerminalWindow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = self.palette;
        let scroll = self.scroll.unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "scroll"), cx, |_, _| ScrollHandle::new())
                .read(cx)
                .clone()
        });
        let lights = (0..3).map(|_| {
            div()
                .size(px(chrome::LIGHT_SIZE))
                .flex_none()
                .rounded_full()
                .bg(p.traffic)
        });
        let bar = div()
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .h(px(chrome::BAR_HEIGHT))
            .px(px(chrome::BAR_PADDING_X))
            .bg(p.title_bar)
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(px(chrome::BAR_PADDING_X))
                    .flex()
                    .items_center()
                    .gap(px(chrome::LIGHT_GAP))
                    .children(lights),
            )
            .when_some(self.title, |d, title| {
                d.child(
                    div()
                        .max_w(::gpui::relative(0.6))
                        .min_w_0()
                        .truncate()
                        .text_size(px(chrome::TITLE_SIZE))
                        .line_height(px(chrome::TITLE_LINE))
                        .text_color(p.title)
                        .child(title),
                )
            });
        let body = div()
            .id((self.id.clone(), "body"))
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .children(self.body);
        let thumb = ScrollThumb::new(scroll, cx.notra().border);
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .w_full()
            .min_w_0()
            .when_some(self.height, |d, h| d.h(px(h)))
            .overflow_hidden()
            .rounded(px(chrome::RADIUS))
            .bg(p.background)
            .shadow(shadows(&chrome::shadows(&p)))
            .font_family(fonts::MONO)
            .font_features(code_features())
            .text_size(px(self.font_size))
            .line_height(px(self.line_height))
            .child(bar)
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .child(body)
                    .child(thumb),
            )
            .children(self.footer)
    }
}

/// The scroll area's vertical thumb (`bg-border rounded-full` in a 10 px track),
/// shown while the content overflows.
#[derive(IntoElement)]
pub struct ScrollThumb {
    handle: ScrollHandle,
    color: crate::theme::Color,
}

impl ScrollThumb {
    pub fn new(handle: ScrollHandle, color: crate::theme::Color) -> Self {
        Self { handle, color }
    }
}

impl RenderOnce for ScrollThumb {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let viewport = f32::from(self.handle.bounds().size.height);
        let max = f32::from(self.handle.max_offset().y);
        let offset = -f32::from(self.handle.offset().y);
        let visible = max > 0.5 && viewport > 0.0;
        let content = viewport + max;
        let track = (viewport - 2.0).max(0.0);
        let length = (track * viewport / content.max(1.0)).max(18.0).min(track);
        let top = 1.0 + (track - length) * (offset / max.max(1.0)).clamp(0.0, 1.0);
        div()
            .absolute()
            .top_0()
            .bottom_0()
            .right_0()
            .w(px(10.0))
            .when(visible, |d| {
                d.child(
                    div()
                        .absolute()
                        .right(px(1.0))
                        .top(px(top))
                        .w(px(8.0))
                        .h(px(length))
                        .rounded_full()
                        .bg(self.color),
                )
            })
    }
}

/// A glyph spinner that steps through `frames` while it is on screen.
#[derive(IntoElement)]
pub struct FrameGlyph {
    id: ElementId,
    frames: &'static [&'static str],
    cycle: FrameCycle,
    paused: bool,
}

impl FrameGlyph {
    pub fn new(
        id: impl Into<ElementId>,
        frames: &'static [&'static str],
        cycle: FrameCycle,
    ) -> Self {
        Self {
            id: id.into(),
            frames,
            cycle,
            paused: false,
        }
    }

    /// Holds the first frame (screenshots, reduced motion).
    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = paused;
        self
    }
}

impl RenderOnce for FrameGlyph {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let started = *window
            .use_keyed_state(self.id.clone(), cx, |_, _| Instant::now())
            .read(cx);
        let still = self.paused || cx.reduce_motion();
        let frame = if still {
            0
        } else {
            window.request_animation_frame();
            self.cycle.frame(started.elapsed())
        };
        div()
            .id(self.id)
            .child(self.frames.get(frame).copied().unwrap_or_default())
    }
}

/// The opacity of an `animate-pulse` element keyed by `id`, counted from its first
/// frame; keeps frames coming while it pulses.
pub fn pulse_opacity(id: impl Into<ElementId>, window: &mut Window, cx: &mut App) -> f32 {
    if cx.reduce_motion() {
        return 1.0;
    }
    let started = *window
        .use_keyed_state(id.into(), cx, |_, _| Instant::now())
        .read(cx);
    window.request_animation_frame();
    terminal::pulse_opacity(started.elapsed())
}

/// A collapsible's open state, focus and eased hover, kept per element.
pub struct Disclosure {
    pub open: bool,
    pub focus: FocusHandle,
    pub hover: Animated,
}

/// The [`Disclosure`] of the collapsible keyed by `id`; `hover` eases its hover value.
pub fn disclosure(
    id: &ElementId,
    default_open: bool,
    hover: Tween,
    window: &mut Window,
    cx: &mut App,
) -> Entity<Disclosure> {
    window.use_keyed_state(id.clone(), cx, |_, cx| Disclosure {
        open: default_open,
        focus: cx.focus_handle(),
        hover: Animated::new(0.0, hover),
    })
}

/// A Base UI collapsible trigger: focusable, toggles on click, Enter and Space, with
/// `ring` drawn while focused from the keyboard.
pub fn disclosure_trigger(
    id: ElementId,
    state: &Entity<Disclosure>,
    ring: Shadow,
    radius: f32,
    cx: &App,
) -> Stateful<Div> {
    let focus = state.read(cx).focus.clone();
    let toggle = state.clone();
    let hover = state.clone();
    div()
        .id(id)
        .track_focus(&focus)
        .tab_stop(true)
        .rounded(px(radius))
        .cursor_pointer()
        .focus_visible(move |s| s.shadow(shadows(&[ring])))
        .on_hover(move |hovered, _, cx| {
            hover.update(cx, |s, cx| {
                s.hover
                    .set(if *hovered { 1.0 } else { 0.0 }, Instant::now());
                cx.notify();
            });
        })
        .on_click(move |_, _, cx| {
            toggle.update(cx, |s, cx| {
                s.open = !s.open;
                cx.notify();
            });
        })
}

/// What a [`PromptInput`] reports to the view that owns it.
#[derive(Clone, Debug, PartialEq)]
pub enum PromptEvent {
    /// Enter with text: the trimmed message.
    Submit(String),
    /// Escape.
    Escape,
    /// Shift+Tab.
    CycleMode,
}

/// A single-line text field driven by key presses, for the terminals' prompts.
/// The family components draw it; this holds the text and the focus.
pub struct PromptInput {
    focus: FocusHandle,
    text: String,
}

impl EventEmitter<PromptEvent> for PromptInput {}

impl PromptInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            text: String::new(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.text.clear();
        cx.notify();
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus
    }

    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus.is_focused(window)
    }

    fn key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        let modifiers = keystroke.modifiers;
        match keystroke.key.as_str() {
            "enter" => {
                let message = self.text.trim().to_owned();
                if !message.is_empty() {
                    cx.emit(PromptEvent::Submit(message));
                }
            }
            "escape" => cx.emit(PromptEvent::Escape),
            "tab" if modifiers.shift => cx.emit(PromptEvent::CycleMode),
            "backspace" => {
                self.text.pop();
            }
            _ if modifiers.control || modifiers.platform || modifiers.function => {}
            _ => {
                if let Some(typed) = keystroke.key_char.as_deref() {
                    self.text.extend(typed.chars().filter(|c| !c.is_control()));
                }
            }
        }
        cx.notify();
    }
}

/// Makes `element` the field of `input`: it takes focus on click and types into it.
pub fn bind_prompt<E: InteractiveElement>(
    element: E,
    input: &::gpui::Entity<PromptInput>,
    cx: &App,
) -> E {
    let focus = input.read(cx).focus.clone();
    let typing = input.clone();
    let clicked = focus.clone();
    element
        .track_focus(&focus)
        .on_key_down(move |event, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
            typing.update(cx, |input, cx| input.key_down(event, cx));
        })
        .on_mouse_down(::gpui::MouseButton::Left, move |_, window, cx| {
            clicked.focus(window, cx);
        })
}
