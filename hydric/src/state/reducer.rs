use super::{Action, Store};
use ordered_float::OrderedFloat;
use shared::model::{GeneratorType, SimpleWaveConfig};

pub fn reducer(store: &mut Store, action: Action) {
    match action {
        Action::SetKey(key) => store.key = key,
        Action::SetScale(scale) => store.scale = scale,
        Action::SetBpm(bpm) => store.project.bpm = bpm,
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
    }
}
