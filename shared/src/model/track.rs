use crate::model::{Note, PitchName};
use crate::pmodel::{PlacedNoteProto, TrackProto};
use crate::types::*;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto, Default)]
pub struct Track {
    /// Ordered by offset.
    #[proto_repeated]
    pub notes: Vec<PlacedNote>,
    pub offset: OrderedFloat<Beats>,
    #[proto_repeated]
    pub delete_pitches: Vec<PitchName>,
}

impl Track {
    pub fn unclipped_duration(&self) -> OrderedFloat<f32> {
        self.notes
            .iter()
            .map(|note| note.offset + OrderedFloat(note.note.beats))
            .max_by(|x, y| x.cmp(y))
            .unwrap_or(OrderedFloat(0.0))
    }
}

/// Ordered by offset.
#[derive(Clone, Debug, FromProto, IntoProto)]
pub struct PlacedNote {
    #[proto_optional]
    pub note: Note,
    pub offset: OrderedFloat<Beats>,
    pub pitch_offset: f32,
    pub note_on: bool,
    pub note_deleted: bool,
}

impl PartialOrd for PlacedNote {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PlacedNote {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

// TODO: consider including note identity in this definition.
impl PartialEq for PlacedNote {
    fn eq(&self, other: &Self) -> bool {
        (self.offset == other.offset)
            && (self.note == other.note)
            && (self.note_on == other.note_on)
    }
}

impl Eq for PlacedNote {}
