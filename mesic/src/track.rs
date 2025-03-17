use ordered_float::OrderedFloat;
use shared::model::{Note, PlacedNote, Track};
use std::collections::BTreeSet;

pub fn create_track(notes: Vec<Note>) -> Track {
    let mut current_beat = 0.0;
    let mut placed_notes = BTreeSet::new();

    for note in notes {
        placed_notes.insert(PlacedNote {
            note: note.clone(),
            offset: OrderedFloat(current_beat),
        });
        current_beat += note.beats;
    }

    Track {
        notes: placed_notes.into_iter().collect(),
    }
}
