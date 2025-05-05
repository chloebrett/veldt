use super::{extract_inputs, extract_outputs};
use crate::graph::ProcessContext;
use dasp_graph::{Buffer, Input, Node};
use shared::types::Volume;

/// Node with a volume control.
/// Currently just reads from the overall project volume, but could be
/// made configurable.
/// Clips the post-gain signal.
pub struct AmpNode {
    // If none, corresponds to the main volume.
    // TODO: just have channel 0 be the main channel,
    // and therefore no need for special casing the main volume.
    // TODO: change to Option<MixerSelector> (and then just MixerSelector).
    pub channel_index: Option<usize>,
}

impl AmpNode {
    fn process_channel(&self, volume: Volume, out: &mut Buffer) {
        for x in out.iter_mut() {
            // Apply the volume multiplier.
            let mut amped = *x * volume;

            // Clip the output so that the magnitude doesn't go above 1.
            amped = amped.clamp(-1.0, 1.0);

            *x = amped
        }
    }
}

impl Node<ProcessContext> for AmpNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        let volume = if let Some(index) = self.channel_index {
            payload.store.project.mixer.channels[index].volume
        } else {
            payload.store.volume
        };

        let (out_left, out_right) = extract_outputs(output);
        let (in_left, in_right) = extract_inputs(inputs)[0];

        out_left.copy_from_slice(in_left);
        self.process_channel(volume, out_left);
        out_right.copy_from_slice(in_right);
        self.process_channel(volume, out_right);
    }
}
