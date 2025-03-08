mod delay;
mod eq;

use crate::sig::{mult, sum};
use shared::model::{Effect, EffectInstance};
use shared::types::KnobPosition;

pub trait EffectFilter {
    fn apply(self, input: Vec<f32>) -> Vec<f32>;
}

pub fn apply_effects(signal: Vec<f32>, effects: Vec<EffectInstance>) -> Vec<f32> {
    let mut output = signal.clone();

    for effect in effects {
        output = apply_effect(output, effect);
    }

    output
}

/// Mixes two signals in the given dry/wet ratio.
fn mix(dry: Vec<f32>, wet: Vec<f32>, ratio: KnobPosition) -> Vec<f32> {
    sum(mult(wet, ratio), mult(dry, 1.0 - ratio))
}

fn apply_effect(dry_signal: Vec<f32>, effect: EffectInstance) -> Vec<f32> {
    let wet_signal = match effect.effect {
        Effect::SimpleDelay { config } => config.apply(dry_signal.clone()),
        Effect::SimpleEq { config } => config.apply(dry_signal.clone()),
        _ => panic!("Effect not implemented yet!"),
    };

    mix(dry_signal, wet_signal, effect.meta.wet)
}
