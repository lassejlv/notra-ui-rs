//! Navigation and controls for Rune: the sidebar and its dashboard block, icon
//! tabs and the step slider.

use ::rune_motion::Easing;

mod app_sidebar;
mod icon_tabs;
mod step_slider;

pub use app_sidebar::{AppSidebar, SidebarLens};
pub use icon_tabs::{IconTabs, IconTabsState};
pub use step_slider::{StepSlider, StepSliderState};

/// `1 - (1 - t)^5` sampled for Rune's piecewise easing.
const EASE_OUT_QUINT: Easing = Easing::Piecewise(&[
    [0.0, 0.0],
    [0.025, 0.119],
    [0.05, 0.226],
    [0.075, 0.323],
    [0.1, 0.410],
    [0.15, 0.556],
    [0.2, 0.672],
    [0.25, 0.763],
    [0.3, 0.832],
    [0.35, 0.884],
    [0.4, 0.922],
    [0.5, 0.969],
    [0.6, 0.990],
    [0.7, 0.998],
    [0.8, 1.0],
    [1.0, 1.0],
]);
