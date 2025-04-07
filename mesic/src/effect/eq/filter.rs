use super::ApplyFilter;
use dasp_graph::Buffer;
use ringbuffer::{AllocRingBuffer, RingBuffer};

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
    a0: f32, // coefficient of x0.
    a1: f32, // coefficient of x1.
    b1: f32, // coefficient of y1.

    x_buffer: f32, // effectively a ring buffer with length 1.
    y_buffer: f32, // same as above.
}

/// Helps to prevent typos in param names.
pub struct FirstOrderFilterConfig {
    pub a0: f32,
    pub a1: f32,
    pub b1: f32,
}

impl FirstOrderFilter {
    pub fn new(config: FirstOrderFilterConfig) -> Self {
        // The buffers are initialized to zero, which means the first sample processed will act as though
        // silence preceded it.
        let x_buffer = 0.0;
        let y_buffer = 0.0;

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
    fn apply(&mut self, buffer: &mut Buffer) {
        for xn in buffer.iter_mut() {
            let xn1 = self.x_buffer;
            let yn1 = self.y_buffer;

            let yn = self.a0 * *xn + self.a1 * xn1 - self.b1 * yn1;

            self.x_buffer = *xn;
            self.y_buffer = yn;

            // Replace the input value with the corresponding output value.
            *xn = yn;
        }
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
    a0: f32, // coefficient of x0.
    a1: f32, // coefficient of x1.
    a2: f32, // coefficient of x2.
    b1: f32, // coefficient of y1.
    b2: f32, // coefficient of y2.

    // TODO: consolidate the ring buffer types. We use dasp_ring_buffer for the delay nodes.
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
        // Ring buffers store up to two samples back.
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
    fn apply(&mut self, buffer: &mut Buffer) {
        for xn in buffer.iter_mut() {
            // The oldest values should be removed from the ring buffer in each iteration.
            let xn2 = self.x_buffer.dequeue().expect("Expected value in x buffer");
            let yn2 = self.y_buffer.dequeue().expect("Expected value in y buffer");

            // The second-oldest values should be checked but not removed, because we will still
            // need them to process the next sample.
            let xn1 = self.x_buffer.front().expect("Expected value in x buffer");
            let yn1 = self.y_buffer.front().expect("Expected value in y buffer");

            let yn = self.a0 * *xn + self.a1 * xn1 + self.a2 * xn2 - self.b1 * yn1 - self.b2 * yn2;

            self.x_buffer.push(*xn);
            self.y_buffer.push(yn);

            // Replace the input value with the corresponding output value.
            *xn = yn;
        }
    }
}

/// A filter which looks at:
/// * x0 (current input)
/// * y1 (output one sample back)
/// * y2 (output two samples back)
///
/// In order to produce y0, the current output.
///
/// This is a special case of the second order filter which doesn't rely on previous input values.
pub struct SecondOrderFeedbackFilter {
    a0: f32, // coefficient of x0.
    b1: f32, // coefficient of y1.
    b2: f32, // coefficient of y2.

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
    fn apply(&mut self, buffer: &mut Buffer) {
        for xn in buffer.iter_mut() {
            let yn2 = self.y_buffer.dequeue().expect("Expected value in y buffer");
            let yn1 = self.y_buffer.front().expect("Expected value in y buffer");

            let yn = self.a0 * *xn - self.b1 * yn1 - self.b2 * yn2;

            self.y_buffer.push(yn);

            // Replace the input value with the corresponding output value.
            *xn = yn;
        }
    }
}
