use crate::Action;
use shared::logger::log;
use shared::model::{Effect, EffectInstance};

pub fn effect_reducer(effect: &mut EffectInstance, action: &Action) -> Action {
    log(&format!("effect_reducer processing: {:?}", action.clone()));

    match action {
        Action::SetEffectWet(wet) => {
            let prev = effect.meta.wet;
            effect.meta.wet = *wet;
            return Action::SetEffectWet(prev);
        }
        Action::SetEffectMute(mute) => {
            let prev = effect.meta.mute;
            effect.meta.mute = *mute;
            return Action::SetEffectMute(prev);
        }
        _ => {}
    }

    match &mut effect.effect {
        Effect::SimpleDelay { config } => match action {
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
            Action::SetEqGain(gain) => {
                let prev = config.gain;
                config.gain = *gain;
                Action::SetEqGain(prev)
            }
            _ => Action::NonReversible,
        },
        Effect::SimpleCompressor { config } => match action {
            Action::SetCompressorThreshold(volume) => {
                let prev = config.threshold;
                config.threshold = *volume;
                Action::SetCompressorThreshold(prev)
            }
            Action::SetCompressorAttackMs(attack_ms) => {
                let prev = config.attack_ms;
                config.attack_ms = *attack_ms;
                Action::SetCompressorAttackMs(prev)
            }
            Action::SetCompressorReleaseMs(release_ms) => {
                let prev = config.release_ms;
                config.release_ms = *release_ms;
                Action::SetCompressorReleaseMs(prev)
            }
            Action::SetCompressorRatio(ratio) => {
                let prev = config.ratio;
                config.ratio = *ratio;
                Action::SetCompressorRatio(prev)
            }
            Action::SetCompressorGain(gain) => {
                let prev = config.gain;
                config.gain = *gain;
                Action::SetCompressorGain(prev)
            }
            _ => Action::NonReversible,
        },
        Effect::ModDelay { .. } => match action {
            _ => Action::NonReversible,
        },
    }
}
