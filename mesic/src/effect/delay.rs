use super::ApplyEffect;
use crate::consts::SAMPLE_RATE;
use crate::sig::mult;
use shared::model::DelayConfig;

impl ApplyEffect for DelayConfig {
    fn apply(&self, input: &[f32]) -> Vec<f32> {
        // TODO: consider if fractional samples / interpolation make sense here.
        let sample_count = (self.delay_ms * (SAMPLE_RATE as f32) / 1000.0) as usize;
        let mut output = vec![0.0; sample_count];

        output.extend(input);

        mult(output, self.amplitude)
    }
}
