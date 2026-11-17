use notra_ui::Icon;
use notra_ui::Theme;
use notra_ui::rune::{Button, ButtonSize, ButtonVariant};
use rune::{El, col};

use crate::{State, preview, section};

#[derive(Default)]
pub struct Demo {
    pub loading: [bool; 5],
    pub progress: Option<f32>,
}

impl Demo {
    pub fn busy() -> Self {
        Self {
            loading: [true; 5],
            progress: Some(60.0),
        }
    }
}

pub fn view(s: &State, t: &Theme) -> El {
    let variants = ButtonVariant::ALL.iter().fold(preview(t), |p, &variant| {
        let label = match variant {
            ButtonVariant::Primary => "Primary",
            ButtonVariant::Secondary => "Secondary",
            ButtonVariant::Outline => "Outline",
            ButtonVariant::Ghost => "Ghost",
            ButtonVariant::GhostMuted => "Ghost muted",
            ButtonVariant::Destructive => "Destructive",
            ButtonVariant::Link => "Link",
        };
        p.child(
            Button::new(label)
                .variant(variant)
                .build(t)
                .key(format!("variant-{label}")),
        )
    });
    let sizes = [
        ("Extra small", ButtonSize::Xs),
        ("Small", ButtonSize::Sm),
        ("Default", ButtonSize::Default),
        ("Large", ButtonSize::Lg),
    ]
    .iter()
    .fold(preview(t), |p, &(label, size)| {
        p.child(
            Button::new(label)
                .size(size)
                .build(t)
                .key(format!("size-{label}")),
        )
    })
    .child(
        Button::new("Disabled")
            .disabled(true)
            .build(t)
            .key("disabled"),
    )
    .child(
        Button::new("Disabled")
            .variant(ButtonVariant::Secondary)
            .disabled(true)
            .build(t)
            .key("disabled-secondary"),
    );

    let demo = &s.button;
    let toggle = |i: usize| move |s: &mut State| s.button.loading[i] = !s.button.loading[i];
    let loading = preview(t)
        .child(
            Button::new("Save changes")
                .loading(demo.loading[0])
                .on_press(toggle(0))
                .build(t)
                .key("save"),
        )
        .child(
            Button::new("Continue")
                .variant(ButtonVariant::Secondary)
                .icon_end(Icon::ArrowRight)
                .loading(demo.loading[1])
                .on_press(toggle(1))
                .build(t)
                .key("continue"),
        )
        .child(
            Button::new("Generate")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::Sm)
                .loading(demo.loading[2])
                .on_press(toggle(2))
                .build(t)
                .key("generate"),
        )
        .child(
            Button::new("Delete")
                .variant(ButtonVariant::Destructive)
                .loading(demo.loading[3])
                .on_press(toggle(3))
                .build(t)
                .key("delete"),
        )
        .child(
            Button::icon_button(Icon::ArrowUp)
                .loading(demo.loading[4])
                .on_press(toggle(4))
                .build(t)
                .key("send"),
        );

    let progress = preview(t).child(
        Button::new("Upload file")
            .variant(ButtonVariant::Secondary)
            .icon(Icon::ArrowUp)
            .progress(demo.progress)
            .on_press(|s: &mut State| {
                s.button.progress = match s.button.progress {
                    None => Some(0.0),
                    Some(p) if p >= 100.0 => None,
                    Some(p) => Some(p + 20.0),
                }
            })
            .build(t)
            .key("upload"),
    );

    col()
        .child(section("Variants", t, variants))
        .child(section("Sizes", t, sizes))
        .child(section("Loading", t, loading))
        .child(section("Progress", t, progress))
}
