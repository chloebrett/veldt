use super::extract_outputs;
use crate::graph::ProcessContext;
use crate::{beats_to_samples, samples_to_beats};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{Placement, DrumTrack};
use state::{PlacementSelector, SampleSelector};

/// Node that plays a drum track.
pub struct DrumTrackNode {
    selector: PlacementSelector,
}

impl DrumTrackNode {
    pub fn new(sel: PlacementSelector) -> Self {
        Self { selector: sel }
    }
}

impl Node<ProcessContext> for DrumTrackNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        let playback_pos = payload.playback_pos;

        let (out_left, out_right) = extract_outputs(output);
        *out_left = Buffer::SILENT;
        *out_right = Buffer::SILENT;

        let store = &payload.store;
        let Some(placement): Option<&Placement> = store.try_select(&self.selector) else {
            log::error!("Couldn't find placement: {:?}", self.selector);
            return;
        };

        let Ok(drum_track) = <&DrumTrack>::try_from(placement) else {
            log::error!("Placement wasn't a drum track: {:?}", self.selector);
            return;
        };

        for (sample_id, sub_track) in &drum_track.drum_sub_tracks {
            let sample_sel = SampleSelector(*sample_id);
            let Some(sample) = store.try_select(&sample_sel) else {
                log::warn!("Sample missing: {:?}", sample_id);
                continue;
            };
        }
    }
}
