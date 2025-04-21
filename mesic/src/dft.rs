use std::f32::consts::PI;

use ordered_float::Float;

use crate::SAMPLE_RATE;

const TWO_PI: f32 = 2.0 * PI;

pub fn dft(length: usize, signal: Vec<f32>) -> (Vec<f32>, Vec<f32>) {
    let inv_length = 1.0 / length as f32;
    let mut im = vec![0f32; length];
    let mut re = vec![0f32; length];
    for bin in 0..length {
        for frame in 0..length {
            re[bin] += signal[frame]
                * (TWO_PI * frame as f32 * bin as f32 * inv_length).cos()
                * inv_length;
            im[bin] -= signal[frame]
                * (TWO_PI * frame as f32 * bin as f32 * inv_length).sin()
                * inv_length;
        }
    }
    (re, im)
}

pub fn get_freq_response(signal: Vec<f32>, frequency: f32) -> f32 {
    let required_length = (SAMPLE_RATE as f32 / frequency) as usize;
    println!("{:?}", required_length);
    let inv_length = 1.0 / required_length as f32;
    if signal.len() < required_length {
       return 0.0 
    }
    let slice = hann_window(signal[0..required_length].to_vec());
    let re = slice.iter().enumerate().map(|(index, it)| {
        it * (TWO_PI * index as f32 * inv_length).cos() * inv_length
    }).sum::<f32>().powf(2.0);
    let im = slice.iter().enumerate().map(|(index, it)| {
        it * (TWO_PI * index as f32 * inv_length).sin() * inv_length
    }).sum::<f32>().powf(2.0);
    (re + im).sqrt()
}

pub fn hann_window(signal: Vec<f32>) -> Vec<f32> {
    let inv_length = 1.0 / signal.len() as f32;
    signal.iter().enumerate().map(|(index, it)| {
        it * (PI * index as f32 * inv_length).sin().powf(2.0)
    }).collect()
}

#[cfg(test)]
mod tests {
    use shared::model::{PitchName, ScaleValue};

    use crate::wave::freq;

    use super::*;

    #[test]
    fn full_cosine_is_decomposed() {
        let n = 128;
        let b = 3.0;
        let signal = (0..n)
            .map(|index| (TWO_PI * (index as f32) * b as f32 / n as f32).cos())
            .collect();
        let (re, _im) = dft(n, signal);
        let epsilon = 1e-5;
        let mut expected = vec![0f32; n];
        expected[b as usize] = 0.5;
        expected[n - b as usize] = 0.5;
        let approx_diff: bool = re
            .iter()
            .enumerate()
            .all(|(index, value)| (value - expected[index]).abs() < epsilon);
        assert!(approx_diff);
    }

    #[test]
    fn a_note_is_detected() {
        // Arrange
        let pitch = PitchName {
            scale_value: ScaleValue::A,
            octave: 4,
        };
        let samples = 630;
        let input: Vec<f32> = (0..samples as usize)
            .map(|it| (it as f32 / SAMPLE_RATE as f32 * freq(pitch)).cos())
            .collect();
        let res: Vec<f32> = (0..30).map(|it| get_freq_response(input.clone(), it as f32 * 10.0)).collect();
        println!("{:?}", res);
        assert!(1==2);

    }
}
