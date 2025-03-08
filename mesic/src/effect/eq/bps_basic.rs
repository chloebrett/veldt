use super::filter::second_order;
use std::f32::consts::PI;
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;

pub fn band_pass_basic(config: EqConfig, input: Vec<f32>) -> Vec<f32> {
    let fc = config.fc;
    let q = config.q;
    let fs = SAMPLE_RATE as f32;
    let k = (PI * fc / fs).tan();
    let k2 = k * k;
    let k2_q = k2 * q;
    let delta = k2_q + k + q;
    let delta_recip = 1.0 / delta;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p273
    let a0: f32 = k * delta_recip;
    let a1: f32 = 0.0;
    let a2: f32 = -a0;
    let b1: f32 = 2.0 * (k2_q - q) * delta_recip;
    let b2: f32 = (k2_q - k + q) * delta_recip;

    second_order(input, a0, a1, a2, b1, b2)
}

pub fn band_stop_basic(config: EqConfig, input: Vec<f32>) -> Vec<f32> {
    let fc = config.fc;
    let q = config.q;
    let fs = SAMPLE_RATE as f32;
    let k = (PI * fc / fs).tan();
    let k2 = k * k;
    let k2_q = k2 * q;
    let delta = k2_q + k + q;
    let delta_recip = 1.0 / delta;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p273
    let a0: f32 = (k2_q + q) * delta_recip;
    let a1: f32 = 2.0 * (k2_q - q) * delta_recip;
    let a2: f32 = a0;
    let b1: f32 = a1;
    let b2: f32 = (k2_q - k + q) * delta_recip;

    second_order(input, a0, a1, a2, b1, b2)
}
