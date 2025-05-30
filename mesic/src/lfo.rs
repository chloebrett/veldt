use crate::consts::SAMPLE_RATE;
use crate::wave::make_wave;
use shared::model::{AntiAliasingMode, LfoConfig};

#[derive(Clone, Debug)]
pub struct LfoGenerator {
    pub config: LfoConfig,
    pub sample_index: usize,
    pub current_value: f32,
}

impl LfoGenerator {
    pub fn new(config: LfoConfig) -> Self {
        Self {
            config,
            sample_index: 0,
            current_value: 0.0,
        }
    }

    pub fn next(&mut self) -> f32 {
        let frequency = self.config.frequency;
        let wave = self.config.wave;

        let period = SAMPLE_RATE as f32 / frequency;
        let phase = self.sample_index as f32 / period;

        self.sample_index += 1;
        self.sample_index = self.sample_index % (SAMPLE_RATE as usize);

        self.current_value = make_wave(
            phase,
            wave,
            frequency,
            AntiAliasingMode::Off, // Surely the freq is low enough to not need Anti Aliasing
        );
        self.current_value
    }

    pub fn set_lfo(&mut self, config: LfoConfig) {
        self.config = config;
    }
}
