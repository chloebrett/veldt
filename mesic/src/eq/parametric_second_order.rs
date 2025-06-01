use super::filter::{Filter, FilterConfig, Mix};
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::TAU;

pub fn parametric_non_constant_q(config: &EqConfig) -> Filter {
    let fs = SAMPLE_RATE as f32;
    let thetac = TAU * config.fc / fs;

    let mu = 10.0_f32.powf(config.gain / 20.0);
    let zeta = 4.0 / (1.0 + mu);
    let tan_constant = (thetac / (2.0 * config.q)).tan();
    let beta = 0.5 * (1.0 - zeta * tan_constant) / (1.0 + zeta * tan_constant);
    let gamma = (0.5 + beta) * thetac.cos();

    let a0 = 0.5 - beta;
    let a1 = 0.0;
    let a2 = -(0.5 - beta);
    let b1 = -2.0 * gamma;
    let b2 = 2.0 * beta;

    let wet: f32 = mu - 1.0;
    let dry: f32 = 1.0;

    Filter::new(FilterConfig { a0, a1, a2, b1, b2 }, Mix { wet, dry })
}
