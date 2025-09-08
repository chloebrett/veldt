use crate::model::{DrumSubTrack, DrumTrack, DrumTrackId, GeneratorId, SampleId, TrackId};
use crate::pmodel::{placement_proto::Kind as PlacementTypeProto, *};
use crate::types::Beats;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Placement {
    pub kind: PlacementType,

    /// The time that the track starts within the arrangement.
    pub offset: OrderedFloat<Beats>,

    /// If None, then duration is not clipped.
    pub clipped_duration: Option<OrderedFloat<Beats>>,

    /// Position that the track should be displayed visually, useful if there are overlapping tracks.
    /// Zero is the top.
    pub visual_placement: u32,

    /// rgb colours
    pub colour: Colour,
}

#[derive(Clone, Copy, Debug, PartialEq, Default, FromProto, IntoProto)]
pub struct Colour {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Eq for Colour {}

impl Colour {
    pub const fn from_8bit(r: u8, g: u8, b: u8) -> Self {
        Colour {
            r: r as f32,
            g: g as f32,
            b: b as f32,
        }
    }

    pub fn to_float_slice(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    pub const fn black() -> Self {
        Colour::from_8bit(0, 0, 0)
    }

    pub const fn white() -> Self {
        Colour::from_8bit(255, 255, 255)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlacementType {
    Track(TrackPlacement),
    Sample(SamplePlacement),
    DrumTrack(DrumTrackPlacement),
}

impl Default for PlacementType {
    fn default() -> Self {
        PlacementType::Track(TrackPlacement::default())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FromProto, IntoProto, Default)]
pub struct TrackPlacement {
    #[proto_into]
    pub track_id: TrackId,

    /// The generator this track placement is associated with.
    #[proto_into]
    pub generator_id: GeneratorId,
}

impl<'a> TryFrom<&'a Placement> for &'a TrackPlacement {
    type Error = ();

    fn try_from(item: &'a Placement) -> Result<Self, ()> {
        match &item.kind {
            PlacementType::Track(it) => Ok(it),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FromProto, IntoProto, Default)]
pub struct SamplePlacement {
    #[proto_into]
    pub sample_id: SampleId,
}

impl<'a> TryFrom<&'a Placement> for &'a SamplePlacement {
    type Error = ();

    fn try_from(item: &'a Placement) -> Result<Self, ()> {
        match &item.kind {
            PlacementType::Sample(it) => Ok(it),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrumTrackPlacement {
    pub drum_track_id: DrumTrackId,

    pub drum_track: DrumTrack,
}

impl Eq for DrumTrackPlacement {}

impl<'a> TryFrom<&'a Placement> for &'a DrumTrackPlacement {
    type Error = ();

    fn try_from(item: &'a Placement) -> Result<Self, ()> {
        match &item.kind {
            PlacementType::DrumTrack(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl From<DrumTrackPlacementProto> for DrumTrackPlacement {
    fn from(item: DrumTrackPlacementProto) -> Self {
        let mut drum_sub_tracks: HashMap<SampleId, DrumSubTrack> = HashMap::new();
        for (id, drum_sub_track_proto) in item.drum_track.unwrap().drum_sub_tracks {
            drum_sub_tracks.insert(SampleId(id as usize), drum_sub_track_proto.into());
        }
        Self {
            drum_track_id: item.drum_track_id.into(),
            drum_track: DrumTrack { drum_sub_tracks },
        }
    }
}

impl From<DrumTrackPlacement> for DrumTrackPlacementProto {
    fn from(item: DrumTrackPlacement) -> Self {
        let mut drum_sub_tracks: HashMap<u32, DrumSubTrackProto> = HashMap::new();
        for (sample_id, sub_track) in item.drum_track.drum_sub_tracks {
            drum_sub_tracks.insert(*sample_id as u32, sub_track.into());
        }
        Self {
            drum_track_id: item.drum_track_id.into(),
            drum_track: Some(DrumTrackProto { drum_sub_tracks }),
        }
    }
}

impl PartialOrd for Placement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Placement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl From<PlacementProto> for Placement {
    fn from(item: PlacementProto) -> Self {
        Self {
            kind: match item.kind.unwrap() {
                PlacementTypeProto::Track(it) => PlacementType::Track(it.into()),
                PlacementTypeProto::Sample(it) => PlacementType::Sample(it.into()),
                PlacementTypeProto::DrumTrack(it) => PlacementType::DrumTrack(it.into()),
            },
            offset: item.offset.into(),
            clipped_duration: item.clipped_duration.map(OrderedFloat),
            visual_placement: item.visual_placement,
            colour: item.colour.unwrap().into(),
        }
    }
}

impl From<Placement> for PlacementProto {
    fn from(item: Placement) -> Self {
        Self {
            kind: Some(match item.kind {
                PlacementType::Track(it) => PlacementTypeProto::Track(it.into()),
                PlacementType::Sample(it) => PlacementTypeProto::Sample(it.into()),
                PlacementType::DrumTrack(it) => PlacementTypeProto::DrumTrack(it.into()),
            }),
            offset: *item.offset,
            clipped_duration: item.clipped_duration.map(|it| *it),
            visual_placement: item.visual_placement,
            colour: Some(item.colour.into()),
        }
    }
}
