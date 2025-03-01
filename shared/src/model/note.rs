use crate::pmodel::*;
use crate::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Note(pub Semitones, pub Beats);

impl From<NoteProto> for Note {
    fn from(item: NoteProto) -> Self {
        Note(item.semitones, item.beats)
    }
}

impl From<Note> for NoteProto {
    fn from(item: Note) -> Self {
        NoteProto {
            semitones: item.0,
            beats: item.1,
        }
    }
}
