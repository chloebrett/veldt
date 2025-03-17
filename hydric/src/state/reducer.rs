use super::{Action, StoreData};
use ordered_float::OrderedFloat;
use shared::model::{
    DelayConfig, Effect, EffectInstance, EqConfig, GeneratorType, SimpleWaveConfig,
};
use std::cell::RefMut;

pub fn reducer(mut data: RefMut<'_, StoreData>, action: Action) {
    match action {
        Action::SetProjectName(name) => data.project.name = name,
        Action::SetKey(key) => store.key = key,
        Action::SetScale(scale) => store.scale = scale,
        Action::SetBpm(bpm) => store.project.bpm = bpm,
        Action::SetVolume(volume) => store.volume = volume,
        Action::SetNoteScaleValue {
            track_index,
            note_index,
            note,
        } => {
            let notes = &mut store.project.tracks[track_index].notes;
            notes[note_index].note.pitch_name.scale_value = note
        }
        Action::SetNoteOctave {
            track_index,
            note_index,
            octave,
        } => {
            let notes = &mut store.project.tracks[track_index].notes;
            notes[note_index].note.pitch_name.octave = octave
        }
        Action::SetNoteDuration {
            track_index,
            note_index,
            duration,
        } => {
            let notes = &mut store.project.tracks[track_index].notes;
            notes[note_index].note.beats = duration
        }
        Action::SetNoteOffset {
            track_index,
            note_index,
            offset,
        } => {
            let notes = &mut store.project.tracks[track_index].notes;
            notes[note_index].offset = OrderedFloat(offset)
        }
        Action::DeleteNote {
            track_index,
            note_index,
        } => {
            store.project.tracks[track_index].notes.remove(note_index);
        }
        Action::AddNote { track_index, note } => {
            store.project.tracks[track_index].notes.push(note);
        }
        Action::SetWave {
            generator_index,
            wave,
        } => {
            let generator_type: &mut GeneratorType =
                &mut store.project.generators[generator_index].kind;
            let generator_config: &mut SimpleWaveConfig = match generator_type {
                GeneratorType::SimpleWave { config } => config,
            };
            generator_config.wave = wave;
        }
        Action::SetOscCount {
            generator_index,
            osc_count,
        } => {
            let generator_type: &mut GeneratorType =
                &mut store.project.generators[generator_index].kind;
            let generator_config: &mut SimpleWaveConfig = match generator_type {
                GeneratorType::SimpleWave { config } => config,
            };
            generator_config.osc_count = osc_count;
        }
        Action::SetDetuneCents {
            generator_index,
            detune_cents,
        } => {
            let generator_type: &mut GeneratorType =
                &mut store.project.generators[generator_index].kind;
            let generator_config: &mut SimpleWaveConfig = match generator_type {
                GeneratorType::SimpleWave { config } => config,
            };
            generator_config.detune_cents = detune_cents;
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
                &mut store.project.generators[generator_index].kind;
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
                &mut store.project.mixer[channel_index].effects[effect_index];
            let config: &mut DelayConfig = match &mut effect_instance.effect {
                Effect::SimpleDelay { config } => config,
                _ => panic!(),
            };
            config.amplitude = amplitude;
        }
        Action::SetDelayMs {
            channel_index,
            effect_index,
            delay_ms,
        } => {
            let effect_instance: &mut EffectInstance =
                &mut store.project.mixer[channel_index].effects[effect_index];
            let config: &mut DelayConfig = match &mut effect_instance.effect {
                Effect::SimpleDelay { config } => config,
                _ => panic!(),
            };
            config.delay_ms = delay_ms;
        }
        Action::SetEffectWet {
            channel_index,
            effect_index,
            wet,
        } => {
            let effect_instance: &mut EffectInstance =
                &mut store.project.mixer[channel_index].effects[effect_index];
            effect_instance.meta.wet = wet;
        }
        Action::SetEqKind {
            channel_index,
            effect_index,
            kind,
        } => {
            let effect_instance: &mut EffectInstance =
                &mut store.project.mixer[channel_index].effects[effect_index];
            let config: &mut EqConfig = match &mut effect_instance.effect {
                Effect::SimpleEq { config } => config,
                _ => panic!(),
            };
            config.kind = kind;
        }
        Action::SetEqFc {
            channel_index,
            effect_index,
            fc,
        } => {
            let effect_instance: &mut EffectInstance =
                &mut store.project.mixer[channel_index].effects[effect_index];
            let config: &mut EqConfig = match &mut effect_instance.effect {
                Effect::SimpleEq { config } => config,
                _ => panic!(),
            };
            config.fc = fc;
        }
        Action::SetEqQ {
            channel_index,
            effect_index,
            q,
        } => {
            let effect_instance: &mut EffectInstance =
                &mut store.project.mixer[channel_index].effects[effect_index];
            let config: &mut EqConfig = match &mut effect_instance.effect {
                Effect::SimpleEq { config } => config,
                _ => panic!(),
            };
            config.q = q;
        }
    }
}
