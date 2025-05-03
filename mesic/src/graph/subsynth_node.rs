use crate::graph::{ProcessContext, pan_multipliers};
use crate::wave::{WaveSource, beats_to_samples};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{
    Generator, GeneratorInstance, GeneratorMeta, Placement, SubSynthConfig, Track, TrackPlacement,
};
use shared::types::Beats;
use std::cmp::min;

pub struct SubSynthNode {
    wave_source: WaveSource,
    config: SubSynthConfig,
    meta: GeneratorMeta,
    generator_index: usize,
    sample_index: u32, // the sample that playback is currently up to.
    placements: Vec<Placement>,
    tracks: Vec<Track>,
    bpm: Beats,
}

impl SubSynthNode {
    pub fn new(
        config: SubSynthConfig,
        meta: GeneratorMeta,
        generator_index: usize,
        placements: Vec<Placement>,
        tracks: Vec<Track>,
        bpm: Beats,
    ) -> Self {
        Self {
            wave_source: WaveSource::default(),
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

// TODO: most of the logic is the same as simple wave, need to move this elsewhere
impl Node<ProcessContext> for SubSynthNode {
    // TODO: a lot of this processing logic is generic and should be shared with
    // other generator types. How?
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        if let Some(seek_pos) = payload.seek_pos {
            self.sample_index = seek_pos as u32;
        }

        // Apply any applicable changes from the store.
        if let Some(GeneratorInstance {
            it: Generator::SubSynth(config),
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
            let &Ok(&TrackPlacement { track_index, .. }) = &placement.try_into() else {
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
                    &self.wave_source.subsynth_wave(
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
