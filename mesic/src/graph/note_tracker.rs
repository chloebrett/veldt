use crate::wave::beats_to_samples;
use dasp_graph::Buffer;
use shared::model::{PitchName, PlacementType, Project, TrackPlacement};
use std::cmp::min;

// Outer index is generator index.
pub type NoteEventsByGenerator = Vec<Vec<NoteEvent>>;

#[derive(Clone, Debug)]
pub struct NoteEvent {
    pub kind: NoteEventType,
    pub sample_index: usize,
    pub pitch_name: PitchName,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NoteEventType {
    On,
    Off,
}

#[derive(Default)]
pub struct NoteTracker;

impl NoteTracker {
    pub fn track(project: &Project, global_sample_index: usize) -> NoteEventsByGenerator {
        let mut result: NoteEventsByGenerator = vec![vec![]; project.generators.len()];

        let bpm = project.bpm;

        // TODO: make this loop more efficient, instead of looping over generators one by one.
        for (generator_index, result) in result.iter_mut().enumerate() {
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

                    let buf_range = 0..Buffer::LEN as isize;

                    let start_sample = note_start_sample as isize - global_sample_index as isize;
                    if buf_range.contains(&start_sample) {
                        result.push({
                            NoteEvent {
                                kind: NoteEventType::On,
                                sample_index: start_sample as usize,
                                pitch_name: note.note.pitch_name,
                            }
                        });
                    }

                    let end_sample = note_end_sample as isize - global_sample_index as isize;
                    if buf_range.contains(&end_sample) {
                        result.push({
                            NoteEvent {
                                kind: NoteEventType::Off,
                                sample_index: end_sample as usize,
                                pitch_name: note.note.pitch_name,
                            }
                        });
                    }
                }
            }
        }
        result
    }
}
