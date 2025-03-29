use crate::consts::REFERENCE_PITCH;
use crate::effect::ApplyEffect;
use shared::model::Effect;
use shared::model::{EffectInstance, PitchName};
use shared::types::{Freq, PitchValue, Volume};
use std::cmp::{max, min};
use dasp_graph::{Node, Input, Buffer, NodeData};

// TODO: rename sig.rs to graph.rs.

// TODO: consider StableGraph.
pub type Graph = petgraph::graph::DiGraph<NodeData<Box<dyn Node>>, (), u32>;

pub type Processor = dasp_graph::Processor<Graph>;

// Note containing a buffer which it outputs.
// TODO: make fields private.
pub struct BufferNode {
    pub buffer: Vec<f32>,
    pub index: usize,
}

impl BufferNode {
    fn reset(&mut self) {
        self.index = 0;
    }
}

impl Node for BufferNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer]) {
        for out_buf in output {
            let index = self.index;
            let slice = &self.buffer[index .. min(index, self.buffer.len())];
            out_buf.copy_from_slice(slice);
        }
    }
}

pub struct EffectNode {
    instance: EffectInstance,
}

impl Node for EffectNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        for (out_buf, in_buf) in output.iter_mut().zip(inputs.get(0).expect("Expected one input").buffers()) {
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
        for (out_buf, in_buf) in output.iter_mut().zip(inputs.get(0).expect("Expected one input").buffers()) {
            let buf: Vec<f32> = 
                in_buf
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

pub fn sum(a: &[f32], b: &[f32]) -> Vec<f32> {
    let max_len = max(a.len(), b.len());
    let mut output: Vec<f32> = vec![0.0; max_len];

    for (i, item) in output.iter_mut().enumerate() {
        let ai = a.get(i).unwrap_or(&0.0);
        let bi = b.get(i).unwrap_or(&0.0);
        *item = ai + bi;
    }

    output
}

pub fn mult(vec: &[f32], scalar: f32) -> Vec<f32> {
    vec.iter().map(|it| it * scalar).collect()
}

// Returns the frequency based on the distance from reference pitch.
pub fn freq(pitch_name: PitchName) -> Freq {
    let pitch: PitchValue = pitch_name.into();
    let reference: PitchValue = (*REFERENCE_PITCH.pitch_name).into();
    let interval: PitchValue = pitch - reference;

    // powf can't be run at compile time.
    // TODO: make this only run once.
    let semitone_increment: f32 = 2.0_f32.powf(1.0 / 12.0);
    REFERENCE_PITCH.frequency * semitone_increment.powf(interval as f32)
}
