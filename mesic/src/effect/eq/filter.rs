use super::ApplyEffect;

pub struct FirstOrderFilter {
    pub a0: f32,
    pub a1: f32,
    pub b1: f32,
}

impl ApplyEffect for FirstOrderFilter {
    fn apply(&self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![0.0];
        for i in 1..input.len() {
            let xn = input[i];
            let xn1 = input[i - 1];
            let yn1 = output[i - 1];
            let yn = self.a0 * xn + self.a1 * xn1 - self.b1 * yn1;

            output.push(yn);
        }

        output.drain(0..1);
        output
    }
}

pub struct SecondOrderFilter {
    pub a0: f32,
    pub a1: f32,
    pub a2: f32,
    pub b1: f32,
    pub b2: f32,
}

impl ApplyEffect for SecondOrderFilter {
    fn apply(&self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![0.0, 0.0];
        for i in 2..input.len() {
            let xn = input[i];
            let xn1 = input[i - 1];
            let xn2 = input[i - 2];
            let yn1 = output[i - 1];
            let yn2 = output[i - 2];
            let yn = self.a0 * xn + self.a1 * xn1 + self.a2 * xn2 - self.b1 * yn1 - self.b2 * yn2;

            output.push(yn);
        }

        output.drain(0..2);
        output
    }
}

/// A second order filter which doesn't rely on previous input values.
/// Special case of SecondOrderFilter.
pub struct SecondOrderFeedbackFilter {
    pub a0: f32,
    pub b1: f32,
    pub b2: f32,
}

impl ApplyEffect for SecondOrderFeedbackFilter {
    fn apply(&self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![0.0, 0.0];
        for i in 2..input.len() {
            let xn = input[i];
            let yn1 = output[i - 1];
            let yn2 = output[i - 2];
            let yn = self.a0 * xn - self.b1 * yn1 - self.b2 * yn2;

            output.push(yn);
        }

        output.drain(0..2);
        output
    }
}
