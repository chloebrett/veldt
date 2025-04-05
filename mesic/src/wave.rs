use crate::consts::{SAMPLE_RATE, SECONDS_PER_MINUTE};
use crate::envelope::apply_envelope;
use crate::sig::{freq, sum};
use shared::model::{AdsrEnvelope, PitchName, SimpleWaveConfig, WaveType};
use shared::types::Beats;
use shared::types::Freq;
use std::f32::consts::{PI, TAU};
use std::ops::Range;

const HALF_PI: f32 = 0.5 * PI;
const INV_HALF_PI: f32 = HALF_PI.recip();

fn wave(
    pitch_name: &PitchName,
    beats: Beats,
    bpm: Beats,
    volume: f32,
    envelope: &AdsrEnvelope,
    wave_type: WaveType,
    detune_cents: f32,
    start_index: usize,
) -> Vec<f32> {
    let step = get_step(pitch_name, detune_cents);
    make_range(start_index, beats, bpm)
        .into_iter()
        .map(|x: i32| {
            make_wave(x as f32 * step, wave_type)
                * volume
                * apply_envelope(x as f32, envelope, beats, bpm)
        })
        .collect()
}

/// Returns a multiplier that controls how fast the wave should cycle.
/// This is dependent on the frequency, the detune and the sample rate.
/// A 1Hz frequency will return a value of TAU / SAMPLE_RATE.
/// TODO: need a better abstraction, this isn't very meaningful.
fn get_step(pitch_name: &PitchName, detune_cents: f32) -> f32 {
    freq(*pitch_name) * detune_multiplier(detune_cents) * TAU / (SAMPLE_RATE as f32)
}

fn make_range(start_index: usize, beats: Beats, bpm: Beats) -> Range<i32> {
    let seconds = beats / bpm * SECONDS_PER_MINUTE;
    let samples = SAMPLE_RATE as f32 * seconds;
    (start_index as i32)..(samples as i32)
}

pub fn polyphonic_wave(
    pitch_name: &PitchName,
    beats: Beats,
    bpm: Beats,
    volume: f32,
    config: &SimpleWaveConfig,
    start_index: usize, // allows starting the wave in the middle.
) -> Vec<f32> {
    let detune = config.detune_cents;
    let osc_count = config.osc_count;
    let partial_volume = volume / (osc_count as f32);

    let detune_amounts = linspace(-detune, detune, osc_count);

    let outputs: Vec<Vec<f32>> = detune_amounts
        .iter()
        .map(|det| {
            wave(
                pitch_name,
                beats,
                bpm,
                partial_volume,
                &config.envelope,
                config.wave,
                *det,
                start_index,
            )
        })
        .collect();

    multi_sum(outputs)
}

fn detune_multiplier(cents: f32) -> Freq {
    if cents == 0.0 {
        return 1.0;
    }

    let interval = cents / 100.0;

    // TODO: de-duplicate this.
    let semitone_increment: f32 = 2.0_f32.powf(1.0 / 12.0);
    semitone_increment.powf(interval)
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

fn multi_sum(buffers: Vec<Vec<f32>>) -> Vec<f32> {
    let max_len = buffers.iter().map(|it| it.len()).max().unwrap();
    let mut output: Vec<f32> = vec![0.0; max_len];

    for buf in buffers {
        output = sum(&output, &buf);
    }

    output
}

/// Constructs the given wave at the given phase. x is between 0 and TAU (or will be modulo'd to be
/// between these numbers).
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

#[cfg(test)]
mod tests {
    use super::*;
    
    use crate::sig::freq;
    use assert_float_eq::assert_float_absolute_eq;
    use shared::model::{PitchName, ScaleValue};
    use shared::types::Volume;

    const FLOAT_THRES: f32 = 1e-6;

    #[test]
    fn get_step_with_detune() {
        let output = get_step(
            &PitchName {
                scale_value: ScaleValue::CSharp,
                octave: 3,
            },
            1200.0, // an entire octave of detune!
        );

        let expected = get_step(
            &PitchName {
                scale_value: ScaleValue::CSharp,
                octave: 4, // one octave higher.
            },
            0.0,
        );

        assert_float_absolute_eq!(output, expected, FLOAT_THRES);
    }

    #[test]
    fn make_range_one_second() {
        let start_index = 0;
        let beats = 2.0;
        let bpm = 120.0;
        let output = make_range(start_index, beats, bpm);

        let expected = 0..SAMPLE_RATE;

        assert_eq!(output, expected);
    }

    #[test]
    fn wave_one_second() {
        let beats = 2.0 as Beats;
        let bpm = 120.0 as Beats;
        let volume = 1.0 as Volume;
        let start_position = 0;
        let pitch = PitchName {
            scale_value: ScaleValue::ASharp,
            octave: 6,
        };
        let detune = 0.0;
        let output = wave(
            &pitch,
            beats,
            bpm,
            volume,
            &AdsrEnvelope {
                attack: 0.0,
                decay: 0.0,
                sustain: 1.0,
                release: 0.0,
            },
            WaveType::Sine,
            detune,
            start_position,
        );

        // Manually construct the same value.
        let expected = (0..SAMPLE_RATE)
            .map(|it| (it as f32 / SAMPLE_RATE as f32 * freq(pitch)).sin())
            .collect();

        assert_float_vec_almost_eq(output, expected);
    }

    #[test]
    fn polyphonic_wave_one_second_no_detune() {
        let beats = 2.0 as Beats;
        let bpm = 120.0 as Beats;
        let volume = 1.0 as Volume;
        let start_position = 0;
        let pitch = PitchName {
            scale_value: ScaleValue::ASharp,
            octave: 6,
        };
        let output = polyphonic_wave(
            &pitch,
            beats,
            bpm,
            volume,
            &SimpleWaveConfig {
                detune_cents: 0.0,
                envelope: AdsrEnvelope {
                    attack: 0.0,
                    decay: 0.0,
                    sustain: 1.0,
                    release: 0.0,
                },
                osc_count: 1,
                wave: WaveType::Sine,
            },
            start_position,
        );

        // Manually construct the same value.
        let expected = (0..SAMPLE_RATE)
            .map(|it| (it as f32 / SAMPLE_RATE as f32 * freq(pitch)).sin())
            .collect();

        assert_float_vec_almost_eq(output, expected);
    }

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
