use super::filter::{Filter, FilterConfig, Mix};
use super::low_high::LowHigh;
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::PI;

pub fn shelf_first_order(config: &EqConfig, low_high: LowHigh) -> Filter {
    let fs = SAMPLE_RATE as f32;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p278 (Shelving Filters)
    let thetac = PI * config.fc / fs;
    let mu = 10.0_f32.powf(config.gain * 0.05);
    let beta = match low_high {
        LowHigh::Low => 4.0 / (1.0 + mu),
        LowHigh::High => (1.0 + mu) * 0.25,
    };
    let delta = beta * (thetac * 0.5).tan();
    let gamma = (1.0 - delta) / (1.0 + delta);

    let low_frac = 0.5 * (1.0 - gamma);
    let high_frac = 0.5 * (1.0 + gamma);

    let a0 = match low_high {
        LowHigh::Low => low_frac,
        LowHigh::High => high_frac,
    };
    let a1 = match low_high {
        LowHigh::Low => low_frac,
        LowHigh::High => -high_frac,
    };
    let b1 = -gamma;

    // TODO: add support for wet/dry to first-order filters
    let wet: f32 = mu - 1.0;
    let dry: f32 = 1.0;

    Filter::new(
        FilterConfig::default().a0(a0).a1(a1).b1(b1),
        Mix { wet, dry },
    )
}
