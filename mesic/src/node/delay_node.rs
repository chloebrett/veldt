use super::{extract_inputs, extract_outputs};
use crate::SAMPLE_RATE;
use crate::graph::ProcessContext;
use dasp_graph::{Buffer, Input, Node};
use ringbuffer::{GrowableAllocRingBuffer, RingBuffer};
use shared::model::{DelayConfig, Effect, EffectInstance};
use state::EffectSelector;

/// Similar to dasp_graph::node::Delay, except delays per-channel.
#[derive(Clone, Debug, PartialEq)]
pub struct DelayNode {
    selector: EffectSelector,
    buffers: [GrowableAllocRingBuffer<f32>; 2],
    config: DelayConfig,
    delay_samples: usize,
}

fn ms_to_samples(ms: f32) -> usize {
    (ms * (SAMPLE_RATE as f32) / 1000.0) as usize
}

impl DelayNode {
    pub fn new(selector: EffectSelector) -> Self {
        let buffer = GrowableAllocRingBuffer::new();
        Self {
            selector,
            buffers: [buffer.clone(), buffer.clone()],
            delay_samples: 0,
            config: DelayConfig::default(),
        }
    }

    fn apply(&mut self, out_buf: &mut Buffer, in_buf: &Buffer, channel_index: usize) {
        for (i, out) in out_buf.iter_mut().enumerate() {
            let buffer = &mut self.buffers[channel_index];
            *out = if buffer.len() >= self.delay_samples {
                buffer.pop_front().unwrap_or(0.0)
            } else {
                0.0
            };
            buffer.push(in_buf[i] + *out * self.config.feedback);
        }
    }
}

impl Node<ProcessContext> for DelayNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        // Apply changes from the store.
        if let Some(EffectInstance {
            it: Effect::Delay(config),
            ..
        }) = &payload.store.try_select(&self.selector)
        {
            if *config != self.config {
                self.config = config.clone();
                self.delay_samples = ms_to_samples(self.config.delay_ms);
            }
        }

        let (left_out, right_out) = extract_outputs(output);
        let (left_in, right_in) = extract_inputs(inputs)[0];

        self.apply(left_out, left_in, /* channel_index= */ 0);
        self.apply(right_out, right_in, /* channel_index= */ 1);
    }
}
