use crate::consts::SAMPLE_RATE;
use crate::envelope::apply_envelope;
use crate::sig::{freq, sum};
use shared::model::{AdsrEnvelope, PitchName, WaveType};
use shared::types::Beats;
use shared::types::Freq;
use std::f32::consts::{PI, TAU};

const HALF_PI: f32 = 0.5 * PI;
const INV_HALF_PI: f32 = HALF_PI.recip();

#[derive(Clone, PartialEq)]
pub struct SupersawConfig {
    pub osc_count: u32,

    pub detune_cents: f32,
}

fn wave(
    pitch_name: &PitchName,
    beats: Beats,
    bpm: Beats,
    volume: f32,
    envelope: &AdsrEnvelope,
    wave_type: WaveType,
    detune_cents: f32,
) -> Vec<f32> {
    let step =
        freq(*pitch_name) * detune_multiplier(detune_cents) * 2.0 * PI / (SAMPLE_RATE as f32);
    let range = 0..(SAMPLE_RATE as f32 * beats / bpm * 60.0) as i32;

    range
        .map(|x: i32| {
            make_wave(x as f32 * step, wave_type)
                * volume
                * apply_envelope(x as f32, envelope, beats, bpm)
        })
        .collect()
}

pub fn polyphonic_wave(
    pitch_name: &PitchName,
    beats: Beats,
    bpm: Beats,
    volume: f32,
    envelope: &AdsrEnvelope,
    wave_type: WaveType,
    supersaw_config: SupersawConfig,
) -> Vec<f32> {
    let detune = supersaw_config.detune_cents;
    let osc_count = supersaw_config.osc_count;
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
                envelope,
                wave_type,
                *det,
            )
        })
        .collect();

    multi_sum(outputs)
}

fn detune_multiplier(cents: f32) -> Freq {
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
        .map(|x| (x as f32) * (high - low) / (count as f32 - 1.0) - low)
        .collect()
}

fn multi_sum(buffers: Vec<Vec<f32>>) -> Vec<f32> {
    let max_len = buffers.iter().map(|it| it.len()).max().unwrap();
    let mut output: Vec<f32> = vec![0.0; max_len];

    for buf in buffers {
        output = sum(output, buf);
    }

    output
}

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
