use super::dual_channel;
use dasp_graph::{Buffer, Input, Node};
use ringbuffer::{AllocRingBuffer, RingBuffer};

/// Similar to dasp_graph::node::Delay, except delays per-channel.
#[derive(Clone, Debug, PartialEq)]
pub struct DelayNode {
    buffers: [AllocRingBuffer<f32>; 2],
}

impl DelayNode {
    pub fn new(delay_samples: usize) -> DelayNode {
        let buffer = AllocRingBuffer::from(vec![0.0; delay_samples]);
        DelayNode {
            buffers: [buffer.clone(), buffer.clone()],
        }
    }

    fn apply(&mut self, out_buf: &mut Buffer, in_buf: &Buffer, channel_index: usize) {
        for (i, out) in out_buf.iter_mut().enumerate() {
            let buffer = &mut self.buffers[channel_index];
            *out = *buffer.front().unwrap();
            buffer.push(in_buf[i]);
        }
    }
}

impl Node for DelayNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        let (left_out, left_in, right_out, right_in) = dual_channel(inputs, output);

        self.apply(left_out, left_in, /* channel_index= */ 0);
        self.apply(right_out, right_in, /* channel_index= */ 1);
    }
}
