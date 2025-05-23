use super::extract_outputs;
use crate::graph::ProcessContext;
use crate::{beats_to_samples, samples_to_beats};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{Placement, SamplePlacement};
use state::{PlacementSelector, SampleSelector};
use std::cmp::max;

// Node that plays a sample.
// Distinct from buffer_node which is more general.
pub struct SampleNode {
    sel: PlacementSelector,
}

impl SampleNode {
    pub fn new(sel: PlacementSelector) -> Self {
        Self { sel }
    }
}

impl Node<ProcessContext> for SampleNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        let playback_pos = payload.playback_pos;

        let (out_left, out_right) = extract_outputs(output);

        *out_left = Buffer::SILENT;
        *out_right = Buffer::SILENT;

        let store = &payload.store;
        let Some(placement): Option<&Placement> = store.try_select(&self.sel) else {
            log::error!("Couldn't find placement: {:?}", self.sel);
            return;
        };
        let Some(sample_placement): &Option<&SamplePlacement> = &placement.try_into().ok() else {
            log::error!("Placement wasn't a sample placement: {:?}", self.sel);
            return;
        };
        let sample_sel = SampleSelector(sample_placement.sample_index);
        let Some(sample) = &store.try_select(&sample_sel) else {
            log::error!("Sample doesn't exist: {:?}", sample_placement.sample_index);
            return;
        };

        let unclipped_duration = samples_to_beats(
            max(sample.left.len(), sample.right.len()),
            store.project.bpm,
        );
        let duration = placement
            .clipped_duration
            .unwrap_or(unclipped_duration.into());
        let duration_samples = beats_to_samples(*duration, store.project.bpm);

        let sample_start_index = beats_to_samples(*placement.offset, store.project.bpm);

        for i in 0..Buffer::LEN {
            let offset: i32 = i as i32 + playback_pos as i32 - sample_start_index as i32;

            if offset < 0 || offset > duration_samples as i32 {
                continue;
            }
            let offset = offset as usize;

            out_left[i] = *sample.left.get(offset).unwrap_or(&0.0);
            out_right[i] = *sample.right.get(offset).unwrap_or(&0.0);
        }
    }
}
