use super::ProcessContext;
use super::extract_outputs;
use dasp_frame::Stereo;
use dasp_graph::{Buffer, Input, Node};
use std::cmp::min;

// Note containing a buffer which it outputs.
pub struct BufferNode {
    buffer: Vec<Stereo<f32>>,
    index: usize,
}

impl BufferNode {
    fn _reset(&mut self) {
        self.index = 0;
    }

    fn process_channel(&self, out: &mut Buffer, channel_index: usize) {
        let start_index = self.index;
        let end_index = min(start_index + Buffer::LEN, self.buffer.len());
        let size = end_index - start_index;

        if end_index <= start_index {
            return;
        }

        for i in 0..size {
            out[i] = self.buffer[start_index + i][channel_index];
        }
    }
}

impl From<Vec<Stereo<f32>>> for BufferNode {
    fn from(item: Vec<Stereo<f32>>) -> Self {
        BufferNode {
            buffer: item,
            index: 0,
        }
    }
}

impl Node<ProcessContext> for BufferNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], _payload: &ProcessContext) {
        let (out_left, out_right) = extract_outputs(output);

        self.process_channel(out_left, 0);
        self.process_channel(out_right, 1);

        self.index += Buffer::LEN;
    }
}
