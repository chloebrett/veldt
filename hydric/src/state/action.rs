use shared::model::{PlacedNote, Scale, ScaleValue};
use shared::types::{Beats, Octave};

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
}
