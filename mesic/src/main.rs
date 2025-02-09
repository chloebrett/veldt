use byteorder::{ByteOrder, LittleEndian};
use std::fs;
use std::io::Write;
use std::f32::consts::PI;

fn main() -> Result<(), std::io::Error> {
    let floats = sine_wave(7.0, 2.0);

    let filename = "out.bin".to_string();
    write_as_bytes(&floats, filename)?;

    Ok(())
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

fn sine_wave(semitones: Semitones, duration: Seconds) -> Vec<f32> {
    let volume = 0.5;

    let step = freq(semitones) * 2.0 * PI / (SAMPLE_RATE as f32);
    let range = 0 .. (SAMPLE_RATE as f32 * duration) as i32;

    range
        .map(|x: i32| (x as f32 * step).sin() * volume)
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
