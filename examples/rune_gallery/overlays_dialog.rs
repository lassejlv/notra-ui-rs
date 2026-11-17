use notra_ui::Theme;
use notra_ui::components::overlays::toast::demo;
use notra_ui::rune::overlays::{Dialog, Sheet, Toast, ToastState, Toaster};
use notra_ui::rune::{Button, ButtonVariant, ToRune, sans};
use rune::{El, Tx, col, text};

use crate::overlays_demo::outline_button;
use crate::{PAGES, State, preview, section};

/// The sonner, dialog and sheet demos' state.
#[derive(Default)]
pub struct Demo {
    pub toasts: ToastState,
    pub dialog: bool,
    pub sheet: bool,
    /// When the loading toast turns into a success, in frame-clock seconds.
    pub publish_at: Option<f64>,
}

fn toasts(s: &mut State) -> &mut ToastState {
    &mut s.overlays_dialog.toasts
}

fn push(make: fn() -> Toast) -> impl Fn(&mut State, &mut Tx) {
    move |s: &mut State, tx: &mut Tx| s.overlays_dialog.toasts.push(make(), tx)
}

pub fn sonner_view(_s: &State, t: &Theme) -> El {
    let button = |label: &str, make: fn() -> Toast| {
        outline_button(label, false, t)
            .key(format!("toast-{label}"))
            .on_press_tx(push(make))
    };
    let types = preview(t)
        .items_start()
        .min_h(320.0)
        .child(button("Default", demo::default))
        .child(button("Success", demo::success))
        .child(button("Info", demo::info))
        .child(button("Warning", demo::warning))
        .child(button("Error", demo::error))
        .child(
            outline_button("Loading", false, t)
                .key("toast-Loading")
                .on_press_tx(|s: &mut State, tx: &mut Tx| {
                    s.overlays_dialog.toasts.push(demo::loading(), tx);
                    s.overlays_dialog.publish_at = Some(tx.clock() + 2.0);
                    tx.wake_after(std::time::Duration::from_secs(2));
                }),
        );
    let actions = preview(t)
        .items_start()
        .min_h(256.0)
        .child(button("With action", demo::with_action))
        .child(button("With cancel", demo::with_cancel));
    col()
        .child(section("Types", t, types))
        .child(section("Action and cancel", t, actions))
}

pub fn dialog_view(s: &State, t: &Theme) -> El {
    col().child(section(
        "Demo",
        t,
        preview(t).child(
            outline_button("Open dialog", s.overlays_dialog.dialog, t)
                .key("open-dialog")
                .on_press(|s: &mut State| s.overlays_dialog.dialog = true),
        ),
    ))
}

pub fn sheet_view(s: &State, t: &Theme) -> El {
    col().child(section(
        "Demo",
        t,
        preview(t).child(
            outline_button("Open sheet", s.overlays_dialog.sheet, t)
                .key("open-sheet")
                .on_press(|s: &mut State| s.overlays_dialog.sheet = true),
        ),
    ))
}

fn body(text_: &str, t: &Theme) -> El {
    text(text_.to_string())
        .font(sans(14.0, 400.0, 20.0))
        .color(t.muted_foreground.rune())
        .wrap()
}

/// What the overlay pages mount over the whole window: the toaster, and the open dialog or
/// sheet.
pub fn window_layer(s: &State, t: &Theme) -> Vec<El> {
    let d = &s.overlays_dialog;
    let page = PAGES[s.page].0;
    let mut layer = Vec::new();
    if page == "dialog" && d.dialog {
        layer.push(
            Dialog::new("rename")
                .title("Rename workflow")
                .description("Pick a name your team will recognise in the run history.")
                .child(body("Runs keep their history; only the label changes.", t))
                .footer(
                    Button::new("Cancel")
                        .variant(ButtonVariant::Outline)
                        .on_press(|s: &mut State| s.overlays_dialog.dialog = false)
                        .build(t)
                        .key("cancel"),
                )
                .footer(
                    Button::new("Save")
                        .on_press(|s: &mut State| s.overlays_dialog.dialog = false)
                        .build(t)
                        .key("save"),
                )
                .build(t, |s: &mut State| s.overlays_dialog.dialog = false),
        );
    }
    if page == "sheet" && d.sheet {
        layer.push(
            Sheet::new("prospect")
                .title("Prospect details")
                .description("devtools.fm, a podcast about developer tools.")
                .child(col().px(16.0).child(body(
                    "Last contacted 12 days ago. Two replies, one meeting booked.",
                    t,
                )))
                .footer(
                    Button::new("Save changes")
                        .on_press(|s: &mut State| s.overlays_dialog.sheet = false)
                        .build(t)
                        .key("save"),
                )
                .footer(
                    Button::new("Cancel")
                        .variant(ButtonVariant::Outline)
                        .on_press(|s: &mut State| s.overlays_dialog.sheet = false)
                        .build(t)
                        .key("cancel"),
                )
                .build(t, |s: &mut State| s.overlays_dialog.sheet = false),
        );
    }
    if page == "sonner" {
        layer.push(Toaster::new("toaster").build(
            &d.toasts,
            t,
            toasts,
            |s: &mut State, toast: &Toast, tx: &mut Tx| {
                if toast.id == "trash" {
                    s.overlays_dialog.toasts.push(demo::restored(), tx);
                }
            },
        ));
    }
    layer
}

/// `AppSpec::poll`: expires toasts and turns the loading toast into a success after 2 s.
pub fn poll(s: &mut State, now: f64, tx: &mut Tx) -> bool {
    let d = &mut s.overlays_dialog;
    let mut changed = false;
    if d.publish_at.is_some_and(|at| now >= at) {
        d.publish_at = None;
        d.toasts.push(demo::published(), tx);
        changed = true;
    }
    d.toasts.poll(now, tx) || changed
}

/// Three toasts already showing, the newest an error.
pub fn stack_scene(s: &mut State) {
    let toasts = &mut s.overlays_dialog.toasts;
    toasts.push_at(demo::default(), 0.0);
    toasts.push_at(demo::warning(), 0.0);
    toasts.push_at(demo::error(), 0.0);
}
