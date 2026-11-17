//! The plain shadcn buttons the live overlay examples use as triggers
//! (`@/components/ui/button`, base-nova): `variant="outline"` and `variant="ghost" size="icon-sm"`.

use notra_ui::Icon;
use notra_ui::Theme;
use notra_ui::motion::Tween;
use notra_ui::rune::{ToRune, sans, tween};
use rune::{El, icon, row, text};
use std::time::Duration;

const HOVER: Tween = Tween::new(Duration::from_millis(150), notra_ui::motion::ease::IN_OUT);

/// `<Button variant="outline">`: 32 pt tall, 10 pt in, 8 pt corners; `bg-muted` on hover
/// and while its popup is open (`aria-expanded`).
pub fn outline_button(label: &str, expanded: bool, t: &Theme) -> El {
    let (fill, hover, border) = if t.is_dark() {
        (t.input.fade(0.3), t.input.fade(0.5), t.input)
    } else {
        (t.background, t.muted, t.border)
    };
    row()
        .h(32.0)
        .px(10.0)
        .gap(6.0)
        .center_y()
        .justify_center()
        .radius(8.0)
        .border(1.0, border.rune())
        .fill(if expanded { hover } else { fill }.rune())
        .hover(hover.rune())
        .hover_motion(tween(HOVER), tween(HOVER))
        .role("button")
        .label(label)
        .expanded(expanded)
        .focusable(true)
        .focus_ring(3.0, 0.0, t.ring.fade(0.5).rune())
        .cursor(rune::Cursor::Pointer)
        .child(
            text(label)
                .font(sans(14.0, 500.0, 20.0))
                .color(t.foreground.rune()),
        )
}

/// `<Button variant="ghost" size="icon-sm">`: a 28 pt square, 6 pt corners, 16 pt icon.
pub fn ghost_icon_button(glyph: Icon, label: &str, t: &Theme) -> El {
    let hover = if t.is_dark() {
        t.muted.fade(0.5)
    } else {
        t.muted
    };
    row()
        .size(28.0, 28.0)
        .center_y()
        .justify_center()
        .radius(6.0)
        .hover(hover.rune())
        .hover_motion(tween(HOVER), tween(HOVER))
        .role("button")
        .label(label)
        .focusable(true)
        .focus_ring(3.0, 0.0, t.ring.fade(0.5).rune())
        .cursor(rune::Cursor::Pointer)
        .child(icon(glyph.rune(), 16.0).color(t.foreground.rune()))
}
