use crate::effect::ApplyEffect;
use dasp_graph::{BoxedNode, Buffer, Input, Node, NodeData};
use shared::model::Effect;
use shared::model::EffectInstance;
use shared::types::Volume;
use std::cmp::min;

pub type Graph = petgraph::stable_graph::StableGraph<NodeData<BoxedNode>, ()>;

pub type Processor = dasp_graph::Processor<Graph>;

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
            let index = self.index;
            let slice = &self.buffer[index..min(index + Buffer::LEN, self.buffer.len())];

            // TODO: account for edge cases.
            if out_buf.len() == slice.len() {
                out_buf.copy_from_slice(slice);
            }
        }
        self.index += Buffer::LEN;
    }
}

pub struct EffectNode {
    instance: EffectInstance,
}

impl Node for EffectNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        for (out_buf, in_buf) in output
            .iter_mut()
            .zip(inputs.get(0).expect("Expected one input").buffers())
        {
            let buf = match &self.instance.effect {
                // TODO: use a dasp_graph Delay node.
                Effect::SimpleDelay { config } => &config.apply(in_buf),
                // TODO: store state on the EQ nodes, so that they don't lose track
                // of their state every 64 samples.
                Effect::SimpleEq { config } => &config.apply(in_buf),
                _ => panic!("Effect not implemented yet!"),
            };
            out_buf.copy_from_slice(buf);
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
            .zip(inputs.get(0).expect("Expected one input").buffers())
        {
            let buf: Vec<f32> = in_buf
                .into_iter()
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
