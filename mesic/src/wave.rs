use crate::consts::{NYQUIST, REFERENCE_PITCH, SAMPLE_RATE, SECONDS_PER_MINUTE};
use crate::envelope::apply_envelope;
use dasp_graph::Buffer;
use lazy_static::lazy_static;
use shared::model::{
    AdsrEnvelope, AntiAliasingMode, PitchName, SimpleWaveConfig, SubSynthConfig, WaveType,
};
use shared::types::Beats;
use shared::types::{Freq, PitchValue};
use std::cmp::min;
use std::f32::consts::{PI, TAU};
use std::iter::repeat_n;
use std::ops::Range;

const HALF_PI: f32 = 0.5 * PI;
const RECIP_HALF_PI: f32 = HALF_PI.recip(); // 2 / PI, not 1 / TAU.
const RECIP_PI: f32 = PI.recip();

lazy_static! {
    // The frequency multiplier for a semitone.
    pub static ref SEMITONE_FREQ: f32 = 2.0_f32.powf(1.0 / 12.0);
}

// Returns the frequency based on the distance from reference pitch.
pub fn freq(pitch_name: PitchName) -> Freq {
    let pitch: PitchValue = pitch_name.into();
    let reference: PitchValue = (*REFERENCE_PITCH.pitch_name).into();
    let interval: PitchValue = pitch - reference;

    REFERENCE_PITCH.frequency * SEMITONE_FREQ.powf(interval as f32)
}

pub fn unison_wave(
    pitch_name: &PitchName,
    beats: Beats,
    bpm: Beats,
    config: &SimpleWaveConfig,
    start_index: i32, // allows starting the wave in the middle. Can be negative - if it is, then
                      // -x will return x samples of silence before starting the wave.
) -> Buffer {
    let detune = config.detune_cents;
    let osc_count = config.osc_count;

    let detune_amounts = linspace(-detune, detune, osc_count);

    let outputs: Vec<Buffer> = detune_amounts
        .iter()
        .map(|det| {
            wave(
                pitch_name,
                beats,
                bpm,
                &config.envelope,
                config.wave,
                config.anti_aliasing_mode,
                *det,
                start_index,
            )
        })
        .collect();

    multi_sum(&outputs)
}

fn wave(
    pitch_name: &PitchName,
    beats: Beats,
    bpm: Beats,
    envelope: &AdsrEnvelope,
    wave_type: WaveType,
    anti_aliasing_mode: AntiAliasingMode,
    detune_cents: f32,
    start_index: i32,
) -> Buffer {
    let wave_freq = freq(*pitch_name) * detune_multiplier(detune_cents);
    let step = wave_freq / (SAMPLE_RATE as f32);

    let mut vec: Vec<_> = make_range(start_index, beats, bpm)
        .map(|x: i32| {
            // Handles the case where start_index < 0.
            // This happens when the start of a note is in the middle of a buffer that is being
            // processed.
            if x < 0 {
                return 0.0;
            }
            make_wave(x as f32 * step, wave_type, wave_freq, anti_aliasing_mode)
                * apply_envelope(x as f32, envelope, beats, bpm)
        })
        .collect();

    let mut buffer = Buffer::SILENT;
    // Handles the case where the range is smaller than the output buffer.
    // This happens when a note finishes in the middle of a buffer.
    if vec.len() < Buffer::LEN {
        vec.extend(repeat_n(0.0, Buffer::LEN - vec.len()));
    }
    buffer.copy_from_slice(&vec);
    buffer
}

pub fn sub_synth_wave(
    pitch_name: &PitchName,
    beats: Beats,
    bpm: Beats,
    config: &SubSynthConfig, // TODO: change to OscConfig later when matrix is made
    start_index: i32,
) -> Buffer {
    let buffers: Vec<Buffer> = config
        .oscillators
        .iter()
        .zip(config.envelopes.iter())
        .map(|(osc, envelope)| {
            let detunes = linspace(-osc.unison_detune, osc.unison_detune, osc.osc_count);

            // Create the unison waves
            let unison_waves: Vec<Buffer> = detunes
                .iter()
                .map(|&detune| {
                    wave(
                        pitch_name,
                        beats,
                        bpm,
                        envelope,
                        osc.wave,
                        AntiAliasingMode::Off, // placeholder
                        osc.osc_detune + detune,
                        start_index,
                    )
                })
                .collect();

            // Sum the unison waves
            let mut buf = multi_sum(&unison_waves);

            for x in buf.iter_mut() {
                *x *= osc.volume;
            }
            // TODO: handle pan
            buf
        })
        .collect();

    multi_sum(&buffers)
}

pub fn beats_to_samples(beats: Beats, bpm: Beats) -> u32 {
    let seconds = beats / bpm * SECONDS_PER_MINUTE;
    (SAMPLE_RATE as f32 * seconds) as u32
}

fn make_range(start_index: i32, beats: Beats, bpm: Beats) -> Range<i32> {
    start_index
        ..min(
            beats_to_samples(beats, bpm) as i32,
            Buffer::LEN as i32 + start_index,
        )
}

fn detune_multiplier(cents: f32) -> Freq {
    if cents == 0.0 {
        return 1.0;
    }

    let interval = cents / 100.0;

    SEMITONE_FREQ.powf(interval)
}

/// Returns a vec range with `count` evenly spaced values from `low` to `high`.
fn linspace(low: f32, high: f32, count: u32) -> Vec<f32> {
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

/// Sums the input buffers into a single buffer.
fn multi_sum(inputs: &[Buffer]) -> Buffer {
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
