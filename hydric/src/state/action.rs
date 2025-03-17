use shared::model::{AdsrEnvelope, EqType, PlacedNote, Scale, ScaleValue, WaveType};
use shared::types::{Beats, Octave};
use shared::types::{Freq, KnobPosition, Milliseconds, Volume};

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
