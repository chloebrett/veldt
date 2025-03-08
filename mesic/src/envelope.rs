use crate::consts::SAMPLE_RATE;
use shared::model::AdsrEnvelope;
use shared::types::Beats;

pub fn apply_envelope(x: f32, envelope: &AdsrEnvelope, duration: Beats, bpm: Beats) -> f32 {
    if duration < envelope.attack + envelope.decay + envelope.release {
        panic!(
            "Envelope {:?} was too short for duration {}",
            envelope, duration
        );
    }

    let scale = bpm / 60.0 / duration;
    let x = x * scale / (SAMPLE_RATE as f32);

    if x < envelope.attack {
        // in attack
        x / envelope.attack
    } else if x < envelope.attack + envelope.decay {
        // in decay
        (envelope.attack - x) / envelope.decay * (1.0 - envelope.sustain) + 1.0
    } else if x < duration - envelope.release {
        // in sustain
        envelope.sustain
    } else {
        // in release
        (duration - x) / envelope.release * envelope.sustain
    }
}
