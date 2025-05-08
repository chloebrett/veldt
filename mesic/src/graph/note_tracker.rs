use crate::wave::beats_to_samples;
use dasp_graph::Buffer;
use shared::model::{PitchName, PlacedNote, PlacementType, Project, TrackPlacement};
use shared::types::Beats;
use std::cmp::min;

// Outer index is generator index.
pub type NoteEventsByGenerator = Vec<Vec<NoteEvent>>;

#[derive(Clone)]
pub struct NoteEvent {
    pub pitch_name: PitchName,

    // Global values.
    // Can be negative.
    pub samples_since_started: i32,
    pub duration: Beats,

    // Local values.
    // Within the current buffer. Always 0 - 64.
    pub start_sample: usize,
    // Within the current buffer. Always 0 - 64.
    pub end_sample: usize,
}

#[derive(Clone, Debug)]
pub struct NoteEvent2 {
    pub kind: NoteEventType,
    pub sample_index: usize,
}

#[derive(Clone, Debug)]
pub enum NoteEventType {
    On { note: PlacedNote },
    Off,
}

#[derive(Default)]
pub struct NoteTracker;

impl NoteTracker {
    pub fn track(project: &Project, global_sample_index: usize) -> NoteEventsByGenerator {
        let mut result: NoteEventsByGenerator = vec![vec![]; project.generators.len()];

        let bpm = project.bpm;

        // TODO: make this loop more efficient, instead of looping over generators one by one.
        for generator_index in 0..project.generators.len() {
            let placements: Vec<_> = project
                .placements
                .clone()
                .into_iter()
                .filter(|it| match &it.kind {
                    PlacementType::Track(it) => it.generator_index == generator_index,
                    _ => false,
                })
                .collect();

            for placement in &placements {
                let &Ok(&TrackPlacement { track_index, .. }) = &placement.try_into() else {
                    continue;
                };
                let track = &project.tracks[track_index];
                let track_offset = *placement.offset;
                let track_duration = *placement
                    .clipped_duration
                    .unwrap_or(track.unclipped_duration());
                let track_end_sample = beats_to_samples(track_offset + track_duration, bpm);

                // TODO: use some kind of tree to determine which notes are in range of the current
                // buffer, instead of always iterating over all notes.
                // Then apply the same idea to tracks.
                for note in &track.notes {
                    let offset = beats_to_samples(track_offset + *note.offset, bpm);
                    let note_start_sample = min(offset, track_end_sample) as usize;
                    let note_end_sample = min(
                        offset + beats_to_samples(note.note.beats, bpm),
                        track_end_sample,
                    ) as usize;

                    // Don't play notes that aren't relevant to this buffer segment.
                    if note_start_sample > global_sample_index + Buffer::LEN
                        || note_end_sample < global_sample_index
                    {
                        continue;
                    }

                    let start_sample = (note_start_sample as i32 - global_sample_index as i32)
                        .clamp(0, Buffer::LEN as i32)
                        as usize;
                    let end_sample = (note_end_sample as i32 - global_sample_index as i32)
                        .clamp(0, Buffer::LEN as i32) as usize;

                    let event = NoteEvent {
                        pitch_name: note.note.pitch_name,
                        samples_since_started: global_sample_index as i32
                            - note_start_sample as i32,
                        duration: note.note.beats,
                        start_sample,
                        end_sample,
                    };

                    result[generator_index].push(event);
                }
            }
        }
        result
    }

    pub fn track2(project: &Project, global_sample_index: usize) -> Vec<Vec<NoteEvent2>> {
        let mut result: Vec<Vec<NoteEvent2>> = vec![vec![]; project.generators.len()];

        let bpm = project.bpm;

        // TODO: make this loop more efficient, instead of looping over generators one by one.
        for generator_index in 0..project.generators.len() {
            let placements: Vec<_> = project
                .placements
                .clone()
                .into_iter()
                .filter(|it| match &it.kind {
                    PlacementType::Track(it) => it.generator_index == generator_index,
                    _ => false,
                })
                .collect();

            for placement in &placements {
                let &Ok(&TrackPlacement { track_index, .. }) = &placement.try_into() else {
                    continue;
                };
                let track = &project.tracks[track_index];
                let track_offset = *placement.offset;
                let track_duration = *placement
                    .clipped_duration
                    .unwrap_or(track.unclipped_duration());
                let track_end_sample = beats_to_samples(track_offset + track_duration, bpm);

                // TODO: use some kind of tree to determine which notes are in range of the current
                // buffer, instead of always iterating over all notes.
                // Then apply the same idea to tracks.
                for note in &track.notes {
                    let offset = beats_to_samples(track_offset + *note.offset, bpm);
                    let note_start_sample = min(offset, track_end_sample) as usize;
                    let note_end_sample = min(
                        offset + beats_to_samples(note.note.beats, bpm),
                        track_end_sample,
                    ) as usize;

                    // Don't play notes that aren't relevant to this buffer segment.
                    if note_start_sample > global_sample_index + Buffer::LEN
                        || note_end_sample < global_sample_index
                    {
                        continue;
                    }

                    let start_sample = note_start_sample as isize - global_sample_index as isize;
                    let end_sample = note_end_sample as isize - global_sample_index as isize;

                    let buf_range = 0..Buffer::LEN as isize;

                    if buf_range.contains(&start_sample) {
                        result[generator_index].push({
                            NoteEvent2 {
                                kind: NoteEventType::On { note: note.clone() },
                                sample_index: start_sample as usize,
                            }
                        });
                    }

                    if buf_range.contains(&end_sample) {
                        result[generator_index].push({
                            NoteEvent2 {
                                kind: NoteEventType::Off,
                                sample_index: end_sample as usize,
                            }
                        });
                    }
                }
            }
        }
        result
    }
}
