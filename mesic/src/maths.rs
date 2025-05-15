/// Returns a vec range with `count` evenly spaced values from `low` to `high`.
pub fn linspace(low: f32, high: f32, count: u32) -> Vec<f32> {
    if count == 0 {
        panic!("Tried to linspace with count == 0");
    }
    if count == 1 {
        let mid = (high + low) / 2.0;
        return vec![mid];
    }

    (0..count)
        .map(|x| (x as f32) * (high - low) / ((count - 1) as f32) + low)
        .collect()
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

pub fn ilerp(a: f32, b: f32, x: f32) -> f32 {
    (x - a) / (b - a)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLOAT_THRES: f32 = 1e-6;

    #[test]
    fn linspace_1() {
        let output = linspace(50.0, 100.0, 1);

        assert_float_vec_almost_eq(output, vec![75.0]);
    }

    #[test]
    fn linspace_2() {
        let output = linspace(10.0, 19.0, 2);

        assert_float_vec_almost_eq(output, vec![10.0, 19.0]);
    }

    #[test]
    fn linspace_even() {
        let output = linspace(10.0, 19.0, 4);

        assert_float_vec_almost_eq(output, vec![10.0, 13.0, 16.0, 19.0]);
    }

    #[test]
    fn linspace_odd() {
        let output = linspace(10.0, 18.0, 5);

        assert_float_vec_almost_eq(output, vec![10.0, 12.0, 14.0, 16.0, 18.0]);
    }

    #[test]
    fn linspace_same_value() {
        let output = linspace(91.0, 91.0, 10);

        assert_float_vec_almost_eq(
            output,
            vec![91.0, 91.0, 91.0, 91.0, 91.0, 91.0, 91.0, 91.0, 91.0, 91.0],
        );
    }

    fn assert_float_vec_almost_eq(a: Vec<f32>, b: Vec<f32>) {
        assert_eq!(a.len(), b.len());
        for (i, (a, b)) in a.into_iter().zip(b.into_iter()).enumerate() {
            assert!((a - b).abs() < FLOAT_THRES, "{a}, {b}, index: {i}");
        }
    }
}
