use shared::action_proto::{
    ActionProto, IndexFieldProto, SetFloatProto, SetUintProto, TypeFieldProto,
    action_proto::Kind as ActionKind,
};
use shared::model::{
    AdsrEnvelope, AntiAliasingMode, EffectInstance, EqType, PitchName, PlacedNote, Project,
    Sample, Scale, ScaleValue, Track, TrackPlacement, WaveType,
};
use std::str::FromStr;
use strum::{Display, EnumString};

/// Fields of type f32.
/// Used to distinguish *which* field of this type is being referred to.
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

/// Fields of type u32.
/// Used to distinguish *which* field of this type is being referred to.
#[derive(EnumString, Display, PartialEq, Clone, Debug)]
pub enum UintField {
    OscCount,
    OversampleFactor,
    MinDepth,
    MaxDepth,
    TrackId,
}

/// Fields of various types.
/// Used to distinguish *which* field of this type is being referred to,
/// and also contains the appropriate value.
/// TODO: investigate size cost of these, and consider boxing large types e.g. Project.
#[derive(PartialEq, Clone, Debug)]
pub enum TypeField {
    ScaleValue(ScaleValue),
    Effect(EffectInstance),
    PitchName(PitchName),
    Key(ScaleValue),
    Scale(Scale),
    TrackPlacement(TrackPlacement),
    Project(Project),
    ProjectName(String),
    ProjectList(Vec<String>),
    LoadProjectName(String),
    Sample(Sample),
    Track(Track),
    PlacedNote(PlacedNote),
    Wave(WaveType),
    Envelope(AdsrEnvelope),
    AntiAliasingMode(AntiAliasingMode),
    EqType(EqType),
    EffectInstance(EffectInstance),
    ClippedDuration(Option<f32>),

    // Note: if we end up with more bools/primitives, make dedicated types for them so that we
    // don't have to keep expanding the proto.
    Mute(bool),
    Octave(i32),
}

impl From<TypeFieldProto> for TypeField {
    fn from(other: TypeFieldProto) -> Self {
        match other.kind {
            _ => panic!(),
        }
    }
}

impl From<TypeField> for TypeFieldProto {
    fn from(other: TypeField) -> Self {
        match other {
            TypeField::Project(_) => panic!(),
            TypeField::ProjectList(_) => panic!(),
            TypeField::Sample(_) => panic!(),
            _ => panic!(),
        }
    }
}

/// Fields that index into a list.
/// Used to distinguish *which* index is being referred to, and also contains the index value.
#[derive(PartialEq, Clone, Debug)]
pub enum IndexField {
    Track(usize),
    PlacedNote(usize),
    TrackPlacement(usize),
    Effect(usize),
    Generator(usize),
}

impl From<IndexFieldProto> for IndexField {
    fn from(other: IndexFieldProto) -> Self {
        match other.kind {
            _ => panic!(),
        }
    }
}

impl From<IndexField> for IndexFieldProto {
    fn from(other: IndexField) -> Self {
        match other {
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // --- MixerSelector ---
    MoveEffectUp(usize),
    MoveEffectDown(usize),

    // --- used by several selectors ---
    SetFloat(FloatField, f32),
    SetUint(UintField, u32),
    DeleteChild(IndexField),
    SetChild(TypeField),
    AddChild(TypeField),

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
            ActionKind::MoveEffectUp(it) => Action::MoveEffectUp(it as usize),
            ActionKind::MoveEffectDown(it) => Action::MoveEffectDown(it as usize),
            ActionKind::SetFloat(it) => Action::SetFloat(
                FloatField::from_str(&it.key)
                    .expect(&format!("Expected float field name: {}", it.key)),
                it.value,
            ),
            ActionKind::SetUint(it) => Action::SetUint(
                UintField::from_str(&it.key)
                    .expect(&format!("Expected uint field name: {}", it.key)),
                it.value,
            ),
            ActionKind::DeleteChild(index) => Action::DeleteChild(index.into()),
            ActionKind::AddChild(child) => Action::AddChild(child.into()),
            ActionKind::SetChild(child) => Action::SetChild(child.into()),
        }
    }
}

impl From<Action> for ActionProto {
    fn from(other: Action) -> ActionProto {
        ActionProto {
            kind: Some(match other {
                Action::SetFloat(key, value) => ActionKind::SetFloat(SetFloatProto {
                    key: key.to_string(),
                    value,
                }),
                Action::SetUint(key, value) => ActionKind::SetUint(SetUintProto {
                    key: key.to_string(),
                    value,
                }),
                Action::SetChild(child) => ActionKind::SetChild(child.into()),
                Action::AddChild(child) => ActionKind::AddChild(child.into()),
                Action::DeleteChild(index) => ActionKind::DeleteChild(index.into()),
                Action::MoveEffectUp(index) => ActionKind::MoveEffectUp(index as u32),
                Action::MoveEffectDown(index) => ActionKind::MoveEffectDown(index as u32),

                // Non-serializable actions
                Action::Release => panic!(),
                Action::NonReversible => panic!(),
            }),
        }
    }
}
