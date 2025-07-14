use crate::model::EffectId;
use crate::pmodel::*;
use crate::types::Volume;
use local_macro::{FromProto, IntoProto};
use std::ops::Deref;

#[derive(Default, Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct Mixer {
    #[proto_optional]
    pub matrix: MixerMatrix,

    #[proto_repeated]
    pub channels: Vec<MixerChannel>,
}

#[derive(Clone, Default, Debug, PartialEq, FromProto, IntoProto)]
pub struct MixerChannel {
    pub volume: Volume,
    pub mute: bool,

    #[proto_repeated]
    pub effect_ids: Vec<EffectId>,
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

    pub fn new(value: f32) -> Self {
        MatrixCell(value)
    }
}

impl Deref for MatrixCell {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
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

    pub fn add_channel(&mut self, channel: usize) {
        // Add new channel input for each existing channel.
        for row in (0..self.channels).rev() {
            self.matrix.insert( row * self.channels + channel, MatrixCell(0.0))
        }
        self.channels += 1;
        // Add row with new channel outputs.
        for col in 0..self.channels {
            self.matrix.insert(channel * self.channels + col, MatrixCell(0.0));
        }
        // Set input to the main channel to 1.0.
        self.get_mut(channel, 0).unwrap().set(1.0);
    }

    pub fn delete_channel(&mut self, channel: usize) {
        // Remove channel inputs for each other channel.
        for row in (0..self.channels).rev() {
            self.matrix.remove(row * self.channels + channel);
        } 
        self.channels -= 1;
        // Remove row with channel inputs.
        for col in (0..self.channels).rev() {
            self.matrix.remove(channel * self.channels + col);
        }
    }

    // TODO: return an owned value, to make the caller ergonomics better.
    pub fn get(&self, row: usize, col: usize) -> Option<&MatrixCell> {
        self.matrix.get(row * self.channels + col)
    }

    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut MatrixCell> {
        self.matrix.get_mut(row * self.channels + col)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn new_channel_add_and_delete_to_matrix() {
        let mut matrix = MixerMatrix::with_channels(3);
        matrix.get_mut(1, 2).unwrap().set(0.25);
        matrix.get_mut(2, 0).unwrap().set(0.5);
        let cells = matrix.matrix.clone();
        matrix.add_channel(1);
        matrix.delete_channel(1);
        assert_eq!(matrix.matrix, cells)
    }

}
