use crate::consts::SAMPLE_RATE;
use crate::wave::{beats_to_samples, polyphonic_wave};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{GeneratorInstance, GeneratorType, Track};
use shared::types::Beats;

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
        let track_beats: f32 = self
            .track
            .notes
            .iter()
            .map(|placed_note| {
                let offset: f32 = placed_note.offset.into();
                offset + placed_note.note.beats
            })
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.0);
        let track_samples = (track_beats / self.bpm * 60.0 * SAMPLE_RATE as f32) as usize;

        let config = match &self.instance.kind {
            GeneratorType::SimpleWave { config } => config,
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
                &polyphonic_wave(
                    &note.note.pitch_name,
                    note.note.beats,
                    self.bpm,
                    self.instance.meta.volume,
                    config,
                    self.sample_index as i32 - note_start_sample as i32,
                ),
            );
        }

        for out_buf in output {
            out_buf.copy_from_slice(&buffer);
        }
        self.sample_index += Buffer::LEN as u32;
    }
}
