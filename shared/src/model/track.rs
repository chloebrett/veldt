use crate::serialize::map_vec;
use crate::pmodel::*;
use crate::model::sequence::Sequence;
use crate::model::synth::Synth;
use crate::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub bpm: Beats,

    pub synths: Vec<Synth>,

    pub sequences: Vec<Sequence>,
}

impl From<TrackProto> for Track {
    fn from(item: TrackProto) -> Self {
        Track {
            bpm: item.bpm,
            synths: map_vec(item.synth),
            sequences: map_vec(item.sequence),
        }
    }
}

impl From<Track> for TrackProto {
    fn from(item: Track) -> Self {
        TrackProto {
            bpm: item.bpm,
            synth: map_vec(item.synths),
            sequence: map_vec(item.sequences),
        }
    }
}

