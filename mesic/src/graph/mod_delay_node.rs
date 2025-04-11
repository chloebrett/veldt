use super::{extract_inputs, extract_outputs};
use crate::consts::SAMPLE_RATE_RECIP;
use crate::wave::make_wave;
use dasp_graph::{Buffer, Input, Node};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use shared::model::{AntiAliasingMode, ModDelayConfig};

/// Modulated delay - i.e. vibrato, flanger, phaser, chorus.
pub struct ModDelayNode {
    config: ModDelayConfig,
    lfo_phase: f32, // current phase of the LFO. Ranges from 0 to 1 then loops back to 0.
    buffers: [AllocRingBuffer<f32>; 2],
    mod_depth: u32,
    mid_depth: u32,
    period: f32, // reciprocal of frequency
}

impl ModDelayNode {
    pub fn new(config: ModDelayConfig) -> ModDelayNode {
        debug_assert!(config.min_depth <= config.max_depth);
        let mod_depth = config.max_depth - config.min_depth;
        let mid_depth = (config.max_depth + config.min_depth) / 2;
        let buffer = AllocRingBuffer::from(vec![0.0; config.max_depth as usize]);
        ModDelayNode {
            config: config.clone(),
            lfo_phase: 0.0,
            mod_depth,
            mid_depth,
            period: 1.0 / config.freq,
            // Using AllocRingBuffer as it has a clearer API than dasp_ring_buffer.
            buffers: [buffer.clone(), buffer.clone()],
        }
    }

    fn apply(&mut self, out_buf: &mut Buffer, in_buf: &Buffer, channel_index: usize) {
        out_buf.copy_from_slice(in_buf);
        let mut lfo_phase = self.lfo_phase;

        for x in out_buf.iter_mut() {
            let buffer = &mut self.buffers[channel_index];
            buffer.push(*x);

            let lfo = make_wave(
                lfo_phase,
                self.config.lfo_type,
                self.config.freq,
                AntiAliasingMode::Off,
            );
            let delay_samples = self.mid_depth + (self.mod_depth as f32 * lfo * 0.5) as u32;
            *x = *buffer.get(delay_samples as usize).unwrap();
            lfo_phase += self.period * SAMPLE_RATE_RECIP;
        }
    }
}

impl Node for ModDelayNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        let (left_out, right_out) = extract_outputs(output);
        let (left_in, right_in) = extract_inputs(inputs)[0];

        self.apply(left_out, left_in, /* channel_index= */ 0);
        self.apply(right_out, right_in, /* channel_index= */ 1);

        // Increase the LFO only by one buffer length (instead of N buffer lengths if there are N
        // channels);
        self.lfo_phase += Buffer::LEN as f32 * self.period * SAMPLE_RATE_RECIP;
    }
}
