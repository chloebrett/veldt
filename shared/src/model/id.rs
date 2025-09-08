use std::ops::Deref;

#[derive(Hash, Default, PartialEq, Eq, Ord, PartialOrd, Debug, Clone, Copy)]
pub struct GeneratorId(pub usize);

impl From<u32> for GeneratorId {
    fn from(other: u32) -> Self {
        Self(other as usize)
    }
}

impl From<GeneratorId> for u32 {
    fn from(other: GeneratorId) -> Self {
        *other as u32
    }
}

impl Deref for GeneratorId {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}

#[derive(Hash, Default, PartialEq, Eq, Ord, PartialOrd, Debug, Clone, Copy)]
pub struct PlacementId(pub usize);

impl From<u32> for PlacementId {
    fn from(other: u32) -> Self {
        Self(other as usize)
    }
}

impl From<PlacementId> for u32 {
    fn from(other: PlacementId) -> Self {
        *other as u32
    }
}

impl Deref for PlacementId {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}

#[derive(Hash, Default, PartialEq, Eq, Ord, PartialOrd, Debug, Clone, Copy)]
pub struct SampleId(pub usize);

impl From<u32> for SampleId {
    fn from(other: u32) -> Self {
        Self(other as usize)
    }
}

impl From<SampleId> for u32 {
    fn from(other: SampleId) -> Self {
        *other as u32
    }
}

impl Deref for SampleId {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}

#[derive(Hash, Default, PartialEq, Eq, Ord, PartialOrd, Debug, Clone, Copy)]
pub struct TrackId(pub usize);

impl From<u32> for TrackId {
    fn from(other: u32) -> Self {
        Self(other as usize)
    }
}

impl From<TrackId> for u32 {
    fn from(other: TrackId) -> Self {
        *other as u32
    }
}

impl Deref for TrackId {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}

#[derive(Hash, Default, PartialEq, Eq, Ord, PartialOrd, Debug, Clone, Copy)]
pub struct DrumTrackId(pub usize);

impl From<u32> for DrumTrackId {
    fn from(other: u32) -> Self {
        Self(other as usize)
    }
}

impl From<DrumTrackId> for u32 {
    fn from(other: DrumTrackId) -> Self {
        *other as u32
    }
}

impl Deref for DrumTrackId {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}

#[derive(Hash, Default, PartialEq, Eq, Ord, PartialOrd, Debug, Clone, Copy)]
pub struct EffectId(pub usize);

impl From<u32> for EffectId {
    fn from(other: u32) -> Self {
        Self(other as usize)
    }
}

impl From<EffectId> for u32 {
    fn from(other: EffectId) -> Self {
        *other as u32
    }
}

impl Deref for EffectId {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}

#[derive(Hash, Default, PartialEq, Eq, Ord, PartialOrd, Debug, Clone, Copy)]
pub struct PlacedDrumId(pub usize);

impl From<u32> for PlacedDrumId {
    fn from(other: u32) -> Self {
        Self(other as usize)
    }
}

impl From<PlacedDrumId> for u32 {
    fn from(other: PlacedDrumId) -> Self {
        *other as u32
    }
}

impl Deref for PlacedDrumId {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}
