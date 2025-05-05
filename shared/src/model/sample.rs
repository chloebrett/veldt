use crate::bytes::{as_bytes, as_floats};
use crate::pmodel::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
    pub sample_rate: f32,
}

impl From<SampleProto> for Sample {
    fn from(item: SampleProto) -> Self {
        Self {
            left: as_floats(&item.left),
            right: as_floats(&item.right),
            sample_rate: item.sample_rate,
        }
    }
}

impl From<Sample> for SampleProto {
    fn from(item: Sample) -> Self {
        Self {
            left: as_bytes(&item.left),
            right: as_bytes(&item.right),
            sample_rate: item.sample_rate,
        }
    }
}
