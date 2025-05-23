use crate::SAMPLE_RATE;
use crate::consts::SECONDS_PER_MINUTE;
use dasp_frame::Stereo;
use shared::types::Beats;

pub fn beats_to_samples(beats: Beats, bpm: Beats) -> u32 {
    let seconds = beats / bpm * SECONDS_PER_MINUTE;
    (SAMPLE_RATE as f32 * seconds) as u32
}

pub fn samples_to_beats(samples: usize, bpm: Beats) -> Beats {
    let seconds = samples as f32 / SAMPLE_RATE as f32;
    seconds * bpm / SECONDS_PER_MINUTE
}

/// Zip left and right audio into a single stereo signal vector.
pub fn interleave_stereo(left: Vec<f32>, right: Vec<f32>) -> Vec<Stereo<f32>> {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| [*left, *right])
        .collect()
}

/// Convert a stereo signal into two vectors for left and right.
pub fn split_stereo_audio(stereo_audio: &[[f32; 2]]) -> (Vec<f32>, Vec<f32>) {
    stereo_audio.iter().map(|it| (it[0], it[1])).unzip()
}

/// Convert linear sound power level to decibels (dB) (e.g. RMS).
pub fn to_db(input: f32) -> f32 {
    20.0 * input.log10()
}

pub fn from_db(input: f32) -> f32 {
    10f32.powf(input / 20.0)
}
