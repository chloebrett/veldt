use dasp_graph::{Buffer, Input, Node};
use shared::types::KnobPosition;

/// Mixes two inputs down to one in the given wet/dry ratio.
/// The first input is the dry signal. The second is the wet signal.
/// wet = 1.0 returns only the wet signal.
/// wet = 0.0 returns only the dry signal.
/// wet = 0.5 returns a 50/50 mix.
/// And so on.
pub struct MixerNode {
    pub wet: KnobPosition,
}

impl Node for MixerNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        debug_assert!(self.wet >= 0.0 && self.wet <= 1.0);
        let dry = 1.0 - self.wet;

        for ((out_buf, dry_buf), wet_buf) in output
            .iter_mut()
            .zip(
                inputs
                    .first()
                    .expect("Expected a dry signal as the first input")
                    .buffers(),
            )
            .zip(
                inputs
                    .get(1)
                    .expect("Expected a wet signal as the second input")
                    .buffers(),
            )
        {
            let buf: Vec<f32> = dry_buf
                .iter()
                .zip(wet_buf.iter())
                .map(|(d, w)| d * dry + w * self.wet)
                .collect();
            out_buf.copy_from_slice(&buf);
        }
    }
}
