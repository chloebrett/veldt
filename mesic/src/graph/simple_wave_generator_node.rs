use super::ProcessContext;
use crate::wave::{beats_to_samples, unison_wave};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{GeneratorInstance, GeneratorType, Track, TrackPlacement};
use shared::types::{Beats, KnobPosition, Volume};

pub struct SimpleWaveGeneratorNode {
    instance: GeneratorInstance,
    sample_index: u32, // the sample that playback is currently up to.
    track: Track,
    track_placement: TrackPlacement,
    bpm: Beats,
    pub sample_count: usize,
}

impl SimpleWaveGeneratorNode {
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
        SimpleWaveGeneratorNode {
            instance,
            track,
            track_placement,
            bpm,
            sample_count,
            sample_index: 0,
        }
    }

    fn apply_volume_and_pan(&self, buffer: &mut Buffer, channel_index: usize) {
        let pan_mult = pan_multipliers(self.instance.meta.pan)[channel_index];
        for x in buffer.iter_mut() {
            *x *= pan_mult * self.instance.meta.volume;
        }
    }
}

impl Node<ProcessContext> for SimpleWaveGeneratorNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        if let Some(seek_pos) = payload.seek_pos {
            self.sample_index = seek_pos as u32;
        }

        // Skip generating if muted!
        // TODO: disconnect muted generators from the graph.
        let track_placement = &self.track_placement;
        if self.instance.meta.mute {
            return;
        }

        let config = match &self.instance.kind {
            GeneratorType::SimpleWave { config } => config,
            GeneratorType::Noise { .. } => todo!(),
            GeneratorType::SubSynth { .. } => todo!(),
        };

        let mut buffer = Buffer::SILENT;
        for note in &self.track.notes {
            // TODO: use a segment tree to determine which notes are in range of the current
            // buffer, instead of always iterating over all notes.
            // Then apply the same idea to tracks.
            let note_start_sample =
                beats_to_samples(*note.offset + *track_placement.offset, self.bpm);
            // Clip note end to sample_count
            // Skip notes that are outside of track sample_length
            if note_start_sample > self.sample_count as u32 {
                continue;
            }
            let note_end_sample = u32::min(
                beats_to_samples(
                    *note.offset + note.note.beats + *track_placement.offset,
                    self.bpm,
                ),
                self.sample_count as u32,
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
                    config,
                    self.sample_index as i32 - note_start_sample as i32,
                ),
            );
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
