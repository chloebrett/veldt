use shared::model::{AdsrEnvelope, EqType, PlacedNote, Scale, ScaleValue, WaveType};
use shared::types::{Beats, Octave};
use shared::types::{Freq, KnobPosition, Milliseconds, Volume};

pub fn track_index(action: Action) -> Option<usize> {
    match action {
        Action::SetNoteScaleValue { track_index, .. } => Some(track_index),
        Action::SetNoteOctave { track_index, .. } => Some(track_index),
        Action::SetNoteOffset { track_index, .. } => Some(track_index),
        Action::SetNoteDuration { track_index, .. } => Some(track_index),
        Action::DeleteNote { track_index, .. } => Some(track_index),
        Action::AddNote { track_index, .. } => Some(track_index),
        _ => None,
    }
}

pub fn note_index(action: Action) -> Option<usize> {
    match action {
        Action::SetNoteScaleValue { note_index, .. } => Some(note_index),
        Action::SetNoteOctave { note_index, .. } => Some(note_index),
        Action::SetNoteOffset { note_index, .. } => Some(note_index),
        Action::SetNoteDuration { note_index, .. } => Some(note_index),
        Action::DeleteNote { note_index, .. } => Some(note_index),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    SetKey(ScaleValue),
    SetScale(Scale),
    SetProjectName(String),
    SetBpm(Beats),
    SetVolume(Volume),
    // TODO: split note-related actions into a note-specific nested reducer.
    SetNoteScaleValue {
        track_index: usize,
        note_index: usize,
        note: ScaleValue,
    },
    SetNoteOctave {
        track_index: usize,
        note_index: usize,
        octave: Octave,
    },
    SetNoteOffset {
        track_index: usize,
        note_index: usize,
        offset: Beats,
    },
    SetNoteDuration {
        track_index: usize,
        note_index: usize,
        duration: Beats,
    },
    DeleteNote {
        track_index: usize,
        note_index: usize,
    },
    AddNote {
        track_index: usize,
        note: PlacedNote,
    },
    SetWave {
        generator_index: usize,
        wave: WaveType,
    },
    SetOscCount {
        generator_index: usize,
        osc_count: u32,
    },
    SetDetuneCents {
        generator_index: usize,
        detune_cents: f32,
    },
    SetEnvelope {
        generator_index: usize,
        envelope: AdsrEnvelope,
    },
    SetDelayAmplitude {
        channel_index: usize,
        effect_index: usize,
        amplitude: Volume,
    },
    SetDelayMs {
        channel_index: usize,
        effect_index: usize,
        delay_ms: Milliseconds,
    },
    SetEffectWet {
        channel_index: usize,
        effect_index: usize,
        wet: KnobPosition,
    },
    SetEqKind {
        channel_index: usize,
        effect_index: usize,
        kind: EqType,
    },
    SetEqFc {
        channel_index: usize,
        effect_index: usize,
        fc: Freq,
    },
    SetEqQ {
        channel_index: usize,
        effect_index: usize,
        q: KnobPosition,
    },
}
