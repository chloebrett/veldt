use std::fmt::Debug;

use dasp_envelope::{Detector, detect::Peak};
use dasp_frame::Frame;

use crate::{FFT_SAMPLE_SIZE, consts::FFT_PEAK_RELEASE_FRAMES};

const HALF_FFT_SAMPLE_SIZE: usize = FFT_SAMPLE_SIZE / 2;

/// Keep track of max FFT response from signal.
/// Extension of `dasp_envelope::Detector` for detecting over any size arrays.
/// `Detector` limits max frame length to [S; 32].
pub struct FftDetector<F: Frame> {
    detectors: [Detector<F, Peak>; HALF_FFT_SAMPLE_SIZE],
}

impl<F: Frame> Default for FftDetector<F> {
    fn default() -> Self {
        let attack_frames = 0.0;
        let detector = |_| Detector::peak(attack_frames, FFT_PEAK_RELEASE_FRAMES);
        Self {
            detectors: core::array::from_fn(detector),
        }
    }
}

impl<F: Frame + Debug> FftDetector<F>
where
    Vec<F>: FromIterator<<F as Frame>::Signed>,
{
    pub fn next(&mut self, next_frame: [F; HALF_FFT_SAMPLE_SIZE]) -> [F; HALF_FFT_SAMPLE_SIZE] {
        let nexts: Vec<F> = self
            .detectors
            .iter_mut()
            .zip(next_frame)
            .map(|(detector, frame)| detector.next(frame))
            .collect();
        nexts.try_into().expect("Should have collected.")
    }
}
