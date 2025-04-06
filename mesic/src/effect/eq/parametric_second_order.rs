use super::filter::{SecondOrderFilter, SecondOrderFilterConfig};
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::PI;

pub fn parametric_non_constant_q(config: &EqConfig) -> SecondOrderFilter {
    let fs = SAMPLE_RATE as f32;

    let thetac: f32 = 2.0 * PI * config.fc / fs;

    let mu:f32 =  10.0_f32.powf(config.gain / 20.0);

    let zeta:f32 = 4.0/(1.0+mu);

    let beta:f32 = 0.5*(1.0 - zeta*(thetac/(2.0*config.q)).tan()) / 
        (1.0 + zeta*(thetac/(2.0*config.q)).tan());
    
    let gamma:f32 = (0.5+beta)*thetac.cos();

    let a0:f32 = 0.5 - beta;
    let a1:f32 = 0.0;
    let a2:f32 = -(0.5-beta);
    let b1:f32 = -2.0*gamma;
    let b2:f32 = 2.0*beta;

    // We may need to use this at some point, currently no handing for it :)
    // let c0:f32 = mu - 1.0;
    // let d0:f32 = 1;

    SecondOrderFilter::new(SecondOrderFilterConfig { a0, a1, a2, b1, b2 })
}   