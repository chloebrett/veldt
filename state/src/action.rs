use shared::action_proto::{ActionProto, action_proto::Kind as ActionKind};
use shared::model::{
    AdsrEnvelope, AntiAliasingMode, EffectInstance, EqType, PitchName, PlacedNote, Project, Sample,
    Scale, ScaleValue, TrackId, TrackPlacement, WaveType,
};
use shared::pmodel::{
    AntiAliasingModeProto, EqTypeProto, PitchNameProto, ScaleProto, WaveTypeProto,
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
    MoveEffectUp(usize),
    MoveEffectDown(usize),
    DeleteEffect(usize),
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
    SetTrackPlacementClippedDuration(Beats),
    RemoveTrackPlacementClippedDuration(bool),

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
            // TODO: macro-ify this.
            ActionKind::SetKey(it) => Action::SetKey(it.into()),
            ActionKind::SetScale(it) => Action::SetScale(ScaleProto::try_from(it).unwrap().into()),
            ActionKind::SetProjectName(it) => Action::SetProjectName(it),
            ActionKind::SetBpm(it) => Action::SetBpm(it),
            ActionKind::SetVolume(it) => Action::SetVolume(it),
            ActionKind::SetGeneratorVolume(it) => Action::SetGeneratorVolume(it),
            ActionKind::SetGeneratorMute(it) => Action::SetGeneratorMute(it),
            ActionKind::AddNote(it) => Action::AddNote(it.into()),
            ActionKind::DeleteNote(it) => Action::DeleteNote(it as usize),
            ActionKind::SetTrackOffset(it) => Action::SetTrackOffset(it),
            ActionKind::SetNoteScaleValue(it) => Action::SetNoteScaleValue(it.into()),
            ActionKind::SetNoteOffset(it) => Action::SetNoteOffset(it),
            ActionKind::SetNoteDuration(it) => Action::SetNoteDuration(it),
            ActionKind::SetGeneratorPan(it) => Action::SetGeneratorPan(it),
            ActionKind::SetWave(it) => Action::SetWave(WaveTypeProto::try_from(it).unwrap().into()),
            ActionKind::SetOscCount(it) => Action::SetOscCount(it),
            ActionKind::SetDetuneCents(it) => Action::SetDetuneCents(it),
            ActionKind::SetEnvelope(it) => Action::SetEnvelope(it.into()),
            ActionKind::SetAntiAliasingMode(it) => {
                Action::SetAntiAliasingMode(AntiAliasingModeProto::try_from(it).unwrap().into())
            }
            ActionKind::SetNoteOctave(it) => Action::SetNoteOctave(it),
            ActionKind::SetNotePitchName(it) => {
                Action::SetNotePitchName(PitchNameProto::try_from(it).unwrap().into())
            }
            ActionKind::SetOversampleFactor(it) => Action::SetOversampleFactor(it),
            ActionKind::MoveEffectUp(it) => Action::MoveEffectUp(it as usize),
            ActionKind::MoveEffectDown(it) => Action::MoveEffectDown(it as usize),
            ActionKind::DeleteEffect(it) => Action::DeleteEffect(it as usize),
            ActionKind::AddEffect(it) => Action::AddEffect(it.into()),
            ActionKind::SetDelayMs(it) => Action::SetDelayMs(it),
            ActionKind::SetEffectWet(it) => Action::SetEffectWet(it),
            ActionKind::SetEffectMute(it) => Action::SetEffectMute(it),
            ActionKind::SetEqFc(it) => Action::SetEqFc(it),
            ActionKind::SetEqQ(it) => Action::SetEqQ(it),
            ActionKind::SetEqGain(it) => Action::SetEqGain(it),
            ActionKind::SetCompressorThreshold(it) => Action::SetCompressorThreshold(it),
            ActionKind::SetCompressorAttackMs(it) => Action::SetCompressorAttackMs(it),
            ActionKind::SetCompressorReleaseMs(it) => Action::SetCompressorReleaseMs(it),
            ActionKind::SetCompressorRatio(it) => Action::SetCompressorRatio(it),
            ActionKind::SetCompressorGain(it) => Action::SetCompressorGain(it),
            ActionKind::SetModDelayMinDepth(it) => Action::SetModDelayMinDepth(it),
            ActionKind::SetModDelayMaxDepth(it) => Action::SetModDelayMaxDepth(it),
            ActionKind::SetModDelayLfoFreq(it) => Action::SetModDelayLfoFreq(it),
            ActionKind::SetModDelayLfoType(it) => {
                Action::SetModDelayLfoType(WaveTypeProto::try_from(it).unwrap().into())
            }
            ActionKind::SetTrackPlacementTrackId(it) => {
                Action::SetTrackPlacementTrackId(it as usize)
            }
            ActionKind::SetTrackPlacementOffset(it) => Action::SetTrackPlacementOffset(it),
            ActionKind::AddTrackPlacement(it) => Action::AddTrackPlacement(it.into()),
            ActionKind::DeleteTrackPlacement(it) => Action::DeleteTrackPlacement(it as usize),
            ActionKind::SetEqKind(it) => {
                Action::SetEqKind(EqTypeProto::try_from(it).unwrap().into())
            }
            ActionKind::SetTrackPlacementClippedDuration(it) => Action::SetTrackPlacementClippedDuration(it as Beats),
            ActionKind::RemoveTrackPlacementClippedDuration(it) => Action::RemoveTrackPlacementClippedDuration(it)
        }
    }
}

