use super::ProcessContext;
use crate::wave::{beats_to_samples, unison_wave};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{
    GeneratorInstance, GeneratorMeta, GeneratorType, Placement, SimpleWaveConfig, Track,
    TrackPlacement,
};
use shared::types::{Beats, KnobPosition, Volume};
use std::cmp::min;

pub struct SimpleWaveGeneratorNode {
    config: SimpleWaveConfig,
    meta: GeneratorMeta,
    generator_index: usize,
    sample_index: u32, // the sample that playback is currently up to.
    placements: Vec<Placement>,
    tracks: Vec<Track>,
    bpm: Beats,
}

impl SimpleWaveGeneratorNode {
    pub fn new(
        config: SimpleWaveConfig,
        meta: GeneratorMeta,
        generator_index: usize,
        placements: Vec<Placement>,
        tracks: Vec<Track>,
        bpm: Beats,
    ) -> Self {
        SimpleWaveGeneratorNode {
            config,
            meta,
            generator_index,
            placements,
            tracks,
            bpm,
            sample_index: 0,
        }
    }

    fn apply_volume_and_pan(&self, buffer: &mut Buffer, channel_index: usize) {
        let pan_mult = pan_multipliers(self.meta.pan)[channel_index];
        for x in buffer.iter_mut() {
            *x *= pan_mult * self.meta.volume;
        }
    }
}

impl Node<ProcessContext> for SimpleWaveGeneratorNode {
    // TODO: a lot of this processing logic is generic and should be shared with
    // other generator types. How?
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        if let Some(seek_pos) = payload.seek_pos {
            self.sample_index = seek_pos as u32;
        }

        // Apply any applicable changes from the store.
        if let Some(GeneratorInstance {
            kind: GeneratorType::SimpleWave { config },
            meta,
            ..
        }) = &payload.store.project.generators.get(self.generator_index)
        {
            if *config != self.config {
                self.config = config.clone();
            }
            if *meta != self.meta {
                self.meta = meta.clone();
            }
        }

        // Skip generating if muted!
        // TODO: disconnect muted generators from the graph.
        if self.meta.mute {
            return;
        }

        let mut buffer = Buffer::SILENT;
        for placement in &self.placements {
            let Ok(TrackPlacement { track_index, .. }) = placement.try_into() else {
                continue;
            };
            let track = &self.tracks[track_index];
            let track_offset = *placement.offset;
            let track_duration = *placement
                .clipped_duration
                .unwrap_or(track.unclipped_duration());
            let track_end_sample = beats_to_samples(track_offset + track_duration, self.bpm);

            // TODO: use a segment tree to determine which notes are in range of the current
            // buffer, instead of always iterating over all notes.
            // Then apply the same idea to tracks.
            for note in &track.notes {
                let offset = track_offset + *note.offset;
                let note_start_sample = min(beats_to_samples(offset, self.bpm), track_end_sample);
                let note_end_sample = min(
                    beats_to_samples(offset + note.note.beats, self.bpm),
                    track_end_sample,
                );

                // Don't play notes that aren't relevant to this buffer segment.
                if note_start_sample > self.sample_index + Buffer::LEN as u32
                    || note_end_sample < self.sample_index
                {
                    continue;
                }

                dasp_slice::add_in_place(
                    &mut buffer,
                    &unison_wave(
                        &note.note.pitch_name,
                        note.note.beats,
                        self.bpm,
                        &self.config,
                        self.sample_index as i32 - note_start_sample as i32,
                    ),
                );
            }
        }

        for (channel_index, out_buf) in output.iter_mut().enumerate() {
            out_buf.copy_from_slice(&buffer);
            self.apply_volume_and_pan(out_buf, channel_index);
        }
        self.sample_index += Buffer::LEN as u32;
    }
}

/// TODO: use exponential pan curves, instead of linear.
fn pan_multipliers(pan: KnobPosition) -> [Volume; 2] {
    let left = 0.5 * (1.0 - pan);
    let right = 0.5 * (1.0 + pan);
    [left, right]
}
