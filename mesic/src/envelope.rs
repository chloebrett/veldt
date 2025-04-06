use crate::consts::{SAMPLE_RATE, SECONDS_PER_MINUTE};
use shared::model::AdsrEnvelope;
use shared::types::Beats;

/// Applies an ADSR envelope given a particular index within a set of samples.
/// Returns the corresponding amplitude multiplier.
/// E.g. sample_index = 0 would be the first sample, and would therefore return the amplitude
/// corresponding to the start of the envelope (usually zero, unless attack = 0 and there is
/// sustain/decay).
pub fn apply_envelope(
    sample_index: f32,
    envelope: &AdsrEnvelope,
    duration: Beats,
    bpm: Beats,
) -> f32 {
    let scale_factor = bpm / SECONDS_PER_MINUTE / duration;
    let beats = sample_index * scale_factor / (SAMPLE_RATE as f32);

    // TODO: apply exponential curves to the envelope.
    if beats < envelope.attack {
        // in attack
        beats / envelope.attack
    } else if beats < envelope.attack + envelope.decay {
        // in decay
        (envelope.attack - beats) / envelope.decay * (1.0 - envelope.sustain) + 1.0
    } else if beats < duration - envelope.release {
        // in sustain
        envelope.sustain
    } else {
        // in release
        (duration - beats) / envelope.release * envelope.sustain
    }
}
