use super::filter::second_degree_feedback_filter;
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

    second_degree_feedback_filter(dry_signal, a0, b1, b2)
}
