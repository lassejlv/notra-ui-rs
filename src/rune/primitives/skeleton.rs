use ::rune::{El, el};
use ::rune_motion::{Beat, LoopSpec, Sequence};

use crate::components::primitives::skeleton;
use crate::rune::ToRune;
use crate::theme::Theme;

const PULSE: &[Beat] = &[
    Beat::new(0.0).opacity(skeleton::FRAMES[0].1),
    Beat::new(1.0)
        .opacity(skeleton::FRAMES[1].1)
        .easing(::rune_motion::Easing::CubicBezier(
            ::rune_motion::CubicBezier::new(0.4, 0.0, 0.6, 1.0),
        )),
    Beat::new(2.0)
        .opacity(skeleton::FRAMES[2].1)
        .easing(::rune_motion::Easing::CubicBezier(
            ::rune_motion::CubicBezier::new(0.4, 0.0, 0.6, 1.0),
        )),
];

/// A pulsing placeholder; size the result: `Skeleton::build(&theme).w(120.0).h(16.0)`.
pub struct Skeleton;

impl Skeleton {
    pub fn build(theme: &Theme) -> El {
        el().radius(skeleton::RADIUS)
            .fill(skeleton::fill(theme).rune())
            .looping(LoopSpec::sequence(Sequence::new(PULSE)))
            .a11y_hidden()
    }
}
