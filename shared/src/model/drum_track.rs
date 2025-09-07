use std::collections::HashMap;

use crate::model::{PitchName, PlacedDrumId, Placement, PlacementType, SampleId};
use crate::pmodel::{DrumSubTrackProto, DrumTrackProto, PitchNameProto, PlacedDrumProto};
use crate::types::*;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DrumTrack {
    pub drum_sub_tracks: HashMap<SampleId, DrumSubTrack>,
}

impl From<DrumTrackProto> for DrumTrack {
    fn from(item: DrumTrackProto) -> Self {
        let mut drum_sub_tracks: HashMap<SampleId, DrumSubTrack> = HashMap::new();
        for (key, proto) in item.drum_sub_tracks {
            drum_sub_tracks.insert(SampleId(key as usize), proto.into());
        }
        Self {
            drum_sub_tracks
        }
    }
}

impl From<DrumTrack> for DrumTrackProto {
    fn from(item: DrumTrack) -> Self {
        let mut drum_sub_tracks: HashMap<u32, DrumSubTrackProto> = HashMap::new();
        for (key, sub_track) in item.drum_sub_tracks {
            drum_sub_tracks.insert(*key as u32, sub_track.into());
        }
        Self {
            drum_sub_tracks
        }
    }
}

impl<'a> TryFrom<&'a Placement> for &'a DrumTrack {
    type Error = ();

    fn try_from(item: &'a Placement) -> Result<Self, ()> {
        match &item.kind {
            PlacementType::DrumTrack(it) => Ok(&it.drum_track),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<&'a mut Placement> for &'a mut DrumTrack {
    type Error = ();

    fn try_from(item: &'a mut Placement) -> Result<Self, ()> {
        match &mut item.kind {
            PlacementType::DrumTrack(it) => Ok(&mut it.drum_track),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto, Default)]
pub struct DrumSubTrack {
    #[proto_hashmap]
    pub placed_drums: HashMap<PlacedDrumId, PlacedDrum>,
}

// impl From<DrumSubTrackProto> for DrumSubTrack {
//     fn from(item: DrumSubTrackProto) -> Self {
//         let mut placed_drums: HashMap<PlacedDrumId, PlacedDrum> = HashMap::new();
//         for (key, proto) in item.placed_drums {
//             placed_drums.insert(PlacedDrumId(key as usize), proto.into());
//         }
//         Self {
//             placed_drums
//         }
//     }
// }

// impl From<DrumSubTrack> for DrumSubTrackProto {
//     fn from(item: DrumSubTrack) -> Self {
//         let mut placed_drums: HashMap<u32, PlacedDrumProto> = HashMap::new();
//         for (key, placed_drum) in item.placed_drums {
//             placed_drums.insert(*key as u32, placed_drum.into());
//         }
//         Self {
//             placed_drums
//         }
//     }
// }

/// Ordered by offset.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlacedDrum {
    // Represents a single drum hit
    pub offset: OrderedFloat<Beats>,

    pub clipped_duration: Option<OrderedFloat<Beats>>,

    pub pitch_name: PitchName,
}

impl From<PlacedDrumProto> for PlacedDrum {
    fn from(item: PlacedDrumProto) -> Self {
        let scale_value = item.pitch_name.unwrap().scale_value.into();
        let octave = item.pitch_name.unwrap().octave.into();
        Self {
            offset: item.offset.into(),
            clipped_duration: item.clipped_duration.map(OrderedFloat),
            pitch_name: PitchName { scale_value, octave },
        }
    }
}

impl From<PlacedDrum> for PlacedDrumProto {
    fn from(item: PlacedDrum) -> Self {
        let scale_value = item.pitch_name.scale_value.into();
        let octave = item.pitch_name.octave.into();
        Self {
            offset: *item.offset,
            clipped_duration: item.clipped_duration.map(|it| *it),
            pitch_name: Some(PitchNameProto {scale_value, octave}),
        }
    }
}

impl DrumTrack {
    // pub fn last_drum_offset(&self) -> OrderedFloat<Beats> {
    //     self.drums
    //         .iter()
    //         .map(|drum| drum.offset)
    //         .max_by(|x, y| x.cmp(y))
    //         .unwrap_or(OrderedFloat(0.0))
    // }
}
