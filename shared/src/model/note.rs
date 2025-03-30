use super::pitch_name::PitchName;
use crate::pmodel::*;
use crate::types::*;
use local_macro::{FromProto, IntoProto};

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct Note {
    #[proto_optional]
    pub pitch_name: PitchName,
    pub beats: Beats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_note_to_proto_and_back() {
        let note = Note {
            pitch_name: 150.into(),
            beats: 1.0,
        };
        let proto: NoteProto = note.clone().into();
        let result: Note = proto.into();
        assert_eq!(note, result);
    }
}
