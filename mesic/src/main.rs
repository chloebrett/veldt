use byteorder::{ByteOrder, LittleEndian};
use std::f32::consts::PI;
use std::fs;
use std::io::Write;

fn main() -> Result<(), std::io::Error> {
    let envelope = AdsrEnvelope {
        attack: 0.5,
        decay: 0.2,
        sustain: 0.5,
        release: 0.2,
    };
    let wave_1 = wave(0.0, 2.0, 0.2, &envelope, WaveType::Triangle);
    let wave_2 = wave(7.0, 1.0, 0.2, &envelope, WaveType::Triangle);

    let envelope = AdsrEnvelope {
        attack: 1.2,
        decay: 0.2,
        sustain: 0.5,
        release: 0.2,
    };
    let wave_3 = wave(7.0, 3.0, 0.2, &envelope, WaveType::Triangle);

    let combined_wave = sum(wave_1, sum(wave_2, wave_3));

    let filename = "out.bin".to_string();
    write_as_bytes(&combined_wave, filename)?;

    Ok(())
}

fn sum(a: Vec<f32>, b: Vec<f32>) -> Vec<f32> {
    a.iter().zip(b.iter()).map(|(&a,&b)| a + b).collect()
}

type Freq = f32;
type Semitones = f32;
type Seconds = f32;

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
    attack: f32,  // duration
    decay: f32,   // duration
    sustain: f32, // amplitude
    release: f32, // duration
}

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

fn apply_envelope(x: f32, envelope: &AdsrEnvelope, duration: Seconds) -> f32 {
    if duration < envelope.attack + envelope.decay + envelope.release {
        panic!("Envelope {:?} was too short for duration {}", envelope, duration);
    }

    let x = x / SAMPLE_RATE as f32;

    if x < envelope.attack { // in attack
        x / envelope.attack
    } else if x < envelope.attack + envelope.decay { // in decay
        (envelope.attack - x) / envelope.decay * (1.0 - envelope.sustain) + 1.0
    } else if x < duration - envelope.release { // in sustain
        envelope.sustain
    } else { // in release
        (duration - x) / envelope.release * envelope.sustain
    }
}

fn wave(
    semitones: Semitones,
    duration: Seconds,
    volume: f32,
    envelope: &AdsrEnvelope,
    wave_type: WaveType,
) -> Vec<f32> {
    let step = freq(semitones) * 2.0 * PI / (SAMPLE_RATE as f32);
    let range = 0..(SAMPLE_RATE as f32 * duration) as i32;

    let wave = match wave_type {
        WaveType::Sine => |x: f32| x.sin(),
        WaveType::Square => |x: f32| square_wave(x),
        WaveType::Saw => |x: f32| saw_wave(x),
        WaveType::Triangle => |x: f32| triangle_wave(x),
    };

    range.map(|x: i32| wave(x as f32 * step) * volume * apply_envelope(x as f32, envelope, duration)).collect()
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
