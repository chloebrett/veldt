use crate::wave::{beats_to_samples, sub_synth_wave, unison_wave};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{GeneratorInstance, GeneratorType, Track, TrackPlacement};
use shared::types::{Beats, KnobPosition, Volume};

pub struct SubSynthNode {
    meta: GeneratorMeta,
    config: SubSynthConfig,
    sample_index: u32, // the sample that playback is currently up to.
    track: Track,
    track_placement: TrackPlacement,
    bpm: Beats,
    pub sample_count: usize,
}

impl SubSynthNode {
    pub fn new(
        instance: GeneratorInstance,
        track: Track,
        track_placement: TrackPlacement,
        bpm: Beats,
    ) -> Self {
        let sample_count = beats_to_samples(
            *track_placement.offset
                + *track_placement
                    .clipped_duration
                    .unwrap_or(track.unclipped_duration()),
            bpm,
        ) as usize;
        SubSynthNode {
            instance,
            track,
            track_placement,
            bpm,
            sample_count,
            sample_index: 0,
        }
    }
}

impl Node for SubSynthNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer]) {}
}