use dasp_graph::Buffer;

/// Pink noise filter using Paul Kellet's implementation (https://www.musicdsp.org/en/latest/Filters/76-pink-noise-filter.html)
/// -3 dB/octave (or -10 dB/decade) rolloff
/// Inspired by FunDSP (https://github.com/SamiPerttu/fundsp/blob/master/src/filter.rs)
#[derive(Clone, Default)]
pub struct PinkFilter {
    pub config: PinkFilterConfig,
}

#[derive(Clone)]
pub struct PinkFilterConfig {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub b3: f32,
    pub b4: f32,
    pub b5: f32,
    pub b6: f32,
}

impl Default for PinkFilterConfig {
    fn default() -> Self {
        Self {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
        }
    }
}

impl PinkFilter {
    pub fn new() -> Self {
        let config = PinkFilterConfig::default();
        Self { config }
    }

    pub fn reset(&mut self) {
        self.config = PinkFilterConfig::default();
    }

    pub fn apply(&mut self, buffer: &mut Buffer) {
        for xn in buffer.iter_mut() {
            self.config.b0 = 0.99886 * self.config.b0 + *xn * 0.0555179;
            self.config.b1 = 0.99332 * self.config.b1 + *xn * 0.0750759;
            self.config.b2 = 0.96900 * self.config.b2 + *xn * 0.1538520;
            self.config.b3 = 0.86650 * self.config.b3 + *xn * 0.3104856;
            self.config.b4 = 0.55000 * self.config.b4 + *xn * 0.5329522;
            self.config.b5 = -0.7616 * self.config.b5 - *xn * 0.0168980;
            let yn = self.config.b0
                + self.config.b1
                + self.config.b2
                + self.config.b3
                + self.config.b4
                + self.config.b5
                + self.config.b6
                + *xn * 0.5362;
            self.config.b6 = *xn * 0.115926;
            *xn = (yn * 0.11).clamp(-1.0, 1.0);
        }
    }
}
