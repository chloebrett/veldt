use shared::action_proto::{ActionProto, SetFloatProto, action_proto::Kind as ActionKind};
use shared::model::{
    AdsrEnvelope, AntiAliasingMode, EffectInstance, EqType, PitchName, PlacedNote, Project, Sample,
    Scale, ScaleValue, Track, TrackId, TrackPlacement, WaveType,
};
use shared::pmodel::{
    AntiAliasingModeProto, EqTypeProto, PitchNameProto, ScaleProto, WaveTypeProto,
};
use shared::types::Octave;
use std::str::FromStr;
use strum::{Display, EnumString};

#[derive(EnumString, Display, PartialEq, Clone, Debug)]
pub enum FloatField {
    Bpm,
    Volume,
    Offset,
    Duration,
    Pan,
    Detune,
    DelayMs,
    Wet,
    Fc,
    Q,
    Gain,
    Threshold,
    AttackMs,
    ReleaseMs,
    Ratio,
    LfoFreq,
    Feedback,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // --- RootSelector ---
    SetKey(ScaleValue),
    SetScale(Scale),
    SetProjectName(String),
    AddTrackPlacement(TrackPlacement),
    DeleteTrackPlacement(usize),
    // Sets the names of loadable projects.
    SetProjectList(Vec<String>),
    // Overwrites the whole project.
    SetProject(Project),
    SetLoadProjectName(String),
    AddSample(Sample),
    AddTrack(Track),
    DeleteTrack(usize),

    // --- TrackSelector ---
    DeleteNote(usize),
    AddNote(PlacedNote),

    // --- NoteSelector ---
    SetNoteScaleValue(ScaleValue),
    SetNoteOctave(Octave),
    SetNotePitchName(PitchName),

    // --- GeneratorSelector ---
    // TODO: rename this to SetVolume, and just differentiate by the selector. (Apply this idea to
    // several action types).
    SetWave(WaveType),
    SetOscCount(u32),
    SetEnvelope(AdsrEnvelope),
    SetAntiAliasingMode(AntiAliasingMode),
    SetOversampleFactor(u32),

    // --- MixerSelector ---
    MoveEffectUp(usize),
    MoveEffectDown(usize),
    DeleteEffect(usize),
    AddEffect(EffectInstance),

    // --- EffectSelector ---
    SetEqKind(EqType),
    SetModDelayMinDepth(u32),
    SetModDelayMaxDepth(u32),
    SetModDelayLfoType(WaveType),

    // --- TrackPlacementSelector ---
    SetTrackPlacementTrackId(TrackId),
    SetTrackPlacementClippedDuration(Option<f32>),

    // --- used by several selectors ---
    SetMute(bool),
    SetFloat(FloatField, f32),

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
            ActionKind::SetMute(it) => Action::SetMute(it),
            ActionKind::AddNote(it) => Action::AddNote(it.into()),
            ActionKind::DeleteNote(it) => Action::DeleteNote(it as usize),
            ActionKind::SetNoteScaleValue(it) => Action::SetNoteScaleValue(it.into()),
            ActionKind::SetWave(it) => Action::SetWave(WaveTypeProto::try_from(it).unwrap().into()),
            ActionKind::SetOscCount(it) => Action::SetOscCount(it),
            ActionKind::SetEnvelope(it) => Action::SetEnvelope(it.into()),
            ActionKind::SetAntiAliasingMode(it) => {
                Action::SetAntiAliasingMode(AntiAliasingModeProto::try_from(it).unwrap().into())
            }
            ActionKind::SetNoteOctave(it) => Action::SetNoteOctave(it),
            ActionKind::SetNotePitchName(it) => Action::SetNotePitchName(it.into()),
            ActionKind::SetOversampleFactor(it) => Action::SetOversampleFactor(it),
            ActionKind::MoveEffectUp(it) => Action::MoveEffectUp(it as usize),
            ActionKind::MoveEffectDown(it) => Action::MoveEffectDown(it as usize),
            ActionKind::DeleteEffect(it) => Action::DeleteEffect(it as usize),
            ActionKind::AddEffect(it) => Action::AddEffect(it.into()),
            ActionKind::SetModDelayMinDepth(it) => Action::SetModDelayMinDepth(it),
            ActionKind::SetModDelayMaxDepth(it) => Action::SetModDelayMaxDepth(it),
            ActionKind::SetModDelayLfoType(it) => {
                Action::SetModDelayLfoType(WaveTypeProto::try_from(it).unwrap().into())
            }
            ActionKind::SetTrackPlacementTrackId(it) => {
                Action::SetTrackPlacementTrackId(it as usize)
            }
            ActionKind::AddTrackPlacement(it) => Action::AddTrackPlacement(it.into()),
            ActionKind::DeleteTrackPlacement(it) => Action::DeleteTrackPlacement(it as usize),
            ActionKind::SetEqKind(it) => {
                Action::SetEqKind(EqTypeProto::try_from(it).unwrap().into())
            }
            ActionKind::SetTrackPlacementClippedDuration(it) => {
                Action::SetTrackPlacementClippedDuration(it.value)
            }
            ActionKind::AddTrack(it) => Action::AddTrack(it.into()),
            ActionKind::DeleteTrack(it) => Action::DeleteTrack(it as usize),
            ActionKind::SetFloat(it) => Action::SetFloat(
                FloatField::from_str(&it.key)
                    .expect(&format!("Expected float field name: {}", it.key)),
                it.value,
            ),
        }
    }
}

