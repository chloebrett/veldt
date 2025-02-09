use std::f32::consts::PI;

use crate::WaveType;

const TAU: f32 = 2.0 * PI;
const HALF_PI: f32 = 0.5 * PI;

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
    if x > PI { -1.0 } else { 1.0 }
}

fn saw_wave(x: f32) -> f32 {
    if x > PI { x / PI - 2.0 } else { x / PI }
}

fn triangle_wave(x: f32) -> f32 {
    if x < HALF_PI {
        x / HALF_PI
    } else if x < 3.0 * HALF_PI {
        2.0 * (1.0 - x / PI)
    } else {
        2.0 * (x / PI) - 4.0
    }
}
