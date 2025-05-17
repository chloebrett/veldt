use crate::model::EffectInstance;
use crate::pmodel::*;
use crate::types::Volume;
use local_macro::{FromProto, IntoProto};

#[derive(Default, Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct Mixer {
    #[proto_optional]
    pub matrix: MixerMatrix,

    #[proto_repeated]
    pub channels: Vec<MixerChannel>,
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct MixerChannel {
    pub volume: Volume,

    #[proto_repeated]
    pub effects: Vec<EffectInstance>,
}

/// Note: distinct from ModMatrix because the mixer's matrix is somewhat symmetrical -
/// that is, inputs and outputs come from the same set (mixer channels) and therefore we
/// need special logic to avoid graph cycles.
#[derive(Clone, Default, Debug, PartialEq, FromProto, IntoProto)]
pub struct MixerMatrix {
    // TODO: this is technically repeated data, since the mixer channels list is the same length.
    // Can we deduplicate/normalise?
    #[proto_type_u32]
    pub channels: usize,

    #[proto_repeated]
    pub matrix: Vec<MatrixCell>,
}

/// NewType wrapper so that we can implement ActionReceiver for this type.
/// TODO: also consider whether this should contain OrderedFloat. I think it isn't necessary.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MatrixCell(f32);

impl MatrixCell {
    pub fn set(&mut self, value: f32) {
        self.0 = value;
    }

    pub fn deref(&self) -> f32 {
        self.0
    }

    pub fn deref_mut(&mut self) -> &mut f32 {
        &mut self.0
    }

    pub fn new(value: f32) -> Self {
        MatrixCell(value)
    }
}

impl From<f32> for MatrixCell {
    fn from(item: f32) -> Self {
        Self(item)
    }
}

impl From<MatrixCell> for f32 {
    fn from(item: MatrixCell) -> Self {
        item.0
    }
}

impl MixerMatrix {
    pub fn with_channels(channels: usize) -> Self {
        let mut matrix = MixerMatrix {
            channels,
            matrix: vec![MatrixCell(0.0); channels * channels],
        };

        // Set all inputs to the first (main) channel to 1.0.
        for row in 0..channels {
            matrix.get_mut(row, /* col= */ 0).unwrap().set(1.0);
        }

        matrix
    }

    // TODO: return an owned value, to make the caller ergonomics better.
    pub fn get(&self, row: usize, col: usize) -> Option<&MatrixCell> {
        self.matrix.get(row * self.channels + col)
    }

    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut MatrixCell> {
        self.matrix.get_mut(row * self.channels + col)
    }
}
