use shared::model::{AdsrEnvelope, Note, Sequence, Synth, Track, WaveType};
use shared::types::{Beats, PitchValue, Volume};

pub fn create_track(
    notes: Vec<Note>,
    wave: WaveType,
    bpm: Beats,
    volume: Volume,
    transpose_interval: PitchValue,
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
            notes: notes
                .into_iter()
                .map(|note| transpose_note(note, transpose_interval))
                .collect(),
        }],
    }
}

fn transpose_note(note: Note, interval: PitchValue) -> Note {
    Note {
        pitch_name: note.pitch_name + interval,
        beats: note.beats,
    }
}
