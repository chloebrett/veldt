use super::{extract_inputs, extract_outputs};
use crate::consts::RECIP_SAMPLE_RATE;
use crate::graph::ProcessContext;
use crate::wave::make_wave;
use dasp_graph::{Buffer, Input, Node};
use ringbuffer::{GrowableAllocRingBuffer, RingBuffer};
use shared::model::{AntiAliasingMode, Effect, EffectInstance, ModDelayConfig};
use state::EffectSelector;

/// Modulated delay - i.e. vibrato, flanger, phaser, chorus.
pub struct ModDelayNode {
    selector: EffectSelector,
    config: ModDelayConfig,
    lfo_phase: f32, // current phase of the LFO. Ranges from 0 to 1 then loops back to 0.
    buffers: [GrowableAllocRingBuffer<f32>; 2],
}

impl ModDelayNode {
    pub fn new(selector: EffectSelector) -> Self {
        let buffer = GrowableAllocRingBuffer::new();
        Self {
            selector,
            config: ModDelayConfig::default(),
            lfo_phase: 0.0,
            buffers: [buffer.clone(), buffer.clone()],
        }
    }

    fn apply(&mut self, out_buf: &mut Buffer, in_buf: &Buffer, channel_index: usize) {
        out_buf.copy_from_slice(in_buf);
        let config = &self.config;
        let mut lfo_phase = self.lfo_phase;
        let mod_depth = config.max_depth - config.min_depth;
        let mid_depth = (config.max_depth + config.min_depth) / 2;

        for x in out_buf.iter_mut() {
            let buffer = &mut self.buffers[channel_index];

            // Handles both popping a single element during normal operation,
            // as well as multiple elements when resizing downwards.
            while buffer.len() >= config.max_depth as usize {
                buffer.pop_front();
            }

            buffer.push(*x);

            let lfo = make_wave(
                lfo_phase,
                config.lfo_type,
                config.freq,
                AntiAliasingMode::Off,
            );
            let delay_samples = mid_depth + (mod_depth as f32 * lfo * 0.5) as u32;
            *x = *buffer.get(delay_samples as usize).unwrap_or(&0.0);
            lfo_phase += config.freq * RECIP_SAMPLE_RATE;
        }
    }
}

impl Node<ProcessContext> for ModDelayNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        // Apply changes from the store.
        if let Some(EffectInstance {
            it: Effect::ModDelay(config),
            ..
        }) = &payload.store.try_select(&self.selector)
        {
            if *config != self.config {
                debug_assert!(config.min_depth <= config.max_depth);
                self.config = config.clone();
            }
        }

        let (left_out, right_out) = extract_outputs(output);
        let (left_in, right_in) = extract_inputs(inputs)[0];

        self.apply(left_out, left_in, /* channel_index= */ 0);
        self.apply(right_out, right_in, /* channel_index= */ 1);

        // Increase the LFO only by one buffer length (instead of N buffer lengths if there are N
        // channels).
        self.lfo_phase += Buffer::LEN as f32 * self.config.freq * RECIP_SAMPLE_RATE;
    }
}
