use std::f32::consts::PI;

use crate::WaveType;

const TAU: f32 = 2.0 * PI;
const HALF_PI: f32 = PI / 2.0;

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
    (x/2.0).tan().atan() / HALF_PI 
}

fn triangle_wave(x: f32) -> f32 {
    x.sin().asin() / HALF_PI 
}
