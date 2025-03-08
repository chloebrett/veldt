use super::filter::first_degree_filter;
use crate::consts::SAMPLE_RATE;
use shared::types::Freq;
use std::f32::consts::TAU;

pub fn apply_low_pass_basic_first_order(dry_signal: Vec<f32>, fc: Freq) -> Vec<f32> {
    let fs = SAMPLE_RATE as f32;
    let theta = TAU * fc / fs;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p271
    let gamma: f32 = theta.cos() / (1.0 + theta.sin());
    let a0: f32 = 0.5 * (1.0 - gamma);
    let a1: f32 = a0;
    let b1: f32 = -gamma;

    first_degree_filter(dry_signal, a0, a1, b1)
}

// TODO: since they are so similar, combine the HPF and LPF functions.
pub fn apply_high_pass_basic_first_order(dry_signal: Vec<f32>, fc: Freq) -> Vec<f32> {
    let fs = SAMPLE_RATE as f32;
    let theta = TAU * fc / fs;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p271
    let gamma: f32 = theta.cos() / (1.0 + theta.sin());
    let a0: f32 = 0.5 * (1.0 + gamma);
    let a1: f32 = -a0;
    let b1: f32 = -gamma;

    first_degree_filter(dry_signal, a0, a1, b1)
}
