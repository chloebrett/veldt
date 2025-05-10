use super::extract_outputs;
use crate::graph::ProcessContext;
use dasp_frame::Stereo;
use dasp_graph::{Buffer, Input, Node};
use std::cmp::min;

// Note containing a buffer which it outputs.
#[derive(Default)]
pub struct BufferNode {
    buffer: Vec<Stereo<f32>>,
    index: usize,
}

impl BufferNode {
    fn _reset(&mut self) {
        self.index = 0;
    }

    fn process_channel(&mut self, out: &mut Buffer, channel_index: usize) {
        let start_index = self.index;
        let end_index = min(start_index + Buffer::LEN, self.buffer.len());

        if end_index <= start_index {
            self.index = 0;
            return;
        }

        let size = end_index - start_index;

        for i in 0..size {
            out[i] = self.buffer[start_index + i][channel_index];
        }
    }
}

impl Node<ProcessContext> for BufferNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        if let Some(seek_pos) = payload.seek_pos {
            self.index = seek_pos;
            log::info!("Updated from seek_pos: {}", seek_pos);
        }
        // TODO: support playing samples back on arbitrary channels
        // and in arbitrary positions - not just a single main buffer.
        if payload.preview_buffer != self.buffer {
            self.buffer = payload.preview_buffer.clone();
            log::info!("Updated buffer in buffer node");
        }

        if self.buffer.is_empty() {
            return;
        }

        let (out_left, out_right) = extract_outputs(output);

        log::info!("Processing buffer, {} {}", self.buffer.len(), self.index);
        self.process_channel(out_left, 0);
        self.process_channel(out_right, 1);

        self.index += Buffer::LEN;
    }
}
