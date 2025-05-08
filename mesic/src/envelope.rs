use crate::consts::{SAMPLE_RATE, SECONDS_PER_MINUTE};
use shared::model::AdsrEnvelope;
use shared::types::Beats;

const MS_PER_SECOND: f32 = 1000.0;

/// Applies an ADSR envelope given a particular index within a set of samples.
/// Returns the corresponding amplitude multiplier.
/// E.g. sample_index = 0 would be the first sample, and would therefore return the amplitude
/// corresponding to the start of the envelope (usually zero, unless attack = 0 and there is
/// sustain/decay).
/// Note: this envelope doesn't correctly handle several things.
/// For example, release is treated as part of the main envelope.
pub fn trivial_envelope(
    sample_index: i32,
    envelope: &AdsrEnvelope,
    duration: Beats,
) -> f32 {
    // TODO: Treat the envelope as a state machine for each synth voice.
    // The release should not be part of the envelope duration.
    let beats = sample_index as f32 / (SAMPLE_RATE as f32) * MS_PER_SECOND;

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
