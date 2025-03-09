use shared::model::{AdsrEnvelope, Note, Sequence, Synth, Track, WaveType};
use shared::types::{Beats, Volume};

pub fn create_track(
    notes: Vec<Note>,
    wave: WaveType,
    bpm: Beats,
    volume: Volume,
    envelope: AdsrEnvelope,
) -> Track {
    let synth = Synth {
        wave,
        envelope,
        volume,
    };
    Track {
        bpm,
        synths: vec![synth],
        sequences: vec![Sequence {
            offset: 0.,
            volume,
            synth_index: 0,
            notes,
        }],
    }
}
