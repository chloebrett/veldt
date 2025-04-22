use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

/// Apply Discrete Fourier Transform to Singal
/// Creates a vector the length of signal.
/// Easch value of the vector corresponds to the signal response for the frequency window:
///      `n * N`
/// Where `n` is the index of the value and N is `SAMPLE_RATE / signal.len()`.
/// E.g. the 3rd value of a response where the sample rate is `44_100` hz and the signal is
/// 1024 samples long would be the window of 86.1-129.1 Hz.
/// See Ch. 20 of Designing Audio Effect Plugins in C++.
pub fn dft(signal: Vec<f32>) -> Vec<f32> {
    let length = signal.len();
    let inv_length = 1.0 / length as f32;
    let mut im = vec![0f32; length];
    let mut re = vec![0f32; length];
    let mut output = vec![0f32; length];
    for bin in 0..length {
        for (frame, value) in signal.iter().enumerate().take(length) {
            // Calculate real (cosine phase) resopnse
            re[bin] += value * (TWO_PI * frame as f32 * bin as f32 * inv_length).cos() * inv_length;
            // Calculate imaginary (sine phase) response
            im[bin] += value * (TWO_PI * frame as f32 * bin as f32 * inv_length).sin() * inv_length;
        }
        // Calculate magnitude of response
        output[bin] = (re[bin].powi(2) + im[bin].powi(2)).sqrt()
    }
    output
}

/// Group DFT results into bins based on frequency log2 value.
/// This makes responses more readable with higher resolution on lower frequencies.
// TODO improve this implementation so that it calculates the log exponent needed to create bin
// sizes that perfect fill up the response space.
pub fn make_log_buckets(response: Vec<f32>, bins: usize) -> Vec<f32> {
    // Halve response as DFT can only discern signal responses for `signal.len()/2` windows.
    let positive_response = response[0..response.len() / 2].to_vec();
    let mut output = vec![0f32; bins];
    for (index, value) in positive_response.iter().enumerate() {
        // Add 2 to index so that 0th and 1st response are grouped together.
        // Subtract 1 from bin to start at bin index == 0.
        let bin = (index as f32 + 2.0).log2() as usize - 1;
        if bin < bins {
            output[bin] += value
        } else {
            break;
        }
    }
    output
}

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
    use shared::model::{PitchName, ScaleValue};

    use crate::wave::freq;

    use super::*;

    use crate::SAMPLE_RATE;

    #[test]
    fn full_cosine_is_decomposed() {
        // ARRANGE
        let sample_size = 128;
        let harmonic = 3;
        let signal = (0..sample_size)
            .map(|index| (TWO_PI * (index as f32) * harmonic as f32 / sample_size as f32).cos())
            .collect();
        // ACT
        let output = dft(signal);
        // ASSERT
        let epsilon = 1e-5;
        let mut expected = vec![0f32; sample_size];
        expected[harmonic] = 0.5;
        expected[sample_size - harmonic] = 0.5;
        let approx_diff: bool = output
            .iter()
            .enumerate()
            .all(|(index, value)| (value - expected[index]).abs() < epsilon);
        assert!(approx_diff);
    }

    #[test]
    fn full_sine_is_decomposed() {
        // ARRANGE
        let sample_size = 128;
        let harmonic = 3;
        let signal = (0..sample_size)
            .map(|index| (TWO_PI * (index as f32) * harmonic as f32 / sample_size as f32).sin())
            .collect();
        // ACT
        let output = dft(signal);
        // ASSERT
        let epsilon = 1e-5;
        let mut expected = vec![0f32; sample_size];
        expected[harmonic] = 0.5;
        expected[sample_size - harmonic] = 0.5;
        let approx_diff: bool = output
            .iter()
            .enumerate()
            .all(|(index, value)| (value - expected[index]).abs() < epsilon);
        assert!(approx_diff);
    }

    #[test]
    fn note_response_max_bucket_is_correct_frequency() {
        let pitch = PitchName {
            scale_value: ScaleValue::A,
            octave: 4,
        };
        let samples = 1024;
        // The frequency window of each values returned in the DFT response vector.
        let freq_window = SAMPLE_RATE as f32 / samples as f32;
        let input: Vec<f32> = (0..samples as usize)
            .map(|it| (TWO_PI * it as f32 / SAMPLE_RATE as f32 * freq(pitch)).sin())
            .collect();
        // Act
        let response = dft(input);
        let bins = make_log_buckets(response, 10);
        let expected_max_bin = (freq(pitch) / freq_window + 2.0).log2() as usize - 1;
        let (max_bin, _) = bins
            .into_iter()
            .enumerate()
            .max_by_key(|(_index, it)| ordered_float::OrderedFloat(*it))
            .unwrap();
        assert_eq!(max_bin, expected_max_bin);
    }
}
