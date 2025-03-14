use crate::consts::REFERENCE_PITCH;
use shared::model::PitchName;
use shared::types::{Freq, PitchValue};
use std::cmp::max;

pub fn sum(a: &Vec<f32>, b: &Vec<f32>) -> Vec<f32> {
    let max_len = max(a.len(), b.len());
    let mut output: Vec<f32> = vec![0.0; max_len];

    for (i, item) in output.iter_mut().enumerate() {
        let ai = a.get(i).unwrap_or(&0.0);
        let bi = b.get(i).unwrap_or(&0.0);
        *item = ai + bi;
    }

    output
}

pub fn mult(vec: &[f32], scalar: f32) -> Vec<f32> {
    vec.into_iter().map(|it| it * scalar).collect()
}

// Returns the frequency based on the distance from reference pitch.
pub fn freq(pitch_name: PitchName) -> Freq {
    let pitch: PitchValue = pitch_name.into();
    let reference: PitchValue = (*REFERENCE_PITCH.pitch_name).into();
    let interval: PitchValue = pitch - reference;

    // powf can't be run at compile time.
    // TODO: make this only run once.
    let semitone_increment: f32 = 2.0_f32.powf(1.0 / 12.0);
    REFERENCE_PITCH.frequency * semitone_increment.powf(interval as f32)
}
