use super::ProcessContext;
use super::{extract_inputs, extract_outputs};
use dasp_graph::{Buffer, Input, Node};
use ringbuffer::{GrowableAllocRingBuffer, RingBuffer};
use shared::types::Volume;

/// Similar to dasp_graph::node::Delay, except delays per-channel.
#[derive(Clone, Debug, PartialEq)]
pub struct DelayNode {
    buffers: [GrowableAllocRingBuffer<f32>; 2],
    delay_samples: usize,
    feedback: Volume,
}

impl DelayNode {
    pub fn new(delay_samples: usize, feedback: f32) -> DelayNode {
        let buffer = GrowableAllocRingBuffer::with_capacity(delay_samples);
        DelayNode {
            buffers: [buffer.clone(), buffer.clone()],
            delay_samples,
            feedback,
        }
    }

    fn apply(&mut self, out_buf: &mut Buffer, in_buf: &Buffer, channel_index: usize) {
        for (i, out) in out_buf.iter_mut().enumerate() {
            let buffer = &mut self.buffers[channel_index];
            *out = if buffer.len() >= self.delay_samples {
                buffer.pop_front().unwrap()
            } else {
                0.0
            };
            buffer.push(in_buf[i] + *out * self.feedback);
        }
    }
}

impl Node<ProcessContext> for DelayNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer], _payload: &ProcessContext) {
        let (left_out, right_out) = extract_outputs(output);
        let (left_in, right_in) = extract_inputs(inputs)[0];

        self.apply(left_out, left_in, /* channel_index= */ 0);
        self.apply(right_out, right_in, /* channel_index= */ 1);
    }
}
