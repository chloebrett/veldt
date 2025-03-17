use crate::state::Action;
use shared::model::{Effect, EffectInstance};
use web_sys::console;

pub fn effect_reducer(effect: &mut EffectInstance, action: &Action) {
    console::log_1(&format!("effect_reducer processing: {:?}", action.clone()).into());

    if let Action::SetEffectWet(wet) = action {
        effect.meta.wet = *wet;
        return;
    }

    match &mut effect.effect {
        Effect::SimpleDelay { config } => match action {
            Action::SetDelayAmplitude(amplitude) => {
                config.amplitude = *amplitude;
            }
            Action::SetDelayMs(delay_ms) => {
                config.delay_ms = *delay_ms;
            }
            _ => {}
        },
        Effect::SimpleEq { config } => match action {
            Action::SetEqKind(kind) => {
                config.kind = kind.clone();
            }
            Action::SetEqFc(fc) => {
                config.fc = *fc;
            }
            Action::SetEqQ(q) => {
                config.q = *q;
            }
            _ => {}
        },
        Effect::SimpleCompressor { .. } => {}
    }
}
