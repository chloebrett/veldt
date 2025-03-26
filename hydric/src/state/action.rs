use shared::model::{AdsrEnvelope, EqType, Sample, PlacedNote, Scale, ScaleValue, Track, WaveType};
use shared::types::{Freq, KnobPosition, Milliseconds, Volume, Beats, Octave};

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
    SaveTrack {
        track_index: usize,
    },
    LoadTrackList,
    SetTrackList {
        tracks: Vec<String>,
    },
    SetTrack {
        track_index: usize,
        track: Track,
    },
    LoadTrack,
    SetLoadTrackName {
        track_name: String,
    },
    ClearLoadTrackPromise,
    ClearSaveTrackPromise,
    ClearTrackListPromise,
    LoadSample {
        filename: String,
    },
    ClearLoadSamplePromise,
    AddSample(Sample),

    /// Denotes the reverse-action for an action that isn't reversible.
    /// Applying this is a no-op.
    /// There might be a better way of describing this concept, keep a look out.
    NonReversible,
}
