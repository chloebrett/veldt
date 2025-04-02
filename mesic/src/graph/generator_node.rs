use crate::consts::SAMPLE_RATE;
use crate::wave::polyphonic_wave;
use dasp_graph::{Buffer, Input, Node};
use shared::model::{GeneratorInstance, GeneratorType, Track};
use shared::types::Beats;

pub struct GeneratorNode {
    instance: GeneratorInstance,
    sample_index: usize, // the sample that playback is currently up to.
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
        let mut buffer: Vec<f32> = vec![0.0; track_samples];

        let config = match &self.instance.kind {
            GeneratorType::SimpleWave { config } => config,
        };

        for note in &self.track.notes {
            let wave = polyphonic_wave(
                &note.note.pitch_name,
                note.note.beats,
                self.bpm,
                self.instance.meta.volume,
                config,
                self.sample_index,
            );

            let offset_samples = *note.offset / self.bpm * 60.0 * SAMPLE_RATE as f32;
            wave.iter().enumerate().for_each(|(i, value)| {
                // Limit the output of the wave to the next 64 samples only.
                // TODO: do this truncation inside the wave function itself, as this is very inefficient!
                if i < Buffer::LEN {
                    buffer[i + offset_samples as usize] += value;
                }
            })
        }

        for out_buf in output {
            out_buf.copy_from_slice(&buffer[0..Buffer::LEN]);
        }
        self.sample_index += Buffer::LEN;
    }
}
