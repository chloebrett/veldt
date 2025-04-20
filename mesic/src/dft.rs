use std::f32::consts::PI;

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

#[cfg(test)]
mod tests {
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
}
