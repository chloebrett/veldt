use super::ProcessContext;
use super::{extract_inputs, extract_outputs};
use dasp_graph::{Buffer, Input, Node};
use shared::types::Volume;

/// Node with a volume control.
/// Can clip the post-gain signal if desired.
pub struct AmpNode {
    pub volume: Volume,
    pub should_clip: bool,
}

impl AmpNode {
    fn process_channel(&self, out: &mut Buffer) {
        for x in out.iter_mut() {
            // Apply the volume multiplier.
            let mut amped = *x * self.volume;

            // If applicable, clip the output so that the magnitude doesn't go above 1.
            if self.should_clip {
                amped = amped.clamp(-1.0, 1.0);
            }

            *x = amped
        }
    }
}

impl Node<ProcessContext> for AmpNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer], _payload: &ProcessContext) {
        let (out_left, out_right) = extract_outputs(output);
        let (in_left, in_right) = extract_inputs(inputs)[0];

        out_left.copy_from_slice(in_left);
        self.process_channel(out_left);
        out_right.copy_from_slice(in_right);
        self.process_channel(out_right);
    }
}
