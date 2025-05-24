use super::ApplyFilter;
use dasp_graph::Buffer;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::f32::consts::TAU;
use crate::consts::SAMPLE_RATE;

pub struct Mix {
    // Note: wet/dry below is independent from wet/dry on the mixer.
    pub wet: f32,
    pub dry: f32,
}

/// A filter which looks at:
/// * x0 (current input)
/// * x1 (input one sample back)
/// * y1 (output one sample back)
///
/// In order to produce y0, the current output.
///
/// This can be considered a special case of the second order filter, with a2 = 0 and b2 = 0
/// always.
/// Because these are set to zero, we can simply use primitive ints instead of a ring buffer, and
/// save an allocation.
pub struct FirstOrderFilter {
    pub config: FirstOrderFilterConfig,
    pub mix: Option<Mix>, // if absent, assumed as wet = 1.0 and dry = 0.0.

    x_buffer: f32, // effectively a ring buffer with length 1.
    y_buffer: f32, // same as above.
}

/// Helps to prevent typos in param names.
pub struct FirstOrderFilterConfig {
    pub a0: f32, // coefficient of x0.
    pub a1: f32, // coefficient of x1.
    pub b1: f32, // coefficient of y1.
}

impl FirstOrderFilter {
    pub fn new_wet(config: FirstOrderFilterConfig) -> Self {
        Self::new_internal(config, None)
    }

    pub fn new(config: FirstOrderFilterConfig, mix: Mix) -> Self {
        Self::new_internal(config, Some(mix))
    }

    fn new_internal(config: FirstOrderFilterConfig, mix: Option<Mix>) -> Self {
        // The buffers are initialized to zero, which means the first sample processed will act as though
        // silence preceded it.
        let x_buffer = 0.0;
        let y_buffer = 0.0;

        Self {
            config,
            mix,
            x_buffer,
            y_buffer,
        }
    }
}

impl ApplyFilter for FirstOrderFilter {
    fn apply(&mut self, buffer: &mut Buffer) {
        let FirstOrderFilterConfig { a0, a1, b1 } = self.config;

        for xn in buffer.iter_mut() {
            let xn1 = self.x_buffer;
            let yn1 = self.y_buffer;

            let yn = a0 * *xn + a1 * xn1 - b1 * yn1;

            self.x_buffer = *xn;
            self.y_buffer = yn;

            apply_mix(xn, yn, *xn, &self.mix);
        }
    }

    fn update(&mut self, _freq: f32, _q: f32) {
        // No-op.
    }
}

/// A filter which looks at:
/// * x0 (current input)
/// * x1 (input one sample back)
/// * y1 (output one sample back)
/// * x2 (input two samples back)
/// * y2 (output two samples back)
///
/// In order to produce y0, the current output.
pub struct SecondOrderFilter {
    pub config: SecondOrderFilterConfig,
    pub mix: Option<Mix>,

    x_buffer: AllocRingBuffer<f32>,
    y_buffer: AllocRingBuffer<f32>,
}

/// Helps to prevent typos in param names.
pub struct SecondOrderFilterConfig {
    pub a0: f32, // coefficient of x0.
    pub a1: f32, // coefficient of x1.
    pub a2: f32, // coefficient of x2.
    pub b1: f32, // coefficient of y1.
    pub b2: f32, // coefficient of y2.
}

impl SecondOrderFilter {
    pub fn new_wet(config: SecondOrderFilterConfig) -> Self {
        Self::new_internal(config, None)
    }

    pub fn new(config: SecondOrderFilterConfig, mix: Mix) -> Self {
        Self::new_internal(config, Some(mix))
    }

    fn new_internal(config: SecondOrderFilterConfig, mix: Option<Mix>) -> Self {
        // Ring buffers store up to two samples back.
        let x_buffer = AllocRingBuffer::from([0.0; 2]);
        let y_buffer = AllocRingBuffer::from([0.0; 2]);

        Self {
            config,
            mix,
            x_buffer,
            y_buffer,
        }
    }
}

impl ApplyFilter for SecondOrderFilter {
    fn apply(&mut self, buffer: &mut Buffer) {
        let SecondOrderFilterConfig { a0, a1, a2, b1, b2 } = self.config;

        for xn in buffer.iter_mut() {
            // The oldest values should be removed from the ring buffer in each iteration.
            let xn2 = self.x_buffer.dequeue().expect("Expected value in x buffer");
            let yn2 = self.y_buffer.dequeue().expect("Expected value in y buffer");

            // The second-oldest values should be checked but not removed, because we will still
            // need them to process the next sample.
            let xn1 = self.x_buffer.front().expect("Expected value in x buffer");
            let yn1 = self.y_buffer.front().expect("Expected value in y buffer");

            let yn = a0 * *xn + a1 * xn1 + a2 * xn2 - b1 * yn1 - b2 * yn2;

            self.x_buffer.push(*xn);
            self.y_buffer.push(yn);

            apply_mix(xn, yn, *xn, &self.mix);
        }
    }

    fn update(&mut self, freq: f32, q: f32) {
        let fs = SAMPLE_RATE as f32;
        let theta = TAU * freq / fs;
        let d = 1.0 / q;
        let alpha = 0.5 * d * theta.sin();
        let beta = 0.5 * (1.0 - alpha) / (1.0 + alpha);
        let gamma = (0.5 + beta) * theta.cos();
        let a1 = 0.5 + beta - gamma;
        let a0 = 0.5 * a1;
        let a2 = a0;
        let b1 = -2.0 * gamma;
        let b2 = 2.0 * beta;

        self.config.a0 = a0;
        self.config.a1 = a1;
        self.config.a2 = a2;
        self.config.b1 = b1;
        self.config.b2 = b2;
    }
}

#[inline]
fn apply_mix(o: &mut f32, w: f32, d: f32, mix: &Option<Mix>) {
    if let Some(Mix { wet, dry }) = mix {
        // Apply the filter using the wet/dry values.
        *o = wet * w + dry * d;
    } else {
        // Slight optimization.
        // Replace the input value with the corresponding output value.
        *o = w;
    }
}
