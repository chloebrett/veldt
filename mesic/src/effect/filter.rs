pub fn first_degree_filter(input: Vec<f32>, a0: f32, a1: f32, b1: f32) -> Vec<f32> {
    let mut output: Vec<f32> = vec![0.0];
    for i in 1..input.len() {
        let xn = input[i];
        let xn1 = input[i - 1];
        let yn1 = output[i - 1];
        let yn = a0 * xn + a1 * xn1 - b1 * yn1;

        output.push(yn);
    }

    output.drain(0..1);
    output
}

pub fn second_degree_filter(
    input: Vec<f32>,
    a0: f32,
    a1: f32,
    a2: f32,
    b1: f32,
    b2: f32,
) -> Vec<f32> {
    let mut output: Vec<f32> = vec![0.0, 0.0];
    for i in 2..input.len() {
        let xn = input[i];
        let xn1 = input[i - 1];
        let xn2 = input[i - 2];
        let yn1 = output[i - 1];
        let yn2 = output[i - 2];
        let yn = a0 * xn + a1 * xn1 + a2 * xn2 - b1 * yn1 - b2 * yn2;

        output.push(yn);
    }

    output.drain(0..2);
    output
}

/// A second degree filter which doesn't rely on previous input values.
/// Special case of second_degree_filter.
pub fn second_degree_feedback_filter(input: Vec<f32>, a0: f32, b1: f32, b2: f32) -> Vec<f32> {
    let mut output: Vec<f32> = vec![0.0, 0.0];
    for i in 2..input.len() {
        let xn = input[i];
        let yn1 = output[i - 1];
        let yn2 = output[i - 2];
        let yn = a0 * xn - b1 * yn1 - b2 * yn2;

        output.push(yn);
    }

    output.drain(0..2);
    output
}
