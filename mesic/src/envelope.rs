use crate::consts::{MS_PER_SECOND, SAMPLE_RATE};
use shared::model::AdsrEnvelope;
use shared::types::Milliseconds;

/// Applies an ADSR envelope given a particular index within a set of samples.
/// Returns the corresponding amplitude multiplier.
/// E.g. sample_index = 0 would be the first sample, and would therefore return the amplitude
/// corresponding to the start of the envelope (usually zero, unless attack = 0 and there is
/// sustain/decay).
/// Note: this envelope doesn't correctly handle several things.
/// For example, release is treated as part of the main envelope.
pub fn trivial_envelope(sample_index: i32, envelope: &AdsrEnvelope, duration: Milliseconds) -> f32 {
    // TODO: Treat the envelope as a state machine for each synth voice.
    // The release should not be part of the envelope duration.
    let ms = sample_index as f32 / (SAMPLE_RATE as f32) * MS_PER_SECOND;

    // TODO: apply exponential curves to the envelope.
    if ms < envelope.attack {
        // in attack
        ms / envelope.attack
    } else if ms < envelope.attack + envelope.decay {
        // in decay
        (envelope.attack - ms) / envelope.decay * (1.0 - envelope.sustain) + 1.0
    } else if ms < duration - envelope.release {
        // in sustain
        envelope.sustain
    } else {
        // in release
        (duration - ms) / envelope.release * envelope.sustain
    }
}
