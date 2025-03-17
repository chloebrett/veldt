use shared::model::{AdsrEnvelope, PlacedNote, Scale, ScaleValue, WaveType};
use shared::types::{Beats, Octave};

#[derive(Debug)]
pub enum Action {
    SetKey(ScaleValue),
    SetScale(Scale),
    SetBpm(Beats),
    // TODO: split note-related actions into a note-specific nested reducer.
    SetNoteScaleValue {
        track_index: usize,
        note_index: usize,
        note: ScaleValue,
    },
    SetNoteOctave {
        track_index: usize,
        note_index: usize,
        octave: Octave,
    },
    SetNoteOffset {
        track_index: usize,
        note_index: usize,
        offset: Beats,
    },
    SetNoteDuration {
        track_index: usize,
        note_index: usize,
        duration: Beats,
    },
    DeleteNote {
        track_index: usize,
        note_index: usize,
    },
    AddNote {
        track_index: usize,
        note: PlacedNote,
    },
    SetWave {
        generator_index: usize,
        wave: WaveType,
    },
    SetOscCount {
        generator_index: usize,
        osc_count: u32,
    },
    SetDetuneCents {
        generator_index: usize,
        detune_cents: f32,
    },
    SetEnvelope {
        generator_index: usize,
        envelope: AdsrEnvelope,
    },
}
