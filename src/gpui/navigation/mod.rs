//! Navigation and controls for GPUI: the sidebar and its dashboard block, icon
//! tabs and the step slider.

use std::sync::LazyLock;
use std::time::Instant;

use crate::theme::Color;

mod app_sidebar;
mod icon_tabs;
mod sidebar;
mod step_slider;

pub use app_sidebar::{AppSidebar, demo_page as app_sidebar_demo_page};
pub use icon_tabs::IconTabs;
pub use sidebar::{
    RailEvent, SidebarFrame, SidebarInset, SidebarMenuButton, SidebarMotion, SidebarRail,
    SidebarTrigger,
};
pub use step_slider::StepSlider;

static EPOCH: LazyLock<Instant> = LazyLock::new(Instant::now);

/// Seconds on a process-wide monotonic clock, the time base of the shared models.
fn seconds(at: Instant) -> f64 {
    at.saturating_duration_since(*EPOCH).as_secs_f64()
}

/// Oklab mix that returns the ends exactly.
fn mix(a: Color, b: Color, t: f32) -> Color {
    if t <= 0.0 {
        a
    } else if t >= 1.0 {
        b
    } else {
        a.mix(b, t)
    }
}
