use rustfft::{
    Fft, FftDirection,
    algorithm::Radix4,
    num_complex::{Complex, ComplexFloat},
};
use shared::serialize::map_vec;
use std::f32::consts::PI;

/// Perform Fast Fourier Transform on Signal to find component frequencies.
pub fn fft(signal: Vec<f32>) -> Vec<f32> {
    let signal_length = signal.len();
    // Initialise a best algorithm for FFT.
    // Signal length must be a power of 2
    let fft = Radix4::new(signal_length, FftDirection::Forward);
    let mut complex_signal: Vec<Complex<f32>> = map_vec(signal);
    let complex_array = &mut complex_signal[0..signal_length];
    fft.process((complex_array).into());
    // Find magnitude of complex output and normalise by array length.
    complex_signal
        .iter()
        .map(|value| value.abs() / signal_length as f32)
        .collect()
}

/// Group FFT results into bins based on frequency log2 value.
/// This makes responses more readable with higher resolution on lower frequencies.
// TODO improve this implementation so that it calculates the log exponent needed to create bin
// sizes that perfect fill up the response space.
pub fn make_log_buckets(response: Vec<f32>, bins: usize) -> Vec<f32> {
    // Halve response as FFT can only discern signal responses for `signal.len()/2` windows.
    let positive_response = response[0..response.len() / 2].to_vec();
    let mut output = vec![0f32; bins];
    for (index, value) in positive_response.iter().enumerate() {
        // Add 2 to index so that 0th and 1st response are grouped together.
        // Subtract 1 from bin to start at bin index == 0.
        let bin = ((index + 2).ilog2() - 1) as usize;
        if bin < bins {
            output[bin] += value
        } else {
            break;
        }
    }
    output
}

/// A filter to improve the results of FFT when applied before transformation.
// TODO try the Hann Window in DASP to see if it is more efficient.
pub fn hann_window(signal: Vec<f32>) -> Vec<f32> {
    let inv_length = 1.0 / signal.len() as f32;
    signal
        .iter()
        .enumerate()
        .map(|(index, value)| value * (PI * index as f32 * inv_length).sin().powi(2))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use ordered_float::OrderedFloat;
    use shared::{
        model::{PitchName, ScaleValue},
        serialize::map_vec,
    };

    use super::*;

    use crate::{SAMPLE_RATE, wave::freq};

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
