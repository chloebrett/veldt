use super::filter::{Filter, FilterConfig};
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::TAU;

pub fn first_order_all_pole(config: &EqConfig) -> Filter {
    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p282
    let theta = TAU * config.fc / SAMPLE_RATE as f32;
    let gamma = 2.0 - theta.cos();
    let b1 = (gamma * gamma - 1.0).sqrt() - gamma;
    let a0 = 1.0 + b1;
    let a1 = 0.0;

    Filter::new_wet(FilterConfig::default().a0(a0).a1(a1).b1(b1))
}
