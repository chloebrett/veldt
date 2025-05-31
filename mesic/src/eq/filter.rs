use super::ApplyFilter;
use dasp_graph::Buffer;
use ringbuffer::{AllocRingBuffer, RingBuffer};

pub struct Mix {
    // Note: wet/dry below is independent from wet/dry on the mixer.
    pub wet: f32,
    pub dry: f32,
}

/// A filter which looks at:
/// * x0 (current input)
/// * x1 (input one sample back)
/// * y1 (output one sample back)
/// * x2 (input two samples back)
/// * y2 (output two samples back)
///
/// In order to produce y0, the current output.
/// For first order filters, a2 and b2 are 0.
pub struct Filter {
    pub config: FilterConfig,
    pub mix: Option<Mix>, // if absent, assumed as wet = 1.0 and dry = 0.0.

    x_buffer: AllocRingBuffer<f32>,
    y_buffer: AllocRingBuffer<f32>,
}

/// Helps to prevent typos in param names.
pub struct FilterConfig {
    pub a0: f32, // coefficient of x0.
    pub a1: f32, // coefficient of x1.
    pub a2: f32, // coefficient of x2.
    pub b1: f32, // coefficient of y1.
    pub b2: f32, // coefficient of y2.
}

impl Filter {
    pub fn new_wet(config: FilterConfig) -> Self {
        Self::new_internal(config, None)
    }

    pub fn new(config: FilterConfig, mix: Mix) -> Self {
        Self::new_internal(config, Some(mix))
    }

    fn new_internal(config: FilterConfig, mix: Option<Mix>) -> Self {
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

impl ApplyFilter for Filter {
    fn apply(&mut self, buffer: &mut Buffer) {
        let FilterConfig { a0, a1, a2, b1, b2 } = self.config;

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
