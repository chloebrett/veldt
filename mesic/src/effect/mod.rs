pub mod resonator_sa;
pub mod resonator_simple;

/// Mixes two signals in the given dry/wet ratio.
fn mix(dry: Vec<f32>, wet: Vec<f32>, ratio: f32) -> Vec<f32> {
    sum(mult(wet, ratio), mult(dry, 1.0 - ratio))
}

fn apply_effects(signal: Vec<f32>, effects: Vec<EffectInstance>) -> Vec<f32> {
    let mut output = signal.clone();

    for effect in effects {
        output = apply_effect(output, effect);
    }

    output
}

fn apply_effect(dry_signal: Vec<f32>, effect: EffectInstance) -> Vec<f32> {
    let wet_signal = match effect.effect {
        Effect::SimpleDelay {
            amplitude,
            delay_ms,
        } => apply_delay(dry_signal.clone(), amplitude, delay_ms),
        Effect::SimpleEq {
            kind:
                EqType::Pass {
                    kind:
                        PassType::Band {
                            algorithm: BandPassAlgorithm::SimpleResonator,
                        },
                },
            freq,
            q_value,
        } => apply_simple_resonator(dry_signal.clone(), freq, q_value),
        Effect::SimpleEq {
            kind:
                EqType::Pass {
                    kind:
                        PassType::Band {
                            algorithm: BandPassAlgorithm::SmithAngell,
                        },
                },
            freq,
            q_value,
        } => apply_smith_angell_resonator(dry_signal.clone(), freq, q_value),
        _ => panic!("Effect not implemented yet!"),
    };

    mix(dry_signal, wet_signal, effect.meta.wet)
}

