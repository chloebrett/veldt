use super::filter::second_degree_filter;
use crate::consts::SAMPLE_RATE;
use shared::types::{Freq, KnobPosition};
use std::f32::consts::TAU;

pub fn apply_smith_angell_resonator(
    dry_signal: Vec<f32>,
    fc: Freq,
    q_value: KnobPosition,
) -> Vec<f32> {
    let fs = SAMPLE_RATE as f32;
    let theta = TAU * fc / fs;
    let bandwidth = fc / q_value;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p260
    let b2 = (-TAU * bandwidth / fs).exp();
    let b1 = (-4.0 * b2) / (1.0 + b2) * theta.cos();
    let a0 = 1.0 - b2.sqrt();
    let a1 = 0.0;
    let a2 = -a0;

    second_degree_filter(dry_signal, a0, a1, a2, b1, b2)
}
