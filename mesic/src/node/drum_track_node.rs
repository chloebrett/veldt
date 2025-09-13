use super::extract_outputs;
use crate::graph::{NoteEventType, ProcessContext};
use crate::{beats_to_samples};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{DrumTrackPlacement, PlacementType};
use state::{PlacementSelector, SampleSelector};
use std::cmp::max;

/// Node that plays a drum track.
pub struct DrumTrackPlacementNode {
    selector: PlacementSelector,
    hits: Vec<(usize, usize)>,
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
        *out_left = Buffer::SILENT;
        *out_right = Buffer::SILENT;

        let store = &payload.store;

        let Some(placement) = store.try_select(&self.selector) else {
            log::error!("Couldn't find placement: {:?}", self.selector);
            return;
        };

        let Some(drum_track_placement): &Option<&DrumTrackPlacement> = &placement.try_into().ok() else {
            log::error!("Placement wasn't a drum track placement: {:?}", self.selector);
            return;
        };

        let sample_sel = SampleSelector(drum_track_placement.sample_id);
        let Some(sample) = store.try_select(&sample_sel) else {
            log::error!("Sample doesn't exist: {:?}", drum_track_placement.sample_id);
            return;
        };

        if let Some(track_placement) = store.project.placements.values().find_map(|p| match &p.kind {
            PlacementType::Track(tp) if tp.track_id == drum_track_placement.track_id => Some(tp),
            _ => None,
        }) {
            if let Some(events) = payload.note_events.get(&track_placement.generator_id) {
                for note_event in events {
                    if note_event.kind == NoteEventType::On {
                        let placement_offset_samples =
                            beats_to_samples(*placement.offset, store.project.bpm) as usize;
                        let start_index = placement_offset_samples + note_event.sample_index;
                        self.hits.push((drum_track_placement.sample_id.0, start_index));
                    }
                }
            }
        }

        let playback_pos = payload.playback_pos;
        let sample_len = max(sample.left.len(), sample.right.len());
        let mut finished_hits = vec![];

        for i in 0..Buffer::LEN {
            let mut left_acc = 0.0;
            let mut right_acc = 0.0;

            for (idx, (_sample_id, start_index)) in self.hits.iter().enumerate() {
                let offset = i as i32 + playback_pos as i32 - *start_index as i32;
                if offset < 0 {
                    continue;
                }
                let offset = offset as usize;
                if offset >= sample_len {
                    finished_hits.push(idx);
                    continue;
                }

                left_acc += sample.left[offset];
                right_acc += sample.right[offset];
            }

            out_left[i] = left_acc;
            out_right[i] = right_acc;
        }

        for &idx in finished_hits.iter().rev() {
            if idx < self.hits.len() {
                self.hits.remove(idx);
            }
        }
    }
}
