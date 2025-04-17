use crate::Action;
use log::info;
use shared::model::{Effect, EffectInstance};
use std::cmp::{max, min};

pub fn effect_reducer(effect: &mut EffectInstance, action: &Action) -> Action {
    info!("effect_reducer processing: {:?}", action.clone());

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
            Action::SetDelayFeedback(feedback) => {
                let prev = config.feedback;
                config.feedback = *feedback;
                Action::SetDelayFeedback(prev)
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
        Effect::ModDelay { config } => match action {
            Action::SetModDelayMinDepth(min_depth) => {
                let prev = config.min_depth;
                // Prevent min_depth from going above max_depth.
                config.min_depth = min(*min_depth, config.max_depth);
                Action::SetModDelayMinDepth(prev)
            }
            Action::SetModDelayMaxDepth(max_depth) => {
                let prev = config.max_depth;
                // Prevent max_depth from going below min_depth.
                config.max_depth = max(*max_depth, config.min_depth);
                Action::SetModDelayMaxDepth(prev)
            }
            Action::SetModDelayLfoFreq(freq) => {
                let prev = config.freq;
                config.freq = *freq;
                Action::SetModDelayLfoFreq(prev)
            }
            Action::SetModDelayLfoType(lfo_type) => {
                let prev = config.lfo_type;
                config.lfo_type = *lfo_type;
                Action::SetModDelayLfoType(prev)
            }
            _ => todo!(),
        },
    }
}
