use crate::consts::SAMPLE_RATE;
use shared::types::{Freq, KnobPosition};
use std::f32::consts::TAU;

pub fn apply_simple_resonator(dry_signal: Vec<f32>, fc: Freq, q_value: KnobPosition) -> Vec<f32> {
    let fs = SAMPLE_RATE as f32;
    let theta = TAU * fc / fs;
    let bandwidth = fc / q_value;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p259
    let b2: f32 = (-TAU * bandwidth / fs).exp();
    let b1: f32 = (-4.0 * b2) / (1.0 + b2) * theta.cos();
    let a0: f32 = (1.0 - b2) * (1.0 - ((b1 * b1) / (4.0 * b2))).sqrt();

    // Pre-fill with two zero values as filling the vec depends on its previous values.
    let mut output: Vec<f32> = vec![0.0, 0.0];
    for i in 2..dry_signal.len() {
        let xn = dry_signal[i];
        let yn1 = output[i - 1];
        let yn2 = output[i - 2];
        let yn = a0 * xn - b1 * yn1 - b2 * yn2;

        output.push(yn);
    }

    output.drain(0..2);
    output
}
