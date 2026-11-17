//! shadcn's `Collapsible` (Base UI), with the panel motion Notra's blocks give it: the
//! panel's rows grow from `0fr` to `1fr` while it fades in, and reverse on close.

use std::time::Duration;

use crate::motion::{Tween, ease};

/// `transition-[grid-template-rows,opacity] duration-300 ease-[cubic-bezier(0.22,1,0.36,1)]`.
pub const PANEL: Tween = Tween::new(Duration::from_millis(300), ease::PROGRESS);
/// The disclosure chevron's half turn when open (`[data-panel-open]>svg:rotate-180`),
/// on the trigger's `transition-all`.
pub const CHEVRON: Tween = Tween::new(Duration::from_millis(150), ease::IN_OUT);

/// An open collapsible's panel, `progress` of the way open: its height and opacity
/// for content `height` tall.
pub fn panel_frame(height: f32, progress: f32) -> (f32, f32) {
    let p = progress.clamp(0.0, 1.0);
    (height * p, p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_opens_from_nothing() {
        assert_eq!(panel_frame(120.0, 0.0), (0.0, 0.0));
        assert_eq!(panel_frame(120.0, 1.0), (120.0, 1.0));
        assert!(PANEL.progress(Duration::from_millis(100)) > 0.6);
    }
}
