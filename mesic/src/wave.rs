use crate::consts::{NYQUIST, SAMPLE_RATE, SECONDS_PER_MINUTE};
use dasp_graph::Buffer;
use ordered_float::OrderedFloat;
use shared::consts::SEMITONE_FREQ;
use shared::model::{AntiAliasingMode, WaveType};
use shared::types::{Beats, Freq};
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

const HALF_PI: f32 = 0.5 * PI;
const RECIP_HALF_PI: f32 = HALF_PI.recip(); // 2 / PI, not 1 / TAU.
const RECIP_PI: f32 = PI.recip();

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct WaveKey {
    pub kind: WaveType,
    pub aa: AntiAliasingMode,
    pub freq: OrderedFloat<Freq>,
}

#[derive(Debug)]
pub struct Wave {
    // One full cycle, however long that may be at SAMPLE_RATE.
    pub buffer: Vec<f32>,
}

#[derive(Default, Debug)]
pub struct WaveCache {
    cache: HashMap<WaveKey, Wave>,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

// Multiplier on the lookup table size.
// This is necessary when the frequency isn't a clean divisor of the sample rate,
// which is most of the time.
// A value of 10, coupled with lerping, gets rid of most of the audible harmonics.
const FIDELITY: f32 = 10.0;

impl WaveCache {
    /// Returns the amplitude of wave with the given configuration (key) at the given phase.
    /// Phase is between 0.0..1.0.
    /// Note: "key" refers to a HashMap key, not a musical key.
    pub fn get(&mut self, key: &WaveKey, phase: f32) -> f32 {
        debug_assert!((0.0..1.0).contains(&phase));
        let total_samples = FIDELITY * SAMPLE_RATE as f32 / *key.freq;
        let total_samples_len = total_samples.round() as usize;
        let phase_samples = total_samples_len as f32 * phase;

        if let Some(wave) = self.cache.get(key) {
            debug_assert!(wave.buffer.len() == total_samples_len);

            let a = wave.buffer[phase_samples.floor() as usize];
            let b = wave.buffer[(phase_samples.floor() as usize) + 1];
            let t = phase_samples % 1.0;

            return lerp(a, b, t);
        }

        let buffer: Vec<_> = (0..=total_samples_len)
            .map(|x| make_wave(x as f32 / total_samples, key.kind, *key.freq, key.aa))
            .collect();

        debug_assert!(buffer.len() == total_samples_len);

        let a = buffer[phase_samples.floor() as usize];
        let b = buffer[(phase_samples.floor() as usize) + 1];
        let t = phase_samples % 1.0;
        let current_value = lerp(a, b, t);

        self.cache.insert(key.clone(), Wave { buffer });

        current_value
    }
}

pub struct Unison {
    pub detune_cents: f32,
    pub osc_count: usize,
}

pub fn beats_to_samples(beats: Beats, bpm: Beats) -> u32 {
    let seconds = beats / bpm * SECONDS_PER_MINUTE;
    (SAMPLE_RATE as f32 * seconds) as u32
}

pub fn detune_multiplier(cents: f32) -> Freq {
    if cents == 0.0 {
        return 1.0;
    }

    let interval = cents / 100.0;

    SEMITONE_FREQ.powf(interval)
}

/// Returns a vec range with `count` evenly spaced values from `low` to `high`.
pub fn linspace(low: f32, high: f32, count: u32) -> Vec<f32> {
    if count == 0 {
        panic!("Tried to linspace with count == 0");
    }
    if count == 1 {
        let mid = (high + low) / 2.0;
        return vec![mid];
    }

    (0..count)
        .map(|x| (x as f32) * (high - low) / ((count - 1) as f32) + low)
        .collect()
}

// TODO: make multi_sum private
/// Sums the input buffers into a single buffer.
pub fn multi_sum(inputs: &[Buffer]) -> Buffer {
    let mut output = Buffer::SILENT;

    for input in inputs {
        dasp_slice::add_in_place(&mut output, input);
    }

    output
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
    (x * 0.5).tan().atan() * RECIP_HALF_PI
}

fn triangle_wave(x: f32) -> f32 {
    x.sin().asin() * RECIP_HALF_PI
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLOAT_THRES: f32 = 1e-6;

    #[test]
    fn linspace_1() {
        let output = linspace(50.0, 100.0, 1);

        assert_float_vec_almost_eq(output, vec![75.0]);
    }

    #[test]
    fn linspace_2() {
        let output = linspace(10.0, 19.0, 2);

        assert_float_vec_almost_eq(output, vec![10.0, 19.0]);
    }

    #[test]
    fn linspace_even() {
        let output = linspace(10.0, 19.0, 4);

        assert_float_vec_almost_eq(output, vec![10.0, 13.0, 16.0, 19.0]);
    }

    #[test]
    fn linspace_odd() {
        let output = linspace(10.0, 18.0, 5);

        assert_float_vec_almost_eq(output, vec![10.0, 12.0, 14.0, 16.0, 18.0]);
    }

    #[test]
    fn linspace_same_value() {
        let output = linspace(91.0, 91.0, 10);

        assert_float_vec_almost_eq(
            output,
            vec![91.0, 91.0, 91.0, 91.0, 91.0, 91.0, 91.0, 91.0, 91.0, 91.0],
        );
    }

    fn assert_float_vec_almost_eq(a: Vec<f32>, b: Vec<f32>) {
        assert_eq!(a.len(), b.len());
        for (i, (a, b)) in a.into_iter().zip(b.into_iter()).enumerate() {
            assert!((a - b).abs() < FLOAT_THRES, "{a}, {b}, index: {i}");
        }
    }
}
