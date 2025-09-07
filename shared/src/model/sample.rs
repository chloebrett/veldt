use crate::bytes::{as_bytes, as_floats};
use crate::pmodel::*;
use crate::types::Beats;

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
    pub sample_rate: f32,
    pub sample_name: String,
}

impl From<SampleProto> for Sample {
    fn from(item: SampleProto) -> Self {
        Self {
            left: as_floats(&item.left),
            right: as_floats(&item.right),
            sample_rate: item.sample_rate,
            sample_name: item.sample_name,
        }
    }
}

impl From<Sample> for SampleProto {
    fn from(item: Sample) -> Self {
        Self {
            left: as_bytes(&item.left),
            right: as_bytes(&item.right),
            sample_rate: item.sample_rate,
            sample_name: item.sample_name,
        }
    }
}

// This function is copied from Mesic. Duplicating here instead of importing to avoid Mesic dependency
pub const SAMPLE_RATE: i32 = 44_100;
pub const SECONDS_PER_MINUTE: f32 = 60.0;

pub fn samples_to_beats(samples: usize, bpm: Beats) -> Beats {
    let seconds = samples as f32 / SAMPLE_RATE as f32;
    seconds * bpm / SECONDS_PER_MINUTE
}
