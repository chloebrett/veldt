use crate::model::Note;
use crate::pmodel::{PlacedNoteProto, TrackProto};
use crate::serialize::map_vec;
use crate::types::*;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    /// Ordered by offset.
    pub notes: Vec<PlacedNote>,
}

impl From<TrackProto> for Track {
    fn from(item: TrackProto) -> Track {
        Track {
            notes: map_vec(item.notes),
        }
    }
}

impl From<Track> for TrackProto {
    fn from(item: Track) -> TrackProto {
        TrackProto {
            notes: map_vec(item.notes),
        }
    }
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
