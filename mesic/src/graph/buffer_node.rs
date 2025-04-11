use super::extract_outputs;
use dasp_graph::{Buffer, Input, Node};
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

    fn process_channel(&self, out: &mut Buffer) {
        let start_index = self.index;
        let end_index = min(start_index + Buffer::LEN, self.buffer.len());
        let size = end_index - start_index;

        if size == Buffer::LEN {
            out.copy_from_slice(&self.buffer[start_index..end_index]);
            return;
        }

        for i in 0..size {
            out[i] = self.buffer[start_index + i];
        }
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
        let (out_left, out_right) = extract_outputs(output);

        self.process_channel(out_left);
        self.process_channel(out_right);

        self.index += Buffer::LEN;
    }
}
