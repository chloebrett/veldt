mod delay;
mod filter;
mod pass_basic_first_order;
mod pass_basic_second_order;
mod resonator_sa;
mod resonator_simple;

use crate::sig::{mult, sum};
use delay::*;
use pass_basic_first_order::*;
use pass_basic_second_order::*;
use resonator_sa::*;
use resonator_simple::*;
use shared::model::{
    BandPassAlgorithm, BandStopAlgorithm, Effect, EffectInstance, EqType, LowHighPassAlgorithm,
    PassType,
};
use shared::types::{Freq, KnobPosition};

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
        Effect::SimpleDelay {
            amplitude,
            delay_ms,
        } => apply_delay(dry_signal.clone(), amplitude, delay_ms),
        Effect::SimpleEq {
            kind: EqType::Pass {
                kind: PassType::Low { algorithm },
            },
            freq,
            q_value,
        } => apply_low_pass(dry_signal.clone(), freq, q_value, algorithm),
        Effect::SimpleEq {
            kind: EqType::Pass {
                kind: PassType::High { algorithm },
            },
            freq,
            q_value,
        } => apply_high_pass(dry_signal.clone(), freq, q_value, algorithm),
        Effect::SimpleEq {
            kind: EqType::Pass {
                kind: PassType::Band { algorithm },
            },
            freq,
            q_value,
        } => apply_band_pass(dry_signal.clone(), freq, q_value, algorithm),
        Effect::SimpleEq {
            kind:
                EqType::BandStop {
                    algorithm: BandStopAlgorithm::SimpleSecondOrder,
                },
            freq,
            q_value,
        } => apply_band_stop_basic(dry_signal.clone(), freq, q_value),
        _ => panic!("Effect not implemented yet!"),
    };

    mix(dry_signal, wet_signal, effect.meta.wet)
}

fn apply_low_pass(
    dry_signal: Vec<f32>,
    freq: Freq,
    q_value: KnobPosition,
    algorithm: LowHighPassAlgorithm,
) -> Vec<f32> {
    match algorithm {
        LowHighPassAlgorithm::SimpleFirstOrder => {
            apply_low_pass_basic_first_order(dry_signal.clone(), freq)
        }
        LowHighPassAlgorithm::SimpleSecondOrder => {
            apply_low_pass_basic_second_order(dry_signal.clone(), freq, q_value)
        }
        _ => panic!("Unimplemented!"),
    }
}

fn apply_high_pass(
    dry_signal: Vec<f32>,
    freq: Freq,
    q_value: KnobPosition,
    algorithm: LowHighPassAlgorithm,
) -> Vec<f32> {
    match algorithm {
        LowHighPassAlgorithm::SimpleFirstOrder => {
            apply_high_pass_basic_first_order(dry_signal.clone(), freq)
        }
        LowHighPassAlgorithm::SimpleSecondOrder => {
            apply_high_pass_basic_second_order(dry_signal.clone(), freq, q_value)
        }
        _ => panic!("Unimplemented!"),
    }
}

fn apply_band_pass(
    dry_signal: Vec<f32>,
    freq: Freq,
    q_value: KnobPosition,
    algorithm: BandPassAlgorithm,
) -> Vec<f32> {
    match algorithm {
        BandPassAlgorithm::SimpleResonator => {
            apply_simple_resonator(dry_signal.clone(), freq, q_value)
        }
        BandPassAlgorithm::SmithAngell => {
            apply_smith_angell_resonator(dry_signal.clone(), freq, q_value)
        }
        BandPassAlgorithm::SimpleSecondOrder => {
            apply_band_pass_basic(dry_signal.clone(), freq, q_value)
        }
    }
}
