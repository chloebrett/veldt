use crate::model::Note;
use crate::pmodel::{PlacedNoteProto, TrackProto};
use crate::types::*;
use ordered_float::OrderedFloat;
use std::cmp::Ordering;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    /// Ordered by offset.
    pub notes: BTreeSet<PlacedNote>,
}

impl From<TrackProto> for Track {
    fn from(item: TrackProto) -> Track {
        Track {
            notes: item.notes.into_iter().map(|it| it.into()).collect(),
        }
    }
}

impl From<Track> for TrackProto {
    fn from(item: Track) -> TrackProto {
        TrackProto {
            notes: item.notes.into_iter().map(|it| it.into()).collect(),
        }
    }
}

impl From<PlacedNoteProto> for PlacedNote {
    fn from(item: PlacedNoteProto) -> PlacedNote {
        PlacedNote {
            note: item.note.unwrap().into(),
            offset: OrderedFloat(item.offset),
        }
    }
}

impl From<PlacedNote> for PlacedNoteProto {
    fn from(item: PlacedNote) -> PlacedNoteProto {
        PlacedNoteProto {
            note: Some(item.note.into()),
            offset: *item.offset,
        }
    }
}

/// Ordered by offset.
#[derive(Clone, Debug)]
pub struct PlacedNote {
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
        self.offset == other.offset
    }
}

impl Eq for PlacedNote {}
