use crate::SAMPLE_RATE;
use crate::consts::SECONDS_PER_MINUTE;
use shared::types::Beats;

pub fn beats_to_samples(beats: Beats, bpm: Beats) -> u32 {
    let seconds = beats / bpm * SECONDS_PER_MINUTE;
    (SAMPLE_RATE as f32 * seconds) as u32
}
