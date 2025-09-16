use crate::model::Project;
use crate::model::samples_to_beats;
use crate::model::{GeneratorId, SampleId, TrackId};
use crate::pmodel::{placement_proto::Kind as PlacementTypeProto, *};
use crate::types::Beats;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;
use std::cmp::Ordering;
use std::cmp::max;

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

#[derive(Clone, Copy, Debug, Eq, Default, PartialEq, FromProto, IntoProto)]
pub struct DrumTrackPlacement {
    pub track_id: TrackId,

    pub sample_id: SampleId,
}

impl DrumTrackPlacement {
    pub fn duration(&self, project: &Project) -> OrderedFloat<f32> {
        let max_offset = project.tracks[&self.track_id]
            .notes
            .iter()
            .max_by_key(|placed_note| placed_note.offset)
            .map(|last_note| last_note.offset.into())
            .unwrap_or(0.0);
        let sample_duration = project
            .samples
            .get(&self.sample_id)
            .map(|sample| samples_to_beats(max(sample.left.len(), sample.right.len()), project.bpm))
            .unwrap_or(0.5);

        // TODO: currently adding the sample duration multiplied by 2 to the last offset to
        // account for if the sample is pitched lower (assuming future tuning approach will change duration)
        // Find a better way to do this once tuning/re-pitching is implemented
        OrderedFloat(max_offset + sample_duration * 2.0)
    }
}

impl<'a> TryFrom<&'a Placement> for &'a DrumTrackPlacement {
    type Error = ();

    fn try_from(item: &'a Placement) -> Result<Self, ()> {
        match &item.kind {
            PlacementType::DrumTrack(it) => Ok(it),
            _ => Err(()),
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
