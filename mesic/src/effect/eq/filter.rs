use super::ApplyFilter;
use ringbuffer::{AllocRingBuffer, RingBuffer};

pub struct FirstOrderFilter {
    pub a0: f32,
    pub a1: f32,
    pub b1: f32,

    x_buffer: AllocRingBuffer<f32>,
    y_buffer: AllocRingBuffer<f32>,
}

/// Helps to prevent typos in param names.
pub struct FirstOrderFilterConfig {
    pub a0: f32,
    pub a1: f32,
    pub b1: f32,
}

impl FirstOrderFilter {
    pub fn new(config: FirstOrderFilterConfig) -> Self {
        let x_buffer = AllocRingBuffer::from([0.0; 2]);
        let y_buffer = AllocRingBuffer::from([0.0; 2]);

        FirstOrderFilter {
            a0: config.a0,
            a1: config.a1,
            b1: config.b1,
            x_buffer,
            y_buffer,
        }
    }
}

impl ApplyFilter for FirstOrderFilter {
    fn apply(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![];

        for xn in input.iter() {
            let xn1 = self.x_buffer.back().expect("Expected value in x buffer");
            let yn1 = self.y_buffer.back().expect("Expected value in y buffer");

            let yn = self.a0 * xn + self.a1 * xn1 - self.b1 * yn1;

            self.x_buffer.push(*xn);
            self.y_buffer.push(yn);
            output.push(yn);
        }

        output
    }
}

pub struct SecondOrderFilter {
    pub a0: f32,
    pub a1: f32,
    pub a2: f32,
    pub b1: f32,
    pub b2: f32,
}

impl ApplyFilter for SecondOrderFilter {
    fn apply(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![0.0, 0.0];
        for i in 2..input.len() {
            let xn = input[i];
            let xn1 = input[i - 1];
            let xn2 = input[i - 2];
            let yn1 = output[i - 1];
            let yn2 = output[i - 2];
            let yn = self.a0 * xn + self.a1 * xn1 + self.a2 * xn2 - self.b1 * yn1 - self.b2 * yn2;

            output.push(yn);
        }

        output.drain(0..2);
        output
    }
}

/// A second order filter which doesn't rely on previous input values.
/// Special case of SecondOrderFilter.
pub struct SecondOrderFeedbackFilter {
    pub a0: f32,
    pub b1: f32,
    pub b2: f32,
}

impl ApplyFilter for SecondOrderFeedbackFilter {
    fn apply(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![0.0, 0.0];
        for i in 2..input.len() {
            let xn = input[i];
            let yn1 = output[i - 1];
            let yn2 = output[i - 2];
            let yn = self.a0 * xn - self.b1 * yn1 - self.b2 * yn2;

            output.push(yn);
        }

        output.drain(0..2);
        output
    }
}
