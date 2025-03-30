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
            let buf: Vec<f32> = in_buf
                .iter()
                .map(|it| {
                    let mut amped = it * self.volume;
                    if self.should_clip {
                        amped = amped.clamp(-1.0, 1.0);
                    }
                    amped
                })
                .collect();
            out_buf.copy_from_slice(&buf);
        }
    }
}
