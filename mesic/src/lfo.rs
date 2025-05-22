use crate::consts::SAMPLE_RATE;
use shared::model::{LfoConfig, AntiAliasingMode};
use crate::wave::make_wave;
use std::f32::consts::TAU;

#[derive(Clone, Debug, PartialEq)]
pub struct Lfo {
    pub config: LfoConfig,
    pub sample_index: usize,
}

impl Lfo {
    pub fn new(config: LfoConfig) -> Self {
        Self {
            config,
            sample_index: 0,
        }
    }

    pub fn next(&mut self) -> f32 {
        let frequency = self.config.frequency;
        let wave = self.config.wave;

        let sample_rate = SAMPLE_RATE as f32;
        let period = sample_rate / frequency;
        let phase = (self.sample_index as f32 / period);

        self.sample_index += 1;
        self.sample_index = self.sample_index % (sample_rate as usize);

        make_wave(
            phase,
            wave,
            frequency,
            AntiAliasingMode::Off, // Surely the freq is low enough to not need Anti Aliasing
        )
    }
}