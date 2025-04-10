use crate::Action;
use shared::logger::log;
use shared::model::{GeneratorInstance, GeneratorType};

pub fn generator_reducer(generator: &mut GeneratorInstance, action: &Action) -> Action {
    log(&format!(
        "generator_reducer processing: {:?}",
        action.clone()
    ));

    match action {
        Action::SetGeneratorVolume(volume) => {
            let prev = generator.meta.volume;
            generator.meta.volume = *volume;
            return Action::SetGeneratorVolume(prev);
        }
        Action::SetGeneratorMute(mute) => {
            let prev = generator.meta.mute;
            generator.meta.mute = *mute;
            return Action::SetGeneratorMute(prev);
        }
        _ => {}
    }

    let config = match &mut generator.kind {
        GeneratorType::SimpleWave { config } => config,
        GeneratorType::Noise { .. } => todo!(),
    };

    match action {
        Action::SetWave(wave) => {
            let prev = config.wave;
            config.wave = *wave;
            Action::SetWave(prev)
        }
        Action::SetOscCount(osc_count) => {
            let prev = config.osc_count;
            config.osc_count = *osc_count;
            Action::SetOscCount(prev)
        }
        Action::SetDetuneCents(detune_cents) => {
            let prev = config.detune_cents;
            config.detune_cents = *detune_cents;
            Action::SetDetuneCents(prev)
        }
        Action::SetEnvelope(envelope) => {
            let mut envelope = envelope.clone();
            let headroom = 1.0 - envelope.attack - envelope.decay - envelope.release;
            let max_attack = headroom + envelope.attack;
            let max_decay = headroom + envelope.decay;
            let max_release = headroom + envelope.release;

            if envelope.attack > max_attack {
                envelope.attack = max_attack;
            }
            if envelope.decay > max_decay {
                envelope.decay = max_decay;
            }
            if envelope.release > max_release {
                envelope.release = max_release;
            }

            let prev = config.envelope.clone();
            config.envelope = envelope;
            Action::SetEnvelope(prev)
        }
        Action::SetAntiAliasingMode(mode) => {
            let prev = config.anti_aliasing_mode;
            config.anti_aliasing_mode = *mode;
            Action::SetAntiAliasingMode(prev)
        }
        Action::SetOversampleFactor(factor) => {
            let prev = config.oversample_factor;
            config.oversample_factor = *factor;
            Action::SetOversampleFactor(prev)
        }
        _ => Action::NonReversible,
    }
}
