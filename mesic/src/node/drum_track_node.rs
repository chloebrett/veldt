use super::extract_outputs;
use crate::beats_to_samples;
use crate::graph::{NoteEventType, ProcessContext};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{DrumTrackPlacement, PlacementType};
use state::{PlacementSelector, SampleSelector};
use std::cmp::max;

/// Node that plays a drum track.
pub struct DrumTrackPlacementNode {
    selector: PlacementSelector,
    hits: Vec<usize>, // stores start index of pending hits
}

impl DrumTrackPlacementNode {
    pub fn new(sel: PlacementSelector) -> Self {
        Self {
            selector: sel,
            hits: Vec::new(),
        }
    }
}

impl Node<ProcessContext> for DrumTrackPlacementNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        let (out_left, out_right) = extract_outputs(output);
        out_left.fill(0.0);
        out_right.fill(0.0);

        let store = &payload.store;
        let Some(placement) = store.try_select(&self.selector) else {
            log::error!("Couldn't find placement: {:?}", self.selector);
            return;
        };

        let Some(drum_track_placement): &Option<&DrumTrackPlacement> = &placement.try_into().ok()
        else {
            log::error!(
                "Placement wasn't a drum track placement: {:?}",
                self.selector
            );
            return;
        };

        let sample_sel = SampleSelector(drum_track_placement.sample_id);
        let Some(sample) = store.try_select(&sample_sel) else {
            log::error!("Sample doesn't exist: {:?}", drum_track_placement.sample_id);
            return;
        };

        if let Some(track_placement) =
            store
                .project
                .placements
                .values()
                .find_map(|p| match &p.kind {
                    PlacementType::Track(tp) if tp.track_id == drum_track_placement.track_id => {
                        Some(tp)
                    }
                    _ => None,
                })
        {
            if let Some(events) = payload.note_events.get(&track_placement.generator_id) {
                let placement_offset_samples =
                    beats_to_samples(*placement.offset, store.project.bpm) as usize;

                for note_event in events {
                    if note_event.kind == NoteEventType::On {
                        let start_index = payload.playback_pos
                            + placement_offset_samples
                            + note_event.sample_index;
                        self.hits.push(start_index);
                    }
                }
            }
        }

        let sample_len = max(sample.left.len(), sample.right.len());
        let playback_pos = payload.playback_pos;
        let buffer_len = out_left.len();

        let remaining_hits = self
            .hits
            .iter()
            .copied()
            .filter(|&hit_start| hit_start + sample_len > playback_pos as usize)
            .collect::<Vec<_>>();

        for i in 0..buffer_len {
            let sample_index: usize = playback_pos as usize + i;
            let mut left_acc = 0.0;
            let mut right_acc = 0.0;

            for &hit_start in &remaining_hits {
                let hit_end = hit_start + sample_len;
                if sample_index >= hit_start && sample_index < hit_end {
                    let sample_offset = sample_index - hit_start;
                    left_acc += sample.left[sample_offset];
                    right_acc += sample.right[sample_offset];
                }
            }

            out_left[i] = left_acc;
            out_right[i] = right_acc;
        }

        self.hits = remaining_hits;
    }
}
