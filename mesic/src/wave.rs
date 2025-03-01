use std::f32::consts::PI;

use crate::WaveType;

const TAU: f32 = 2.0 * PI;
const HALF_PI: f32 = 0.5 * PI;
const INV_HALF_PI: f32 = HALF_PI.recip();

pub fn make_wave(x: f32, wave_type: WaveType) -> f32 {
    let x = x % TAU;

    match wave_type {
        WaveType::Sine => x.sin(),
        WaveType::Square => square_wave(x),
        WaveType::Saw => saw_wave(x),
        WaveType::Triangle => triangle_wave(x),
    }
}

fn square_wave(x: f32) -> f32 {
    x.sin().signum()
}

fn saw_wave(x: f32) -> f32 {
    (x * 0.5).tan().atan() * INV_HALF_PI
}

fn triangle_wave(x: f32) -> f32 {
    x.sin().asin() * INV_HALF_PI
}
