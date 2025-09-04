use crate::model::{DrumTrackId, SampleId};
use crate::pmodel::{DrumTrackProto, PlacedDrumProto};
use crate::types::*;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto, Default)]
pub struct DrumTrack {
    #[proto_into]
    pub drum_track_id: DrumTrackId,
    pub sample_id: SampleId,
    #[proto_repeated]
    pub drums: Vec<PlacedDrum>,
}

/// Ordered by offset.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, FromProto, IntoProto)]
pub struct PlacedDrum {
    //represents a single drum hit
    pub offset: OrderedFloat<Beats>,
}

impl DrumTrack {
    pub fn last_drum_offset(&self) -> OrderedFloat<Beats> {
        self.drums
            .iter()
            .map(|drum| drum.offset)
            .max_by(|x, y| x.cmp(y))
            .unwrap_or(OrderedFloat(0.0))
    }
}