impl From<Action> for ActionProto {
    fn from(other: Action) -> ActionProto {
        ActionProto {
            kind: Some(match other {
                // TODO: macro-ify this.
                Action::SetKey(it) => ActionKind::SetKey(it.into()),
                Action::SetScale(it) => {
                    ActionKind::SetScale(ScaleProto::try_from(it).unwrap().into())
                }
                Action::SetProjectName(it) => ActionKind::SetProjectName(it),
                Action::SetBpm(it) => ActionKind::SetBpm(it),
                Action::SetVolume(it) => ActionKind::SetVolume(it),
                Action::SetGeneratorVolume(it) => ActionKind::SetGeneratorVolume(it),
                Action::SetGeneratorMute(it) => ActionKind::SetGeneratorMute(it),
                Action::AddNote(it) => ActionKind::AddNote(it.into()),
                Action::DeleteNote(it) => ActionKind::DeleteNote(it as u32),
                Action::SetTrackOffset(it) => ActionKind::SetTrackOffset(it),
                Action::SetNoteScaleValue(it) => ActionKind::SetNoteScaleValue(it.into()),
                Action::SetNoteOffset(it) => ActionKind::SetNoteOffset(it),
                Action::SetNoteDuration(it) => ActionKind::SetNoteDuration(it),
                Action::SetGeneratorPan(it) => ActionKind::SetGeneratorPan(it),
                Action::SetWave(it) => {
                    ActionKind::SetWave(WaveTypeProto::try_from(it).unwrap().into())
                }
                Action::SetOscCount(it) => ActionKind::SetOscCount(it),
                Action::SetDetuneCents(it) => ActionKind::SetDetuneCents(it),
                Action::SetEnvelope(it) => ActionKind::SetEnvelope(it.into()),
                Action::SetAntiAliasingMode(it) => ActionKind::SetAntiAliasingMode(
                    AntiAliasingModeProto::try_from(it).unwrap().into(),
                ),
                Action::SetNoteOctave(it) => ActionKind::SetNoteOctave(it),
                Action::SetNotePitchName(it) => {
                    ActionKind::SetNotePitchName(PitchNameProto::try_from(it).unwrap().into())
                }
                Action::SetOversampleFactor(it) => ActionKind::SetOversampleFactor(it),
                Action::MoveEffectUp(it) => ActionKind::MoveEffectUp(it as u32),
                Action::MoveEffectDown(it) => ActionKind::MoveEffectDown(it as u32),
                Action::DeleteEffect(it) => ActionKind::DeleteEffect(it as u32),
                Action::AddEffect(it) => ActionKind::AddEffect(it.into()),
                Action::SetDelayMs(it) => ActionKind::SetDelayMs(it),
                Action::SetEffectWet(it) => ActionKind::SetEffectWet(it),
                Action::SetEffectMute(it) => ActionKind::SetEffectMute(it),
                Action::SetEqFc(it) => ActionKind::SetEqFc(it),
                Action::SetEqQ(it) => ActionKind::SetEqQ(it),
                Action::SetEqGain(it) => ActionKind::SetEqGain(it),
                Action::SetCompressorThreshold(it) => ActionKind::SetCompressorThreshold(it),
                Action::SetCompressorAttackMs(it) => ActionKind::SetCompressorAttackMs(it),
                Action::SetCompressorReleaseMs(it) => ActionKind::SetCompressorReleaseMs(it),
                Action::SetCompressorRatio(it) => ActionKind::SetCompressorRatio(it),
                Action::SetCompressorGain(it) => ActionKind::SetCompressorGain(it),
                Action::SetModDelayMinDepth(it) => ActionKind::SetModDelayMinDepth(it),
                Action::SetModDelayMaxDepth(it) => ActionKind::SetModDelayMaxDepth(it),
                Action::SetModDelayLfoFreq(it) => ActionKind::SetModDelayLfoFreq(it),
                Action::SetModDelayLfoType(it) => {
                    ActionKind::SetModDelayLfoType(WaveTypeProto::try_from(it).unwrap().into())
                }
                Action::SetTrackPlacementTrackId(it) => {
                    ActionKind::SetTrackPlacementTrackId(it as u32)
                }
                Action::SetTrackPlacementOffset(it) => ActionKind::SetTrackPlacementOffset(it),
                Action::SetTrackPlacementClippedDuration(it) => ActionKind::SetTrackPlacementClippedDuration(it),
                Action::AddTrackPlacement(it) => ActionKind::AddTrackPlacement(it.into()),
                Action::DeleteTrackPlacement(it) => ActionKind::DeleteTrackPlacement(it as u32),
                Action::SetEqKind(it) => {
                    ActionKind::SetEqKind(EqTypeProto::try_from(it).unwrap().into())
                }
                Action::RemoveTrackPlacementClippedDuration(it) => ActionKind::RemoveTrackPlacementClippedDuration(it),


                // Non-serializable actions
                Action::Release => panic!(),
                Action::NonReversible => panic!(),
                Action::SetProjectList(_) => panic!(),
                Action::SetProject(_) => panic!(),
                Action::SetLoadProjectName(_) => panic!(),
                Action::AddSample(_) => panic!(),
            }),
        }
    }
}
