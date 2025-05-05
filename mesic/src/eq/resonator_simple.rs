use super::filter::{SecondOrderFilter, SecondOrderFilterConfig};
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::TAU;

pub fn resonator_simple(config: &EqConfig) -> SecondOrderFilter {
    let fs = SAMPLE_RATE as f32;
    let theta = TAU * config.fc / fs;
    let bandwidth = config.fc / config.q;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p259
    let b2: f32 = (-TAU * bandwidth / fs).exp();
    let b1: f32 = (-4.0 * b2) / (1.0 + b2) * theta.cos();
    let a0: f32 = (1.0 - b2) * (1.0 - ((b1 * b1) / (4.0 * b2))).sqrt();

    SecondOrderFilter::new_wet(SecondOrderFilterConfig {
        a0,
        a1: 0.0,
        b1,
        a2: 0.0,
        b2,
    })
}