impl From<Action> for ActionProto {
    fn from(other: Action) -> ActionProto {
        ActionProto {
            kind: Some(match other {
                // TODO: macro-ify this.
                // But also, we could just make the whole app more modular instead of having
                // centralised actions like this.
                Action::SetKey(it) => ActionKind::SetKey(it.into()),
                Action::SetScale(it) => ActionKind::SetScale(ScaleProto::from(it).into()),
                Action::SetProjectName(it) => ActionKind::SetProjectName(it),
                Action::SetMute(it) => ActionKind::SetMute(it),
                Action::AddNote(it) => ActionKind::AddNote(it.into()),
                Action::DeleteNote(it) => ActionKind::DeleteNote(it as u32),
                Action::SetNoteScaleValue(it) => ActionKind::SetNoteScaleValue(it.into()),
                Action::SetWave(it) => ActionKind::SetWave(WaveTypeProto::from(it).into()),
                Action::SetOscCount(it) => ActionKind::SetOscCount(it),
                Action::SetEnvelope(it) => ActionKind::SetEnvelope(it.into()),
                Action::SetAntiAliasingMode(it) => {
                    ActionKind::SetAntiAliasingMode(AntiAliasingModeProto::from(it).into())
                }
                Action::SetNoteOctave(it) => ActionKind::SetNoteOctave(it),
                Action::SetNotePitchName(it) => {
                    ActionKind::SetNotePitchName(PitchNameProto::from(it))
                }
                Action::SetOversampleFactor(it) => ActionKind::SetOversampleFactor(it),
                Action::MoveEffectUp(it) => ActionKind::MoveEffectUp(it as u32),
                Action::MoveEffectDown(it) => ActionKind::MoveEffectDown(it as u32),
                Action::DeleteEffect(it) => ActionKind::DeleteEffect(it as u32),
                Action::AddEffect(it) => ActionKind::AddEffect(it.into()),
                Action::SetModDelayMinDepth(it) => ActionKind::SetModDelayMinDepth(it),
                Action::SetModDelayMaxDepth(it) => ActionKind::SetModDelayMaxDepth(it),
                Action::SetModDelayLfoType(it) => {
                    ActionKind::SetModDelayLfoType(WaveTypeProto::from(it).into())
                }
                Action::SetTrackPlacementTrackId(it) => {
                    ActionKind::SetTrackPlacementTrackId(it as u32)
                }
                Action::SetTrackPlacementClippedDuration(it) => {
                    ActionKind::SetTrackPlacementClippedDuration(it.into())
                }
                Action::AddTrackPlacement(it) => ActionKind::AddTrackPlacement(it.into()),
                Action::DeleteTrackPlacement(it) => ActionKind::DeleteTrackPlacement(it as u32),
                Action::SetEqKind(it) => ActionKind::SetEqKind(EqTypeProto::from(it).into()),
                Action::AddTrack(it) => ActionKind::AddTrack(it.into()),
                Action::DeleteTrack(it) => ActionKind::DeleteTrack(it as u32),
                Action::SetFloat(key, value) => ActionKind::SetFloat(SetFloatProto {
                    key: key.to_string(),
                    value,
                }),

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
