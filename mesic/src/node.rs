use crate::effect::ApplyFilter;
use dasp_graph::{Buffer, Input, Node};
use shared::types::{KnobPosition, Volume};
use std::cmp::min;

// Note containing a buffer which it outputs.
pub struct BufferNode {
    buffer: Vec<f32>,
    index: usize,
}

impl BufferNode {
    fn _reset(&mut self) {
        self.index = 0;
    }
}

impl From<Vec<f32>> for BufferNode {
    fn from(item: Vec<f32>) -> Self {
        BufferNode {
            buffer: item,
            index: 0,
        }
    }
}

impl Node for BufferNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer]) {
        for out_buf in output {
            let start_index = self.index;
            let end_index = min(start_index + Buffer::LEN, self.buffer.len());

            let mut slice = if start_index < end_index {
                &self.buffer[start_index..end_index]
            } else {
                &vec![]
            };

            // If the slice isn't long enough, fill the rest with zeroes.
            let mut vec;
            if slice.len() < Buffer::LEN {
                vec = slice.to_vec();
                vec.resize(Buffer::LEN, 0.0);
                slice = &vec;
            }

            if slice.len() > Buffer::LEN {
                panic!(
                    "Got a slice with length {}, which is more than {} and shouldn't happen!",
                    slice.len(),
                    Buffer::LEN
                );
            }

            out_buf.copy_from_slice(slice);
        }
        self.index += Buffer::LEN;
    }
}

pub struct EqNode {
    pub filter: Box<dyn ApplyFilter>,
}

impl Node for EqNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        for (out_buf, in_buf) in output
            .iter_mut()
            .zip(inputs.first().expect("Expected one input").buffers())
        {
            let buf = self.filter.apply(in_buf);
            out_buf.copy_from_slice(&buf);
        }
    }
}

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
