use crate::model::Note;
use crate::pmodel::{PlacedNoteProto, TrackProto};
use crate::types::*;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct Track {
    /// Ordered by offset.
    #[proto_repeated]
    pub notes: Vec<PlacedNote>,
}

/// Ordered by offset.
#[derive(Clone, Debug, FromProto, IntoProto)]
pub struct PlacedNote {
    #[proto_optional]
    pub note: Note,
    pub offset: OrderedFloat<Beats>,
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
        (self.offset == other.offset) & (self.note == other.note)
    }
}

impl Eq for PlacedNote {}
