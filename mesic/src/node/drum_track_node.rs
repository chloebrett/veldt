use super::extract_outputs;
use crate::graph::{NoteEventType, ProcessContext};
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

        let track_placement = store
            .project
            .placements
            .values()
            .find_map(|placement| match &placement.kind {
                PlacementType::Track(track_placement)
                    if track_placement.track_id == drum_track_placement.track_id =>
                {
                    Some(track_placement)
                }
                _ => None,
            })
            .expect("DrumTrackPlacement must have a corresponding TrackPlacement");

        if let Some(events) = payload.note_events.get(&track_placement.generator_id) {
            for note_event in events {
                if note_event.kind == NoteEventType::On {
                    self.hits.push((drum_track_placement.sample_id.0, 0));
                } else {
                    self.hits.clear();
                }
            }
        }

        let sample_len = max(sample.left.len(), sample.right.len());
        let mut finished = vec![];

        for i in 0..Buffer::LEN {
            let mut left_acc = 0.0;
            let mut right_acc = 0.0;

            for (idx, (sample_id, playback_index)) in self.hits.iter_mut().enumerate() {
                if *sample_id != drum_track_placement.sample_id.0 {
                    continue;
                }
                if *playback_index >= sample_len {
                    finished.push(idx);
                    continue;
                }

                left_acc += sample.left.get(*playback_index).copied().unwrap_or(0.0);
                right_acc += sample.right.get(*playback_index).copied().unwrap_or(0.0);
                *playback_index += 1;
            }

            out_left[i] += left_acc;
            out_right[i] += right_acc;
        }

        for &idx in finished.iter().rev() {
            self.hits.remove(idx);
        }
    }
}
