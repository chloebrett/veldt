use super::ApplyFilter;
use ringbuffer::{AllocRingBuffer, RingBuffer};

pub struct FirstOrderFilter {
    a0: f32,
    a1: f32,
    b1: f32,

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
        let x_buffer = AllocRingBuffer::from([0.0; 1]);
        let y_buffer = AllocRingBuffer::from([0.0; 1]);

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
            let xn1 = self.x_buffer.dequeue().expect("Expected value in x buffer");
            let yn1 = self.y_buffer.dequeue().expect("Expected value in y buffer");

            let yn = self.a0 * xn + self.a1 * xn1 - self.b1 * yn1;

            self.x_buffer.push(*xn);
            self.y_buffer.push(yn);
            output.push(yn);
        }

        output
    }
}

pub struct SecondOrderFilter {
    a0: f32,
    a1: f32,
    a2: f32,
    b1: f32,
    b2: f32,

    x_buffer: AllocRingBuffer<f32>,
    y_buffer: AllocRingBuffer<f32>,
}

/// Helps to prevent typos in param names.
pub struct SecondOrderFilterConfig {
    pub a0: f32,
    pub a1: f32,
    pub a2: f32,
    pub b1: f32,
    pub b2: f32,
}

impl SecondOrderFilter {
    pub fn new(config: SecondOrderFilterConfig) -> Self {
        let x_buffer = AllocRingBuffer::from([0.0; 2]);
        let y_buffer = AllocRingBuffer::from([0.0; 2]);

        SecondOrderFilter {
            a0: config.a0,
            a1: config.a1,
            a2: config.a2,
            b1: config.b1,
            b2: config.b2,
            x_buffer,
            y_buffer,
        }
    }
}

impl ApplyFilter for SecondOrderFilter {
    fn apply(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![];

        for xn in input.iter() {
            let xn2 = self.x_buffer.dequeue().expect("Expected value in x buffer");
            let yn2 = self.y_buffer.dequeue().expect("Expected value in y buffer");
            let xn1 = self.x_buffer.front().expect("Expected value in x buffer");
            let yn1 = self.y_buffer.front().expect("Expected value in y buffer");

            let yn = self.a0 * xn + self.a1 * xn1 + self.a2 * xn2 - self.b1 * yn1 - self.b2 * yn2;

            self.x_buffer.push(*xn);
            self.y_buffer.push(yn);
            output.push(yn);
        }

        output
    }
}

/// A second order filter which doesn't rely on previous input values.
/// Special case of SecondOrderFilter.
pub struct SecondOrderFeedbackFilter {
    a0: f32,
    b1: f32,
    b2: f32,

    y_buffer: AllocRingBuffer<f32>,
    // Does not have x_buffer.
}

/// Helps to prevent typos in param names.
pub struct SecondOrderFeedbackFilterConfig {
    pub a0: f32,
    pub b1: f32,
    pub b2: f32,
}

impl SecondOrderFeedbackFilter {
    pub fn new(config: SecondOrderFeedbackFilterConfig) -> Self {
        let y_buffer = AllocRingBuffer::from([0.0; 2]);

        SecondOrderFeedbackFilter {
            a0: config.a0,
            b1: config.b1,
            b2: config.b2,
            y_buffer,
        }
    }
}

impl ApplyFilter for SecondOrderFeedbackFilter {
    fn apply(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![];

        for xn in input.iter() {
            let yn2 = self.y_buffer.dequeue().expect("Expected value in y buffer");
            let yn1 = self.y_buffer.front().expect("Expected value in y buffer");

            let yn = self.a0 * xn - self.b1 * yn1 - self.b2 * yn2;

            self.y_buffer.push(yn);
            output.push(yn);
        }

        output
    }
}
