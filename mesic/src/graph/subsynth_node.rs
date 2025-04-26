use crate::wave::{beats_to_samples, sub_synth_wave};
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
        meta: GeneratorMeta,
        config: SubSynthConfig,
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
            meta,
            config,
            track,
            track_placement,
            bpm,
            sample_count,
            sample_index: 0,
        }
    }
}

impl Node for SubSynthNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer]) {
        // Skip generating if muted!
        // TODO: disconnect muted generators from the graph.
        let track_placement = &self.track_placement;
        if self.meta.mute {
            return;
        }

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
                &sub_synth_wave(
                    &note.note.pitch_name,
                    note.note.beats,
                    self.bpm,
                    self.config,
                    self.sample_index as i32 - note_start_sample as i32,
                ),
            );
        }

        for out_buf in output.iter_mut() {
            out_buf.copy_from_slice(&buffer);
        }

        self.sample_index += Buffer::LEN as u32;
    }
}