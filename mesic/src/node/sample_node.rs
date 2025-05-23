use super::extract_outputs;
use crate::graph::{PlaybackMode, ProcessContext};
use dasp_frame::Stereo;
use dasp_graph::{Buffer, Input, Node};
use std::cmp::min;
use shared::model::{Placement, SamplePlacement};
use state::{StoreData, PlacementSelector};

// Node that plays a sample.
// Distinct from buffer_node which is more general.
pub struct SampleNode {
    sel: PlacementSelector,

    // Global playback position.
    // TODO: send this down in the payload instead?
    playback_index: usize,
}

impl SampleNode {
    pub fn new(sel: PlacementSelector) -> Self {
        Self {
            sel,
            playback_index: 0,
        }
    }

    fn process_channel(&self, store: &StoreData, buffer: &mut Buffer, channel_index: usize) {
        let Some(placement): Option<&Placement> = store.try_select(&self.sel) else {
            return;
        };
        let Some(sample_placement): &Option<&SamplePlacement> = &placement.try_into().ok() else {
            return;
        };
    }
}

impl Node<ProcessContext> for SampleNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        if let Some(seek_pos) = payload.preview_seek_pos {
            self.playback_index = seek_pos;
            log::info!("Updated from seek_pos: {}", seek_pos);
        }

        let (out_left, out_right) = extract_outputs(output);

        *out_left = Buffer::SILENT;
        *out_right = Buffer::SILENT;

        self.process_channel(&payload.store, out_left, 0);
        self.process_channel(&payload.store, out_right, 1);

        self.playback_index += Buffer::LEN;
    }
}
