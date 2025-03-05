use shared::model::adsr_envelope::AdsrEnvelope;
use shared::model::note::Note;
use shared::model::pitch_name::PitchName;
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
    // powf can't be run at compile time.
    // TODO: make this only run once.

    let reference_value = <PitchName as Into<PitchValue>>::into(REFERENCE_PITCH.pitch_name.clone());
    let other_value = <PitchName as Into<PitchValue>>::into(pitch_name.clone());
    let interval = other_value - reference_value;
    let semitone_increment: f32 = 2.0_f32.powf(1.0 / 12.0);
    REFERENCE_PITCH.frequency * semitone_increment.powf(interval as f32)
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

pub fn render(track: &Track) -> Vec<f32> {
    let bpm = track.bpm;
    let mut total_wave: Vec<f32> = vec![];

    for sequence in &track.sequences {
        // TODO: use the offset value instead of ignoring.
        let synth = &track.synths.get(sequence.synth_index).unwrap();
        println!("{:?}", sequence);

        let mut sequence_wave: Vec<f32> = vec![];
        for note in &sequence.notes {
            println!("{:?}", note);
            let mut wave = wave(
                &note.pitch_name,
                note.beats,
                bpm,
                sequence.volume * synth.volume,
                &synth.envelope,
                synth.wave,
            );
            sequence_wave.append(&mut wave);
        }

        total_wave = sum(total_wave, sequence_wave);
    }

    total_wave
}

fn wave(
    pitch_name: &PitchName,
    beats: Beats,
    bpm: Beats,
    volume: f32,
    envelope: &AdsrEnvelope,
    wave_type: WaveType,
) -> Vec<f32> {
    let step = freq(pitch_name) * 2.0 * PI / (SAMPLE_RATE as f32);
    let range = 0..(SAMPLE_RATE as f32 * beats / bpm * 60.0) as i32;

    range
        .map(|x: i32| {
            make_wave(x as f32 * step, wave_type)
                * volume
                * apply_envelope(x as f32, envelope, beats, bpm)
        })
        .collect()
}
