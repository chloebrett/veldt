use crate::model::note::Note;
use crate::pmodel::*;
use crate::serialize::map_vec;
use crate::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Sequence {
    pub offset: Beats,

    pub volume: Volume,

    pub synth_index: usize,

    pub notes: Vec<Note>,
}

impl From<SequenceProto> for Sequence {
    fn from(item: SequenceProto) -> Self {
        Sequence {
            offset: item.offset,
            volume: item.volume,
            synth_index: item.synth_index as usize,
            notes: map_vec(item.note),
        }
    }
}

impl From<Sequence> for SequenceProto {
    fn from(item: Sequence) -> Self {
        SequenceProto {
            offset: item.offset,
            volume: item.volume,
            synth_index: item.synth_index as u32,
            note: map_vec(item.notes),
        }
    }
}
