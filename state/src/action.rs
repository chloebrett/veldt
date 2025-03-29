use shared::model::{AdsrEnvelope, EqType, PlacedNote, Sample, Scale, ScaleValue, Track, WaveType};
use shared::types::{Beats, Freq, KnobPosition, Milliseconds, Octave, Volume};

#[derive(Debug, Clone)]
pub enum Action {
    SetKey(ScaleValue),
    SetScale(Scale),
    SetProjectName(String),
    SetBpm(Beats),
    SetVolume(Volume),
    SetNoteScaleValue(ScaleValue),
    SetNoteOctave(Octave),
    SetNoteOffset(Beats),
    SetNoteDuration(Beats),
    DeleteNote {
        note_index: usize,
    },
    AddNote(PlacedNote),
    SetWave(WaveType),
    SetOscCount(u32),
    SetDetuneCents(KnobPosition),
    SetEnvelope(AdsrEnvelope),
    SetDelayAmplitude(Volume),
    SetDelayMs(Milliseconds),
    SetEffectWet(KnobPosition),
    SetEqKind(EqType),
    SetEqFc(Freq),
    SetEqQ(KnobPosition),
    SetTrackList {
        tracks: Vec<String>,
    },
    SetTrack {
        track_index: usize,
        track: Track,
    },
    SetLoadTrackName {
        track_name: String,
    },
    AddSample(Sample),

    /// Denotes the reverse-action for an action that isn't reversible.
    /// Applying this is a no-op.
    /// There might be a better way of describing this concept, keep a look out.
    NonReversible,
}
