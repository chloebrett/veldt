use crate::consts::RECIP_SAMPLE_RATE;
use crate::eq::BiquadCoefficients;
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub struct FrequencyResponsePoint {
    pub frequency: f32, // Hz
    pub gain: f32,      // dB
}

// Used to generate frequency response curve for visualising EQ
// Essentially only need the magnitude of the H(z) formula as it represents the gain which is what gets plotted:
// H(z) = (a0 + a1*z^-1 + a2*z^-2)/(b0 + b1*z^-1 + b2*z^-2)
// To get the frequency response H(z) needs to be evaluated on the unit circle which means z needs to be replaced with e^(j*w*T)
// j is just i as in the imaginary number, w is the angualar frequency which is 2*pi*frequency, T is sampling period (1/SAMPLE_RATE)
// Computing H(e^(j*w*T)) directly is kind of messy so expand everything using Euler which will make it easier to get the numbers
// needed to calculate the magnitude which is all we're interested in anyway. See all the maths working here:
// https://drive.google.com/drive/folders/1flyEcv6HdQu_zPauZ6WSmkOsGYMAaVzx
pub fn calculate_frequency_response(
    coeffs: &BiquadCoefficients,
    num_points: usize,
    min_freq: f32,
    max_freq: f32,
) -> Vec<FrequencyResponsePoint> {
    let mut points = Vec::with_capacity(num_points);

    // Calculate frequencies on a logarithmic scale using log10 to space evenly
    let log_min_freq = min_freq.log10();
    let log_max_freq = max_freq.log10();
    let delta_log_freq = (log_max_freq - log_min_freq) / (num_points as f32 - 1.0);

    for i in 0..num_points {
        // Calculate the current frequency in Hz
        let current_log_freq = log_min_freq + (i as f32 * delta_log_freq);
        let freq = 10.0_f32.powf(current_log_freq);

        // Calculate the angular frequency and w * sample_period
        let w_t = TAU * freq * RECIP_SAMPLE_RATE;

        // Calculate terms for the complex numerator and denominator
        let cos_w_t = w_t.cos();
        let sin_w_t = w_t.sin();
        let cos_2_w_t = (2.0 * w_t).cos();
        let sin_2_w_t = (2.0 * w_t).sin();

        // Numerator (N = RN + j*IN)
        let real_numerator = coeffs.a0 + coeffs.a1 * cos_w_t + coeffs.a2 * cos_2_w_t;
        let imaginary_numerator = -coeffs.a1 * sin_w_t - coeffs.a2 * sin_2_w_t;

        // Denominator (D = RD + j*ID)
        let real_denominator = 1.0 + coeffs.b1 * cos_w_t + coeffs.b2 * cos_2_w_t;
        let imaginary_denominator = -coeffs.b1 * sin_w_t - coeffs.b2 * sin_2_w_t;

        // Calculate the magnitude squared of the denominator: |D|^2 = RD^2 + ID^2
        let denominator_magnitude_sq =
            real_denominator * real_denominator + imaginary_denominator * imaginary_denominator;

        // To avoid accidentally dividing by zero just set magnitude to zero if the denominator magnitude is somehow zero or lower
        let magnitude = if denominator_magnitude_sq <= 0.0 {
            0.0
        } else {
            let numerator_magnitude = (real_numerator * real_numerator
                + imaginary_numerator * imaginary_numerator)
                .sqrt();
            let denominator_magnitude = denominator_magnitude_sq.sqrt();
            numerator_magnitude / denominator_magnitude
        };

        // Convert magnitude to decibels
        let gain = if magnitude > 0.0 {
            20.0 * magnitude.log10()
        } else {
            -60.0 // if magnitude is zero then just set the gain_db to something very low
        };

        points.push(FrequencyResponsePoint {
            frequency: freq,
            gain,
        });
    }

    points
}
