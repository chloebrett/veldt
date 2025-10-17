use crate::convert::beats_to_samples;
use dasp_graph::Buffer;
use shared::model::{GeneratorId, PitchName, PlacementId, PlacementType, Project, TrackPlacement};
use std::cmp::min;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct NoteEvent {
    pub kind: NoteEventType,
    pub sample_index: usize,
    pub pitch_name: PitchName,
    pub pitch_offset: f32,
    pub global_start_sample_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NoteEventType {
    On,
    Off,
}

#[derive(Default)]
pub struct NoteTracker;

impl NoteTracker {
    pub fn track(
        project: &Project,
        global_sample_index: usize,
        ignore_on_events: bool,
    ) -> HashMap<GeneratorId, Vec<NoteEvent>> {
        let mut result: HashMap<GeneratorId, Vec<NoteEvent>> = HashMap::new();

        let bpm = project.bpm;

        // TODO: make this loop more efficient, instead of looping over generators one by one.
        for generator_id in project.generators.keys() {
            let placements: Vec<_> = project
                .placements
                .values()
                .filter(|placement| match &placement.kind {
                    PlacementType::Track(it) => it.generator_id == *generator_id,
                    _ => false,
                })
                .collect();

            for placement in placements {
                let &Ok(&TrackPlacement { track_id, .. }) = &placement.try_into() else {
                    continue;
                };
                let track = &project.tracks[&track_id];
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
                    if !ignore_on_events {
                        if buf_range.contains(&start_sample) {
                            result.entry(*generator_id).or_default().push({
                                NoteEvent {
                                    kind: NoteEventType::On,
                                    sample_index: start_sample as usize,
                                    pitch_name: note.note.pitch_name,
                                    pitch_offset: 0.0,
                                    global_start_sample_index: note_start_sample,
                                }
                            });
                        }
                    }

                    let end_sample = note_end_sample as isize - global_sample_index as isize;
                    if buf_range.contains(&end_sample) {
                        result.entry(*generator_id).or_default().push({
                            NoteEvent {
                                kind: NoteEventType::Off,
                                sample_index: end_sample as usize,
                                pitch_name: note.note.pitch_name,
                                pitch_offset: 0.0,
                                global_start_sample_index: note_start_sample,
                            }
                        });
                    }
                }
            }
        }
        result
    }
}

#[derive(Default)]
pub struct DrumNoteTracker;

impl DrumNoteTracker {
    pub fn track(
        project: &Project,
        global_sample_index: usize,
    ) -> HashMap<PlacementId, Vec<NoteEvent>> {
        let mut result: HashMap<PlacementId, Vec<NoteEvent>> = HashMap::new();

        let bpm = project.bpm;

        for (placement_id, placement) in &project.placements {
            if let PlacementType::DrumTrack(drum_track) = &placement.kind {
                let track = &project.tracks[&drum_track.track_id];
                let track_offset = *placement.offset;
                let track_duration = *placement
                    .clipped_duration
                    .unwrap_or(track.unclipped_duration());
                let track_end_sample = beats_to_samples(track_offset + track_duration, bpm);

                // TODO: use some kind of tree to determine which notes are in range of the current
                // buffer, instead of always iterating over all notes.
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
                        result.entry(*placement_id).or_default().push({
                            NoteEvent {
                                kind: NoteEventType::On,
                                sample_index: start_sample as usize,
                                pitch_name: note.note.pitch_name,
                                pitch_offset: note.pitch_offset,
                                global_start_sample_index: note_start_sample,
                            }
                        });
                    }

                    let end_sample = note_end_sample as isize - global_sample_index as isize;
                    if buf_range.contains(&end_sample) {
                        result.entry(*placement_id).or_default().push({
                            NoteEvent {
                                kind: NoteEventType::Off,
                                sample_index: end_sample as usize,
                                pitch_name: note.note.pitch_name,
                                pitch_offset: note.pitch_offset,
                                global_start_sample_index: note_start_sample,
                            }
                        });
                    }
                }
            }
        }
        result
    }
}
