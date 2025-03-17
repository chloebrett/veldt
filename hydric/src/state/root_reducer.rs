use super::{Action, StoreData, track_index, track_reducer};
use ordered_float::OrderedFloat;
use shared::model::{
    DelayConfig, Effect, EffectInstance, EqConfig, GeneratorType, SimpleWaveConfig,
};
use std::cell::RefMut;
use web_sys::console;

pub fn root_reducer(mut data: RefMut<'_, StoreData>, action: &Action) {
    console::log_1(&format!("root_reducer processing: {:?}", action.clone()).into());

    if let Some(track_index) = track_index(action) {
        return track_reducer(&mut data.project.tracks[track_index], action);
    }

    match action {
        Action::SetProjectName(name) => data.project.name = name.to_string(),
        Action::SetKey(key) => data.key = *key,
        Action::SetScale(scale) => data.scale = *scale,
        Action::SetBpm(bpm) => data.project.bpm = *bpm,
        Action::SetVolume(volume) => {
            console::log_1(&format!("old/new: {:?} {:?}", data.volume, volume).into());
            data.volume = *volume;
        }
        Action::SetWave {
            generator_index,
            wave,
        } => {
            let generator_type: &mut GeneratorType =
                &mut data.project.generators[*generator_index].kind;
            let generator_config: &mut SimpleWaveConfig = match generator_type {
                GeneratorType::SimpleWave { config } => config,
            };
            generator_config.wave = *wave;
        }
        Action::SetOscCount {
            generator_index,
            osc_count,
        } => {
            let generator_type: &mut GeneratorType =
                &mut data.project.generators[*generator_index].kind;
            let generator_config: &mut SimpleWaveConfig = match generator_type {
                GeneratorType::SimpleWave { config } => config,
            };
            generator_config.osc_count = *osc_count;
        }
        Action::SetDetuneCents {
            generator_index,
            detune_cents,
        } => {
            let generator_type: &mut GeneratorType =
                &mut data.project.generators[*generator_index].kind;
            let generator_config: &mut SimpleWaveConfig = match generator_type {
                GeneratorType::SimpleWave { config } => config,
            };
            generator_config.detune_cents = *detune_cents;
        }
        Action::SetEnvelope {
            generator_index,
            envelope,
        } => {
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

            let generator_type: &mut GeneratorType =
                &mut data.project.generators[*generator_index].kind;
            let generator_config: &mut SimpleWaveConfig = match generator_type {
                GeneratorType::SimpleWave { config } => config,
            };
            generator_config.envelope = envelope;
        }
        Action::SetDelayAmplitude {
            channel_index,
            effect_index,
            amplitude,
        } => {
            let effect_instance: &mut EffectInstance =
                &mut data.project.mixer[*channel_index].effects[*effect_index];
            let config: &mut DelayConfig = match &mut effect_instance.effect {
                Effect::SimpleDelay { config } => config,
                _ => panic!(),
            };
            config.amplitude = *amplitude;
        }
        Action::SetDelayMs {
            channel_index,
            effect_index,
            delay_ms,
        } => {
            let effect_instance: &mut EffectInstance =
                &mut data.project.mixer[*channel_index].effects[*effect_index];
            let config: &mut DelayConfig = match &mut effect_instance.effect {
                Effect::SimpleDelay { config } => config,
                _ => panic!(),
            };
            config.delay_ms = *delay_ms;
        }
        Action::SetEffectWet {
            channel_index,
            effect_index,
            wet,
        } => {
            let effect_instance: &mut EffectInstance =
                &mut data.project.mixer[*channel_index].effects[*effect_index];
            effect_instance.meta.wet = *wet;
        }
        Action::SetEqKind {
            channel_index,
            effect_index,
            kind,
        } => {
            let effect_instance: &mut EffectInstance =
                &mut data.project.mixer[*channel_index].effects[*effect_index];
            let config: &mut EqConfig = match &mut effect_instance.effect {
                Effect::SimpleEq { config } => config,
                _ => panic!(),
            };
            config.kind = kind.clone();
        }
        Action::SetEqFc {
            channel_index,
            effect_index,
            fc,
        } => {
            let effect_instance: &mut EffectInstance =
                &mut data.project.mixer[*channel_index].effects[*effect_index];
            let config: &mut EqConfig = match &mut effect_instance.effect {
                Effect::SimpleEq { config } => config,
                _ => panic!(),
            };
            config.fc = *fc;
        }
        Action::SetEqQ {
            channel_index,
            effect_index,
            q,
        } => {
            let effect_instance: &mut EffectInstance =
                &mut data.project.mixer[*channel_index].effects[*effect_index];
            let config: &mut EqConfig = match &mut effect_instance.effect {
                Effect::SimpleEq { config } => config,
                _ => panic!(),
            };
            config.q = *q;
        }
        _ => {}
    }
}
