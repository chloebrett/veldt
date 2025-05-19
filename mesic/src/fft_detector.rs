use dasp_envelope::{Detector, detect::Peak};

use crate::{FFT_SAMPLE_SIZE, consts::FFT_PEAK_RELEASE_FRAMES};

const HALF_FFT_SAMPLE_SIZE: usize = FFT_SAMPLE_SIZE / 2;

pub struct FftDetector {
    detectors: [Detector<f32, Peak>; HALF_FFT_SAMPLE_SIZE],
}

impl Default for FftDetector {
    fn default() -> Self {
        let attack_frames = 0.0;
        let detector = |_| Detector::peak(attack_frames, FFT_PEAK_RELEASE_FRAMES);
        Self {
            detectors: core::array::from_fn(detector),
        }
    }
}

impl FftDetector {
    pub fn next(&mut self, next_frame: [f32; HALF_FFT_SAMPLE_SIZE]) -> [f32; HALF_FFT_SAMPLE_SIZE] {
        let nexts: Vec<f32> = self
            .detectors
            .iter_mut()
            .zip(next_frame)
            .map(|(detector, frame)| detector.next(frame))
            .collect();
        nexts.try_into().unwrap()
    }
}
