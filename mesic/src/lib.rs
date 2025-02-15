use std::cmp::max;
use strum::{Display, EnumString};
use std::f32::consts::PI;
use shared::model::wave_type::WaveType;
use shared::model::adsr_envelope::AdsrEnvelope;
use shared::model::synth::Synth;
use shared::model::sequence::Sequence;
use shared::model::track::Track;
use shared::model::note::Note;
use shared::types::*;

pub mod io;
mod wave;

use crate::io::*;
use crate::wave::*;

#[derive(EnumString, Display)]
pub enum DemoOption {
    Demo1,
    Demo2,
}

pub struct Demo {
    sequences: Vec<Sequence>,
    bpm: f32,
}

impl Demo {
    pub fn new(demo_option: DemoOption) -> Self {
        match demo_option {
            DemoOption::Demo1 => { 
                Self {
                    sequences: vec! [
                        Sequence {
                            offset: 0.,
                            volume: 1.,
                            synth_index: 0,
                            notes: vec![
                                Note(7., 0.5),
                                Note(7., 1.),
                                Note(7., 1.),
                                Note(3., 0.5),
                                Note(7., 1.),
                                Note(10., 2.),
                                Note(-2., 2.),
                            ],
                        },
                        Sequence {
                            offset: 0.,
                                volume: 0.5,
                            synth_index: 0,
                            notes: vec![
                                Note(-19., 0.5),
                                Note(-19., 1.),
                                Note(-19., 1.),
                                Note(-19., 0.5),
                               Note(-19., 1.),
                                Note(-14., 2.),
                                Note(-26., 2.),
                            ],
                        }
                    ],
                    bpm:160.
                }
            }
            DemoOption::Demo2 => {
                Self {
                    sequences: vec! [
                        Sequence {
                            offset: 0.,
                            volume: 1.,
                            synth_index: 0,
                            notes: vec![
                                Note(7., 0.5),
                                Note(6., 0.5),
                                Note(7., 0.5),
                                Note(6., 0.5),
                                Note(7., 0.5),
                                Note(2., 0.5),
                                Note(5., 0.5),
                                Note(3., 0.5),
                                Note(0., 2.0),
                            ],
                        },
                        Sequence {
                            offset: 0.,
                            volume: 0.5,
                            synth_index: 0,
                            notes: vec![
                                Note(-5., 2.0),
                                Note(-10., 2.0),
                                Note(-12., 2.0),
                            ],
                        }
                    ],
                    bpm: 120.
                }
            } 
        }
    }
}

pub fn demo() -> Result<(), std::io::Error> {
    let output = render(&create_demo_track(DemoOption::Demo1, WaveType::Sine));
    let filename = "out.bin".to_string();
    write_as_bytes(&output, filename)?;
    Ok(())
}

pub fn demo_floats(demo_option: DemoOption, wave: WaveType) -> Vec<f32> {
    render(&create_demo_track(demo_option, wave))
}

pub fn create_demo_track(demo_option: DemoOption, wave: WaveType) -> Track {
    let demo = Demo::new(demo_option);
    let envelope = AdsrEnvelope {
        attack: 0.05,
        decay: 0.1,
        sustain: 0.6,
        release: 0.2,
    };
    let synth = Synth {
        wave,
        envelope,
        volume: 1.,
    };
    Track {
        bpm: demo.bpm, 
        synths: vec![synth],
        sequences: demo.sequences 
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

const REFERENCE_FREQUENCY: Freq = 440.0; // 440 Hz = A4
const SAMPLE_RATE: i32 = 44_100;

// Returns the frequency a given number of semitones above/below A4.
fn freq(semitones: Semitones) -> Freq {
    // powf can't be run at compile time.
    // TODO: make this only run once.
    let semitone_increment: f32 = 2.0_f32.powf(1.0 / 12.0);

    REFERENCE_FREQUENCY * semitone_increment.powf(semitones)
}

fn apply_envelope(x: f32, envelope: &AdsrEnvelope, duration: Beats) -> f32 {
    if duration < envelope.attack + envelope.decay + envelope.release {
        panic!(
            "Envelope {:?} was too short for duration {}",
            envelope, duration
        );
    }

    let x = x / SAMPLE_RATE as f32;

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
                note.0,
                note.1,
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
    semitones: Semitones,
    beats: Beats,
    bpm: Beats,
    volume: f32,
    envelope: &AdsrEnvelope,
    wave_type: WaveType,
) -> Vec<f32> {
    let step = freq(semitones) * 2.0 * PI / (SAMPLE_RATE as f32);
    let range = 0..(SAMPLE_RATE as f32 * beats / bpm * 60.0) as i32;

    range
        .map(|x: i32| {
            make_wave(x as f32 * step, wave_type)
                * volume
                * apply_envelope(x as f32, envelope, beats)
        })
        .collect()
}
