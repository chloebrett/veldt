use shared::model::{
    AdsrEnvelope, AntiAliasingMode, EqType, PlacedNote, Project, Sample, Scale, ScaleValue,
    TrackId, TrackPlacement, WaveType,
};
use shared::types::{Beats, Freq, GainDB, KnobPosition, Milliseconds, Octave, Volume};

#[derive(Debug, Clone)]
pub enum Action {
    // --- RootSelector ---
    SetKey(ScaleValue),
    SetScale(Scale),
    SetProjectName(String),
    SetBpm(Beats),
    SetVolume(Volume),
    AddTrackPlacement(TrackPlacement),
    DeleteTrackPlacement {
        track_placement_index: usize,
    },
    // Sets the names of loadable projects.
    SetProjectList {
        projects: Vec<String>,
    },
    // Overwrites the whole project.
    SetProject {
        project: Project,
    },
    SetLoadProjectName {
        project_name: String,
    },
    AddSample(Sample),

    // --- TrackSelector ---
    DeleteNote {
        note_index: usize,
    },
    AddNote(PlacedNote),

    // --- NoteSelector ---
    SetNoteScaleValue(ScaleValue),
    SetNoteOctave(Octave),
    SetNoteOffset(Beats),
    SetNoteDuration(Beats),

    // --- GeneratorSelector ---
    SetWave(WaveType),
    SetOscCount(u32),
    SetDetuneCents(KnobPosition),
    SetEnvelope(AdsrEnvelope),
    SetAntiAliasingMode(AntiAliasingMode),
    SetOversampleFactor(u32),

    // --- MixerSelector ---
    MoveEffectUp(/* effect_index= */ usize),
    MoveEffectDown(/* effect_index= */ usize),

    // --- EffectSelector ---
    SetDelayMs(Milliseconds),
    SetEffectWet(KnobPosition),
    SetEffectMute(bool),
    SetEqKind(EqType),
    SetEqFc(Freq),
    SetEqQ(KnobPosition),
    SetEqGain(GainDB),
    SetCompressorThreshold(Volume),
    SetCompressorAttackMs(Milliseconds),
    SetCompressorReleaseMs(Milliseconds),
    SetCompressorRatio(KnobPosition),
    SetCompressorGain(Volume),

    // --- TrackPlacementSelector ---
    SetTrackPlacementTrackId(TrackId),
    SetTrackPlacementOffset(Beats),

    // -- other --
    /// Denotes the reverse-action for an action that isn't reversible.
    /// Applying this is a no-op.
    /// There might be a better way of describing this concept, keep a look out.
    NonReversible,
}
