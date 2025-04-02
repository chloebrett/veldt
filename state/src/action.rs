use shared::model::{
    AdsrEnvelope, EqType, PlacedNote, Project, Sample, Scale, ScaleValue, TrackId, TrackPlacement,
    WaveType,
};
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
    SetDelayMs(Milliseconds),
    SetEffectWet(KnobPosition),
    SetEqKind(EqType),
    SetEqFc(Freq),
    SetEqQ(KnobPosition),
    AddTrackPlacement(TrackPlacement),
    DeleteTrackPlacement {
        track_placement_index: usize,
    },
    SetTrackPlacementTrackId(TrackId),
    SetTrackPlacementOffset(Beats),
    // Sets the names of loadable projects.
    SetProjectList {
        projects: Vec<String>,
    },
    SetCompressorThreshold(Volume),
    SetCompressorAttackMs(Milliseconds),
    SetCompressorReleaseMs(Milliseconds),
    SetCompressorRatio(KnobPosition),
    SetCompressorGain(Volume),
    // Overwrites the whole project.
    SetProject {
        project: Project,
    },
    SetLoadProjectName {
        project_name: String,
    },
    AddSample(Sample),

    /// Denotes the reverse-action for an action that isn't reversible.
    /// Applying this is a no-op.
    /// There might be a better way of describing this concept, keep a look out.
    NonReversible,
}
