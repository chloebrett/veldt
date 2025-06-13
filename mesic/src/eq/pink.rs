use crate::eq::filter::FilterState;
use dasp_graph::Buffer;
use ringbuffer::RingBuffer;

pub struct PinkFilter {
    pub config: PinkFilterConfig,
    pub state: FilterState,
}

pub struct PinkFilterConfig {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub b3: f32,
    pub b4: f32,
    pub b5: f32,
    pub b6: f32,
}

impl PinkFilterConfig {
    pub fn default() -> Self {
        Self { b0: 0.0, b1: 0.0, b2: 0.0, b3: 0.0, b4: 0.0, b5: 0.0, b6: 0.0 }
    }
}

impl PinkFilter {
    pub fn new() -> Self {
        let config = PinkFilterConfig::default();
        let state = FilterState::default();
        Self { config, state }
    }

    pub fn apply(&mut self, buffer: &mut Buffer) {
        let PinkFilterConfig { mut b0, mut b1, mut b2, mut b3, mut b4, mut b5, mut b6 } = self.config;

        for xn in buffer.iter_mut() {
            b0 = 0.99886 * b0 + *xn * 0.0555179;
            b1 = 0.99332 * b1 + *xn * 0.0750759;
            b2 = 0.96900 * b2 + *xn * 0.1538520;
            b3 = 0.86650 * b3 + *xn * 0.3104856;
            b4 = 0.55000 * b4 + *xn * 0.5329522;
            b5 = -0.7616 * b5 - *xn * 0.0168980;
            let yn = b0 + b1 + b2 + b3 + b4 + b5 + b6 + *xn * 0.5362;
            b6 = *xn * 0.115926;
            self.state.x_buffer.push(*xn);
            self.state.y_buffer.push(yn);
        }
    }
}