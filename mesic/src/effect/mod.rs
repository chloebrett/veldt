mod delay;
mod eq;

use crate::sig::{mult, sum};
use shared::model::{Effect, EffectInstance};
use shared::types::KnobPosition;

/// Trait corresponding to something that knows how to apply an effect.
/// Implemented for the various effect config types by the various effect plugins.
pub trait ApplyEffect {
    fn apply(&self, input: &[f32]) -> Vec<f32>;
}

pub fn apply_effects(signal: &[f32], effects: &[EffectInstance]) -> Vec<f32> {
    let mut output = signal.to_owned();

    for effect in effects {
        output = apply_effect(output, effect);
    }

    output
}

/// Mixes two signals in the given dry/wet ratio.
fn mix(dry: &Vec<f32>, wet: &Vec<f32>, ratio: KnobPosition) -> Vec<f32> {
    sum(&mult(wet, ratio), &mult(dry, 1.0 - ratio))
}

fn apply_effect(dry_signal: Vec<f32>, effect: &EffectInstance) -> Vec<f32> {
    let wet_signal = match &effect.effect {
        Effect::SimpleDelay { config } => config.apply(&dry_signal),
        Effect::SimpleEq { config } => config.apply(&dry_signal),
        _ => panic!("Effect not implemented yet!"),
    };

    mix(&dry_signal, &wet_signal, effect.meta.wet)
}
