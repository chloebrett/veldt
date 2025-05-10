use crate::consts::{DOUBLE_RECIP_PI, NYQUIST, RECIP_PI};
use shared::consts::SEMITONE_FREQ;
use shared::model::{AntiAliasingMode, WaveType};
use shared::types::Freq;
use std::f32::consts::TAU;

pub fn detune_multiplier(cents: f32) -> Freq {
    if cents == 0.0 {
        return 1.0;
    }

    let interval = cents / 100.0;

    SEMITONE_FREQ.powf(interval)
}

/// Constructs the given wave at the given phase. x is between 0 and TAU (or will be modulo'd to be
/// between these numbers).
/// wave_freq is sent to determine cutoffs for additive anti-aliasing.
pub fn make_wave(
    x: f32,
    wave_type: WaveType,
    wave_freq: Freq,
    anti_aliasing_mode: AntiAliasingMode,
) -> f32 {
    let x = (x % 1.0) * TAU;

    match wave_type {
        WaveType::Sine => x.sin(),
        WaveType::Square => match anti_aliasing_mode {
            AntiAliasingMode::Off => square_wave(x),
            AntiAliasingMode::Additive => square_wave_additive(x, wave_freq),
            AntiAliasingMode::Oversample => todo!(),
        },
        WaveType::Saw => match anti_aliasing_mode {
            AntiAliasingMode::Off => saw_wave(x),
            AntiAliasingMode::Additive => saw_wave_additive(x, wave_freq),
            AntiAliasingMode::Oversample => todo!(),
        },
        WaveType::Triangle => triangle_wave(x),
    }
}

fn square_wave(x: f32) -> f32 {
    x.sin().signum()
}

/// Builds an anti-aliased square wave by summing up sine waves according to the square wave
/// formula.
fn square_wave_additive(x: f32, wave_freq: Freq) -> f32 {
    // Maximum number of iterations to try. Reducing this produces a sort of low pass effect,
    // and also makes the wave faster to compute.
    let max_k = 1000;

    // The output value.
    let mut y = 0.0;

    for i in 1..max_k {
        // Square wave formula: https://en.wikipedia.org/wiki/Square_wave_(waveform)
        let w = (2 * i - 1) as f32;

        // Stop adding harmonics once they exceed the Nyquist limit (half the sample rate).
        // This prevents the signal from aliasing.
        let harmonic = w * wave_freq;
        if harmonic > NYQUIST as Freq {
            break;
        }

        let s = (w * x).sin();
        y += s / w
    }
    y * 4.0 * RECIP_PI
}

/// Builds an anti-aliased saw wave by summing up sine waves according to the saw wave
/// formula.
fn saw_wave_additive(x: f32, wave_freq: Freq) -> f32 {
    // Maximum number of iterations to try. Reducing this produces a sort of low pass effect,
    // and also makes the wave faster to compute.
    let max_k = 1000;

    // The output value.
    let mut y = 0.0;

    for i in 1..max_k {
        let w = i as f32;

        // Stop adding harmonics once they exceed the Nyquist limit (half the sample rate).
        // This prevents the signal from aliasing.
        let harmonic = w * wave_freq;
        if harmonic > NYQUIST as Freq {
            break;
        }

        // Saw wave formula: https://en.wikipedia.org/wiki/Sawtooth_wave
        let s = (w * x).sin();
        let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        y += sign * s / w
    }
    y * -2.0 * RECIP_PI
}

fn saw_wave(x: f32) -> f32 {
    (x * 0.5).tan().atan() * DOUBLE_RECIP_PI
}

fn triangle_wave(x: f32) -> f32 {
    x.sin().asin() * DOUBLE_RECIP_PI
}
