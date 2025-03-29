use crate::Action;
use shared::logger::log;
use shared::model::{Effect, EffectInstance};

pub fn effect_reducer(effect: &mut EffectInstance, action: &Action) -> Action {
    log(&format!("effect_reducer processing: {:?}", action.clone()));

    if let Action::SetEffectWet(wet) = action {
        let prev = effect.meta.wet;
        effect.meta.wet = *wet;
        return Action::SetEffectWet(prev);
    }

    match &mut effect.effect {
        Effect::SimpleDelay { config } => match action {
            Action::SetDelayAmplitude(amplitude) => {
                let prev = config.amplitude;
                config.amplitude = *amplitude;
                Action::SetDelayAmplitude(prev)
            }
            Action::SetDelayMs(delay_ms) => {
                let prev = config.delay_ms;
                config.delay_ms = *delay_ms;
                Action::SetDelayMs(prev)
            }
            _ => Action::NonReversible,
        },
        Effect::SimpleEq { config } => match action {
            Action::SetEqKind(kind) => {
                let prev = config.kind.clone();
                config.kind = kind.clone();
                Action::SetEqKind(prev)
            }
            Action::SetEqFc(fc) => {
                let prev = config.fc;
                config.fc = *fc;
                Action::SetEqFc(prev)
            }
            Action::SetEqQ(q) => {
                let prev = config.q;
                config.q = *q;
                Action::SetEqQ(prev)
            }
            _ => Action::NonReversible,
        },
        // TODO
        Effect::SimpleCompressor { .. } => Action::NonReversible,
    }
}
