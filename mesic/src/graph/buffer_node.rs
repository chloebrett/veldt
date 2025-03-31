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
