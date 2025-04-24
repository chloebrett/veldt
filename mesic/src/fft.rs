use rustfft::{num_complex::{Complex, ComplexFloat}, FftPlanner};
use shared::serialize::map_vec;

pub fn fft(signal: Vec<f32>) -> Vec<f32> {
    let signal_length = signal.len();
    let mut complex_signal: Vec<Complex<f32>> = map_vec(signal); 
    let complex_array = &mut complex_signal[0..signal_length];
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(signal_length);
    fft.process((complex_array).into());
    complex_signal.iter().map(|value| {
        value.abs()
    }).collect()
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use ordered_float::OrderedFloat;
    use shared::{
        model::{PitchName, ScaleValue},
        serialize::map_vec,
    };

    use crate::{dft::make_log_buckets, wave::freq};

    use super::*;

    use crate::SAMPLE_RATE;

    const EPSILON: f32 = 1e-5;

    fn index_of_max<T: Ord>(vec: &Vec<T>) -> usize {
        let (max_index, _) = vec
            .into_iter()
            .enumerate()
            .max_by_key(|(_index, it)| *it)
            .unwrap();
        max_index
    }

    #[test]
    fn full_cosine_is_decomposed() {
        // ARRANGE
        let sample_size = 128;
        let harmonic = 3;
        let signal = (0..sample_size)
            .map(|index| (TAU * (index as f32) * harmonic as f32 / sample_size as f32).cos())
            .collect();
        // ACT
        let output = fft(signal);
        // ASSERT
        let mut expected = vec![0f32; sample_size];
        expected[harmonic] = 0.5;
        expected[sample_size - harmonic] = 0.5;
        let approx_diff: bool = output
            .iter()
            .enumerate()
            .all(|(index, value)| (value - expected[index]).abs() < EPSILON);
        assert!(approx_diff);
    }

    #[test]
    fn full_sine_is_decomposed() {
        // ARRANGE
        let sample_size = 128;
        let harmonic = 3;
        let signal = (0..sample_size)
            .map(|index| (TAU * (index as f32) * harmonic as f32 / sample_size as f32).sin())
            .collect();
        // ACT
        let output = fft(signal);
        // ASSERT
        let mut expected = vec![0f32; sample_size];
        expected[harmonic] = 0.5;
        expected[sample_size - harmonic] = 0.5;
        let approx_diff: bool = output
            .iter()
            .enumerate()
            .all(|(index, value)| (value - expected[index]).abs() < EPSILON);
        assert!(approx_diff);
    }

    #[test]
    fn note_response_max_bucket_is_correct_frequency() {
        let pitch = PitchName {
            scale_value: ScaleValue::A,
            octave: 4,
        };
        let samples = 1024;
        // The frequency window of each values returned in the fft response vector.
        let freq_window = SAMPLE_RATE as f32 / samples as f32;
        let input: Vec<f32> = (0..samples as usize)
            .map(|it| (TAU * it as f32 / SAMPLE_RATE as f32 * freq(pitch)).sin())
            .collect();
        // Act
        let response = fft(input);
        let bins = make_log_buckets(response, 10);
        let expected_max_bin = (freq(pitch) / freq_window + 2.0).log2() as usize - 1;
        let max_bin = index_of_max(&map_vec::<f32, OrderedFloat<f32>>(bins));
        assert_eq!(max_bin, expected_max_bin);
    }
}
