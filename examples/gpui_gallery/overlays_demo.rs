//! The plain shadcn buttons the live overlay examples use as triggers
//! (`@/components/ui/button`, base-nova): `variant="outline"` and `variant="ghost" size="icon-sm"`.

use gpui::{
    App, Div, ElementId, FontWeight, InteractiveElement, ParentElement, SharedString, Stateful,
    Styled, div, px, svg,
};
use notra_ui::Icon;
use notra_ui::fonts;
use notra_ui::gpui::{ActiveTheme, shadows};
use notra_ui::theme::Shadow;

/// `<Button variant="outline">`: 32 px tall, 10 px in, 8 px corners; `bg-muted` on hover
/// and while its popup is open (`aria-expanded`).
pub fn outline_button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    expanded: bool,
    cx: &App,
) -> Stateful<Div> {
    let t = *cx.notra();
    let (fill, hover, border) = if t.is_dark() {
        (t.input.fade(0.3), t.input.fade(0.5), t.input)
    } else {
        (t.background, t.muted, t.border)
    };
    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .gap(px(6.0))
        .h(px(32.0))
        .px(px(10.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(border)
        .bg(if expanded { hover } else { fill })
        .text_color(t.foreground)
        .text_size(px(14.0))
        .line_height(px(20.0))
        .font_family(fonts::SANS)
        .font_weight(FontWeight(500.0))
        .whitespace_nowrap()
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .focus_visible(move |s| {
            s.border_color(t.ring)
                .shadow(shadows(&[Shadow::drop(0.0, 0.0, 3.0, t.ring.fade(0.5))]))
        })
        .child(label.into())
}

/// `<Button variant="ghost" size="icon-sm">`: a 28 px square, 6 px corners, 16 px icon.
pub fn ghost_icon_button(id: impl Into<ElementId>, icon: Icon, cx: &App) -> Stateful<Div> {
    let t = *cx.notra();
    let hover = if t.is_dark() {
        t.muted.fade(0.5)
    } else {
        t.muted
    };
    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(28.0))
        .rounded(px(6.0))
        .border_1()
        .border_color(gpui::transparent_black())
        .text_color(t.foreground)
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .focus_visible(move |s| {
            s.border_color(t.ring)
                .shadow(shadows(&[Shadow::drop(0.0, 0.0, 3.0, t.ring.fade(0.5))]))
        })
        .child(
            svg()
                .path(icon.path())
                .size(px(16.0))
                .text_color(t.foreground),
        )
}
