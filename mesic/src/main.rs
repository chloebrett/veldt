use byteorder::{ByteOrder, LittleEndian};
use std::f32::consts::PI;
use std::fs;
use std::io::Write;
use std::cmp::max;

fn main() -> Result<(), std::io::Error> {
    let envelope = AdsrEnvelope {
        attack: 0.2,
        decay: 0.2,
        sustain: 0.2,
        release: 0.2,
    };
    let synth = Synth {
        wave: WaveType::Sine,
        envelope: envelope,
        volume: 1.,
    };
    let sequence_1 = Sequence {
        offset: 0.,
        volume: 1.,
        synth: &synth,
        notes: vec![
            Note(0., 1.),
            Note(2., 1.),
            Note(4., 1.),
            Note(5., 1.),
            Note(7., 1.),
            Note(9., 1.),
            Note(11., 1.),
            Note(12., 1.),
        ],
    };
    let sequence_2 = Sequence {
        offset: 0.,
        volume: 1.,
        synth: &synth,
        notes: vec![
            Note(0., 1.),
            Note(7., 1.),
            Note(7., 1.),
            Note(0., 1.),
            Note(5., 1.),
            Note(7., 1.),
            Note(12., 1.),
        ],
    };
    let track = Track {
        bpm: 120.,
        sequences: vec![sequence_1, sequence_2],
    };
    let output = render(&track);

    let filename = "out.bin".to_string();
    write_as_bytes(&output, filename)?;

    Ok(())
}

#[derive(Debug)]
struct Track<'a> {
    bpm: Beats,
    sequences: Vec<Sequence<'a>>,
}

#[derive(Debug)]
struct Sequence<'a> {
    offset: Beats,
    volume: Volume,
    synth: &'a Synth,
    notes: Vec<Note>,
}

#[derive(Debug)]
struct Synth {
    wave: WaveType,
    envelope: AdsrEnvelope,
    volume: Volume,
}

#[derive(Debug)]
struct Note(Semitones, Beats);

fn sum(a: Vec<f32>, b: Vec<f32>) -> Vec<f32> {
    let max_len = max(a.len(), b.len());
    let mut output: Vec<f32> = vec![0.0; max_len];

    for i in 0..max_len {
        let ai = a.get(i).unwrap_or(&0.0);
        let bi = b.get(i).unwrap_or(&0.0);
        output[i] = ai + bi;
    }

    output
}

type Volume = f32;
type Freq = f32;
type Semitones = f32;
type Seconds = f32;
type Beats = f32;

const REFERENCE_FREQUENCY: Freq = 440.0; // 440 Hz = A4
const SAMPLE_RATE: i32 = 48_000;

// Returns the frequency a given number of semitones above/below A4.
fn freq(semitones: Semitones) -> Freq {
    // powf can't be run at compile time.
    // TODO: make this only run once.
    let semitone_increment: f32 = 2.0_f32.powf(1.0 / 12.0);

    REFERENCE_FREQUENCY * semitone_increment.powf(semitones)
}

#[derive(Debug)]
struct AdsrEnvelope {
    attack: Beats,
    decay: Beats,
    sustain: Volume,
    release: Beats,
}

#[derive(Debug, Clone, Copy)]
enum WaveType {
    Sine,
    Square,
    Saw,
    Triangle, // TODO: also add a generator for white noise - but it's not constrained by freq.
}

fn square_wave(x: f32) -> f32 {
    let x = x % (2.0 * PI);

    if x > PI { -1.0 } else { 1.0 }
}

fn saw_wave(x: f32) -> f32 {
    let x = x % (2.0 * PI);

    if x > PI { x / PI - 2.0 } else { x / PI }
}

fn triangle_wave(x: f32) -> f32 {
    let x = x % (2.0 * PI);

    if x <= PI / 2.0 {
        x * 2.0 / PI
    } else if x <= 1.5 * PI {
        2.0 * (1.0 - x / PI)
    } else {
        2.0 * (x / PI) - 4.0
    }
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

fn render(track: &Track) -> Vec<f32> {
    let bpm = track.bpm;
    let mut total_wave: Vec<f32> = vec![];

    for sequence in &track.sequences {
        // TODO: use the offset value instead of ignoring.
        let synth = &sequence.synth;
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

    let wave = match wave_type {
        WaveType::Sine => |x: f32| x.sin(),
        WaveType::Square => |x: f32| square_wave(x),
        WaveType::Saw => |x: f32| saw_wave(x),
        WaveType::Triangle => |x: f32| triangle_wave(x),
    };

    range
        .map(|x: i32| wave(x as f32 * step) * volume * apply_envelope(x as f32, envelope, beats))
        .collect()
}

fn write_as_bytes(floats: &Vec<f32>, filename: String) -> Result<(), std::io::Error> {
    let mut bytes: Vec<u8> = vec![0; floats.len() * 4];
    LittleEndian::write_f32_into(&floats.as_slice(), &mut bytes);

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(filename)?;

    let _ = file.write_all(&bytes);

    Ok(())
}
