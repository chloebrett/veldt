use crate::SAMPLE_RATE;
use crate::wave::make_wave;
use ordered_float::OrderedFloat;
use shared::model::{AntiAliasingMode, WaveType};
use shared::types::Freq;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Wave {
    // One full cycle, however long that may be at SAMPLE_RATE.
    pub buffer: Vec<f32>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct WaveKey {
    pub kind: WaveType,
    pub aa: AntiAliasingMode,
    pub freq: OrderedFloat<Freq>,
}

#[derive(Default, Debug)]
pub struct WaveCache {
    cache: HashMap<WaveKey, Wave>,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

// Multiplier on the lookup table size.
// This is necessary when the frequency isn't a clean divisor of the sample rate,
// which is most of the time.
// A value of 10, coupled with lerping, gets rid of most of the audible harmonics.
const FIDELITY: f32 = 10.0;

impl WaveCache {
    /// Returns the amplitude of wave with the given configuration (key) at the given phase.
    /// Phase is between 0.0..1.0.
    /// Note: "key" refers to a HashMap key, not a musical key.
    pub fn get(&mut self, key: &WaveKey, phase: f32) -> f32 {
        debug_assert!((0.0..1.0).contains(&phase));
        let total_samples = FIDELITY * SAMPLE_RATE as f32 / *key.freq;
        let total_samples_len = total_samples.round() as usize;
        let phase_samples = total_samples_len as f32 * phase;

        if let Some(wave) = self.cache.get(key) {
            debug_assert!(wave.buffer.len() == total_samples_len);

            let a = wave.buffer[phase_samples.floor() as usize];
            let b = wave.buffer[(phase_samples.floor() as usize) + 1];
            let t = phase_samples % 1.0;

            return lerp(a, b, t);
        }

        let buffer: Vec<_> = (0..=total_samples_len)
            .map(|x| make_wave(x as f32 / total_samples, key.kind, *key.freq, key.aa))
            .collect();

        debug_assert!(buffer.len() == total_samples_len);

        let a = buffer[phase_samples.floor() as usize];
        let b = buffer[(phase_samples.floor() as usize) + 1];
        let t = phase_samples % 1.0;
        let current_value = lerp(a, b, t);

        self.cache.insert(key.clone(), Wave { buffer });

        current_value
    }
}
