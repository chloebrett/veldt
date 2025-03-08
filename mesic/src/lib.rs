use shared::model::adsr_envelope::AdsrEnvelope;
use shared::model::note::Note;
use shared::model::pitch_name::PitchName;
use shared::model::project::{Effect, EffectInstance, EffectMeta};
use shared::model::scale_value::ScaleValue;
use shared::model::sequence::Sequence;
use shared::model::synth::Synth;
use shared::model::track::Track;
use shared::model::wave_type::WaveType;
use shared::types::*;
use std::cmp::max;
use std::f32::consts::PI;

pub mod scale;
mod wave;

use crate::wave::*;

pub fn create_track(
    notes: Vec<Note>,
    wave: WaveType,
    bpm: Beats,
    volume: Volume,
    transpose_interval: PitchValue,
    envelope: AdsrEnvelope,
) -> Track {
    let synth = Synth {
        wave,
        envelope,
        volume: volume,
    };
    Track {
        bpm,
        synths: vec![synth],
        sequences: vec![Sequence {
            offset: 0.,
            volume: volume,
            synth_index: 0,
            notes: notes
                .into_iter()
                .map(|note| transpose_note(note, transpose_interval))
                .collect(),
        }],
    }
}

fn transpose_note(note: Note, interval: PitchValue) -> Note {
    Note {
        pitch_name: note.pitch_name + interval,
        beats: note.beats,
    }
}

fn sum(a: Vec<f32>, b: Vec<f32>) -> Vec<f32> {
    let max_len = max(a.len(), b.len());
    let mut output: Vec<f32> = vec![0.0; max_len];

    for (i, item) in output.iter_mut().enumerate() {
        let ai = a.get(i).unwrap_or(&0.0);
        let bi = b.get(i).unwrap_or(&0.0);
        *item = ai + bi;
    }

    output
}

struct ReferencePitch<'a> {
    pitch_name: &'a PitchName,
    frequency: Freq,
}

const REFERENCE_PITCH: ReferencePitch<'static> = ReferencePitch {
    pitch_name: &PitchName {
        scale_value: ScaleValue::A,
        octave: 4,
    },
    frequency: 440.0,
};

const SAMPLE_RATE: i32 = 44_100;

// Returns the frequency based on the distance from reference pitch.
fn freq(pitch_name: &PitchName) -> Freq {
    let reference_value = <PitchName as Into<PitchValue>>::into(REFERENCE_PITCH.pitch_name.clone());
    let other_value = <PitchName as Into<PitchValue>>::into(pitch_name.clone());
    let interval = other_value - reference_value;

    // powf can't be run at compile time.
    // TODO: make this only run once.
    let semitone_increment: f32 = 2.0_f32.powf(1.0 / 12.0);
    REFERENCE_PITCH.frequency * semitone_increment.powf(interval as f32)
}

fn detune_multiplier(cents: f32) -> Freq {
    let interval = cents / 100.0;

    // TODO: de-duplicate this.
    let semitone_increment: f32 = 2.0_f32.powf(1.0 / 12.0);
    semitone_increment.powf(interval)
}

fn apply_envelope(x: f32, envelope: &AdsrEnvelope, duration: Beats, bpm: Beats) -> f32 {
    if duration < envelope.attack + envelope.decay + envelope.release {
        panic!(
            "Envelope {:?} was too short for duration {}",
            envelope, duration
        );
    }

    let scale = bpm / 60.0 / duration;
    let x = x * scale / (SAMPLE_RATE as f32) as f32;

    if x < envelope.attack {
        // in attack
        x / envelope.attack
    } else if x < envelope.attack + envelope.decay {
        // in decay
        (envelope.attack - x) / envelope.decay * (1.0 - envelope.sustain) + 1.0
    } else if x < duration - envelope.release {
        // in sustain
        envelope.sustain
    } else {
        // in release
        (duration - x) / envelope.release * envelope.sustain
    }
}

pub fn render(track: &Track, supersaw_config: SupersawConfig) -> Vec<f32> {
    let bpm = track.bpm;
    let mut total_wave: Vec<f32> = vec![];

    let delay = EffectInstance {
        effect: Effect::SimpleDelay {
            amplitude: 0.5,

            delay_ms: 250.0,
        },
        meta: EffectMeta { id: 0, wet: 0.5 },
    };
    let effects = vec![delay];

    for sequence in &track.sequences {
        // TODO: use the offset value instead of ignoring.
        let synth = &track.synths.get(sequence.synth_index).unwrap();
        println!("{:?}", sequence);

        let mut sequence_wave: Vec<f32> = vec![];
        for note in &sequence.notes {
            println!("{:?}", note);
            let mut wave = polyphonic_wave(
                &note.pitch_name,
                note.beats,
                bpm,
                sequence.volume * synth.volume,
                &synth.envelope,
                synth.wave,
                supersaw_config.clone(),
            );
            sequence_wave.append(&mut wave);
        }

        total_wave = sum(total_wave, sequence_wave);
    }

    apply_effects(total_wave, effects)
}

fn apply_effects(signal: Vec<f32>, effects: Vec<EffectInstance>) -> Vec<f32> {
    let mut output = signal.clone();

    for effect in effects {
        output = apply_effect(output, effect);
    }

    output
}

fn apply_effect(dry_signal: Vec<f32>, effect: EffectInstance) -> Vec<f32> {
    let wet_signal = match effect.effect {
        Effect::SimpleDelay {
            amplitude,
            delay_ms,
        } => apply_delay(dry_signal.clone(), amplitude, delay_ms),
        _ => panic!("Effect not implemented yet!"),
    };

    mix(dry_signal, wet_signal, effect.meta.wet)
}

/// Mixes two signals in the given dry/wet ratio.
fn mix(dry: Vec<f32>, wet: Vec<f32>, ratio: f32) -> Vec<f32> {
    sum(mult(wet, ratio), mult(dry, 1.0 - ratio))
}

fn mult(vec: Vec<f32>, scalar: f32) -> Vec<f32> {
    vec.into_iter().map(|it| it * scalar).collect()
}

fn apply_delay(dry_signal: Vec<f32>, amplitude: Volume, delay_ms: Milliseconds) -> Vec<f32> {
    // TODO: consider if fractional samples / interpolation make sense here.
    let sample_count = (delay_ms * (SAMPLE_RATE as f32) / 1000.0) as usize;
    let mut wet_signal = vec![0.0; sample_count];

    wet_signal.extend(dry_signal);

    mult(wet_signal, amplitude)
}

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
    let step = freq(pitch_name) * detune_multiplier(detune_cents) * 2.0 * PI / (SAMPLE_RATE as f32);
    let range = 0..(SAMPLE_RATE as f32 * beats / bpm * 60.0) as i32;

    range
        .map(|x: i32| {
            make_wave(x as f32 * step, wave_type)
                * volume
                * apply_envelope(x as f32, envelope, beats, bpm)
        })
        .collect()
}

fn polyphonic_wave(
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
