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
            let size = end_index - start_index;

            if size == Buffer::LEN {
                out_buf.copy_from_slice(&self.buffer[start_index..end_index]);
                continue;
            }

            for i in 0..size {
                out_buf[i] = self.buffer[start_index + i];
            }
        }
        self.index += Buffer::LEN;
    }
}
