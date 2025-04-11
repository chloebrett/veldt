use crate::wave::{beats_to_samples, unison_wave};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{GeneratorInstance, GeneratorType, Track};
use shared::types::{Beats, KnobPosition, Volume};

pub struct GeneratorNode {
    instance: GeneratorInstance,
    sample_index: u32, // the sample that playback is currently up to.
    track: Track,
    bpm: Beats,
}

impl GeneratorNode {
    pub fn new(instance: GeneratorInstance, track: Track, bpm: Beats) -> Self {
        GeneratorNode {
            instance,
            track,
            bpm,
            sample_index: 0,
        }
    }
}

impl Node for GeneratorNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer]) {
        // Skip generating if muted!
        // TODO: disconnect muted generators from the graph.
        if self.instance.meta.mute {
            return;
        }

        let config = match &self.instance.kind {
            GeneratorType::SimpleWave { config } => config,
            GeneratorType::Noise { .. } => todo!(),
        };

        let mut buffer = Buffer::SILENT;
        for note in &self.track.notes {
            // TODO: use a segment tree to determine which notes are in range of the current
            // buffer, instead of always iterating over all notes.
            // Then apply the same idea to tracks.
            let note_start_sample = beats_to_samples(*note.offset, self.bpm);
            let note_end_sample = beats_to_samples(*note.offset + note.note.beats, self.bpm);

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

impl GeneratorNode {
    fn apply_volume_and_pan(&self, buffer: &mut Buffer, channel_index: usize) {
        let pan_mult = pan_multipliers(self.instance.meta.pan)[channel_index];
        for x in buffer.iter_mut() {
            *x *= pan_mult * self.instance.meta.volume;
        }
    }
}

/// TODO: use exponential pan curves, instead of linear.
fn pan_multipliers(pan: KnobPosition) -> [Volume; 2] {
    let left = 0.5 * (1.0 - pan);
    let right = 0.5 * (1.0 + pan);
    [left, right]
}
