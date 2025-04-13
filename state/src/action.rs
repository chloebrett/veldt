use crate::action_proto::{ActionProto, action_proto::Kind as ActionKind};
use shared::model::{
    AdsrEnvelope, AntiAliasingMode, EffectInstance, EqType, PitchName, PlacedNote, Project, Sample,
    Scale, ScaleValue, TrackId, TrackPlacement, WaveType,
};
use shared::types::{Beats, Freq, GainDB, KnobPosition, Milliseconds, Octave, Volume};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // --- RootSelector ---
    SetKey(ScaleValue),
    SetScale(Scale),
    SetProjectName(String),
    SetBpm(Beats),
    SetVolume(Volume),
    AddTrackPlacement(TrackPlacement),
    DeleteTrackPlacement(usize),
    // Sets the names of loadable projects.
    SetProjectList(Vec<String>),
    // Overwrites the whole project.
    SetProject(Project),
    SetLoadProjectName(String),
    AddSample(Sample),

    // --- TrackSelector ---
    DeleteNote(usize),
    AddNote(PlacedNote),
    SetTrackOffset(Beats),

    // --- NoteSelector ---
    SetNoteScaleValue(ScaleValue),
    SetNoteOctave(Octave),
    SetNotePitchName(PitchName),
    SetNoteOffset(Beats),
    SetNoteDuration(Beats),

    // --- GeneratorSelector ---
    // TODO: rename this to SetVolume, and just differentiate by the selector. (Apply this idea to
    // several action types).
    SetGeneratorVolume(Volume),
    SetGeneratorMute(bool),
    SetGeneratorPan(KnobPosition),
    SetWave(WaveType),
    SetOscCount(u32),
    SetDetuneCents(KnobPosition),
    SetEnvelope(AdsrEnvelope),
    SetAntiAliasingMode(AntiAliasingMode),
    SetOversampleFactor(u32),

    // --- MixerSelector ---
    MoveEffectUp(/* effect_index= */ usize),
    MoveEffectDown(/* effect_index= */ usize),
    DeleteEffect(/* effect_index= */ usize),
    AddEffect(EffectInstance),

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
    SetModDelayMinDepth(u32),
    SetModDelayMaxDepth(u32),
    SetModDelayLfoFreq(f32),
    SetModDelayLfoType(WaveType),

    // --- TrackPlacementSelector ---
    SetTrackPlacementTrackId(TrackId),
    SetTrackPlacementOffset(Beats),

    // -- other --
    /// Denotes that the mouse has been released from a UI element, finalizing its value.
    /// This is how we know to flatten (in the undo stack) actions that modify floats.
    /// This event is just a marker, it doesn't get passed on to the reducers.
    Release,
    /// Denotes the reverse-action for an action that isn't reversible.
    /// Applying this is a no-op.
    /// There might be a better way of describing this concept, keep a look out.
    NonReversible,
}

impl From<ActionProto> for Action {
    fn from(other: ActionProto) -> Action {
        match other.kind.unwrap() {
            // TODO: we could probably easily macro-ify this if all actions were required to have
            // exactly one tuple parameter. More complex actions can pass a struct as their
            // parameter.
            ActionKind::SetBpm(it) => Action::SetBpm(it),
            ActionKind::SetVolume(it) => Action::SetVolume(it),
            ActionKind::SetGeneratorVolume(it) => Action::SetGeneratorVolume(it),
            ActionKind::SetGeneratorMute(it) => Action::SetGeneratorMute(it),
        }
    }
}

impl From<Action> for ActionProto {
    fn from(other: Action) -> ActionProto {
        ActionProto {
            kind: Some(match other {
                Action::SetBpm(it) => ActionKind::SetBpm(it),
                Action::SetVolume(it) => ActionKind::SetVolume(it),
                Action::SetGeneratorVolume(it) => ActionKind::SetGeneratorVolume(it),
                Action::SetGeneratorMute(it) => ActionKind::SetGeneratorMute(it),
                _ => todo!(),
            }),
        }
    }
}
