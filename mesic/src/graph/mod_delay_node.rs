use crate::consts::SAMPLE_RATE_RECIP;
use crate::wave::make_wave;
use dasp_graph::{Buffer, Input, Node};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use shared::model::{AntiAliasingMode, ModDelayConfig};

/// Modulated delay - i.e. vibrato, flanger, phaser, chorus.
pub struct ModDelayNode {
    config: ModDelayConfig,
    lfo_phase: f32, // current phase of the LFO. Ranges from 0 to 1 then loops back to 0.
    buffer: AllocRingBuffer<f32>,
    mod_depth: u32,
    mid_depth: u32,
    period: f32, // reciprocal of frequency
}

impl ModDelayNode {
    pub fn new(config: ModDelayConfig) -> ModDelayNode {
        debug_assert!(config.min_depth <= config.max_depth);
        let mod_depth = config.max_depth - config.min_depth;
        let mid_depth = (config.max_depth + config.min_depth) / 2;
        ModDelayNode {
            config: config.clone(),
            lfo_phase: 0.0,
            mod_depth,
            mid_depth,
            period: 1.0 / config.freq,
            // Using AllocRingBuffer as it has a clearer API than dasp_ring_buffer.
            buffer: AllocRingBuffer::from(vec![0.0; config.max_depth as usize]),
        }
    }
}

impl Node for ModDelayNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        for (out_buf, in_buf) in output
            .iter_mut()
            .zip(inputs.first().expect("Expected one input").buffers())
        {
            out_buf.copy_from_slice(in_buf);

            for x in out_buf.iter_mut() {
                self.buffer.push(*x);

                let lfo = make_wave(
                    self.lfo_phase,
                    self.config.lfo_type,
                    self.config.freq,
                    AntiAliasingMode::Off,
                );
                let delay_samples = self.mid_depth + (self.mod_depth as f32 * lfo / 2.0) as u32;
                *x = *self.buffer.get(delay_samples as usize).unwrap();
                self.lfo_phase += self.period * SAMPLE_RATE_RECIP;
            }
        }
    }
}
