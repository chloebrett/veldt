use dasp_graph::{Buffer, Input, Node};
use shared::types::Volume;

/// Node with a volume control.
/// Can clip the post-gain signal if desired.
pub struct AmpNode {
    pub volume: Volume,
    pub should_clip: bool,
}

impl Node for AmpNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        for (out_buf, in_buf) in output
            .iter_mut()
            .zip(inputs.first().expect("Expected one input").buffers())
        {
            out_buf.copy_from_slice(in_buf);
            for x in out_buf.iter_mut() {
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
}
