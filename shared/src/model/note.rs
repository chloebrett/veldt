use crate::pmodel::*;
use crate::types::*;
use crate::model::pitch_name::PitchName;

#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub pitch_name: PitchName, 
    pub beats: Beats
}

impl From<NoteProto> for Note {
    fn from(item: NoteProto) -> Self {
        Note {
            pitch_name: item.pitch_name.unwrap().into(),
            beats: item.beats
        }
    }
}

impl From<Note> for NoteProto {
    fn from(item: Note) -> Self {
        NoteProto {
            pitch_name: Some(item.pitch_name.into()),
            beats: item.beats,
        }
    }
}
