use shared::action_proto::{
    IndexFieldProto, TypeFieldProto, index_field_proto::Kind as IndexFieldKind,
    type_field_proto::Kind as TypeFieldKind,
};
use shared::model::{
    AdsrEnvelope, AntiAliasingMode, EffectInstance, EqType, PitchName, PlacedNote, Project, Sample,
    Scale, ScaleValue, Track, TrackPlacement, WaveType,
};
use shared::pmodel::{
    AdsrEnvelopeProto, AntiAliasingModeProto, EqTypeProto,
    ScaleProto, WaveTypeProto,
};
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
    Effect(EffectInstance),
    PitchName(PitchName),
    Key(ScaleValue),
    ScaleValue(ScaleValue),
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
    ClippedDuration(Option<f32>),

    // Note: if we end up with more bools/primitives, make dedicated types for them so that we
    // don't have to keep expanding the proto.
    Mute(bool),
    Octave(i32),
}

impl From<TypeFieldProto> for TypeField {
    fn from(other: TypeFieldProto) -> Self {
        match other.kind.unwrap() {
            TypeFieldKind::Effect(it) => TypeField::Effect(it.into()),
            TypeFieldKind::PitchName(it) => TypeField::PitchName(it.into()),
            TypeFieldKind::Key(it) => TypeField::Key(it.into()),
            TypeFieldKind::ScaleValue(it) => {
                TypeField::ScaleValue(ScaleValue::try_from(it).unwrap().into())
            }
            TypeFieldKind::Scale(it) => TypeField::Scale(ScaleProto::try_from(it).unwrap().into()),
            TypeFieldKind::TrackPlacement(it) => TypeField::TrackPlacement(it.into()),
            TypeFieldKind::ProjectName(it) => TypeField::ProjectName(it),
            TypeFieldKind::Track(it) => TypeField::Track(it.into()),
            TypeFieldKind::PlacedNote(it) => TypeField::PlacedNote(it.into()),
            TypeFieldKind::Wave(it) => TypeField::Wave(WaveTypeProto::try_from(it).unwrap().into()),
            TypeFieldKind::Envelope(it) => TypeField::Envelope(AdsrEnvelopeProto::from(it).into()),
            TypeFieldKind::AntiAliasingMode(it) => {
                TypeField::AntiAliasingMode(AntiAliasingModeProto::try_from(it).unwrap().into())
            }
            TypeFieldKind::EqType(it) => {
                TypeField::EqType(EqTypeProto::try_from(it).unwrap().into())
            }
            TypeFieldKind::ClippedDuration(it) => TypeField::ClippedDuration(it.into()),

            // Note: if we end up with more bools/primitives, make dedicated types for them so that we
            // don't have to keep expanding the proto.
            TypeFieldKind::Mute(it) => TypeField::Mute(it),
            TypeFieldKind::Octave(it) => TypeField::Octave(it),
        }
    }
}

impl From<TypeField> for TypeFieldProto {
    fn from(other: TypeField) -> Self {
        TypeFieldProto {
            kind: Some(match other {
                // Some types are never transmitted.
                TypeField::Project(_) => panic!(),
                TypeField::LoadProjectName(_) => panic!(),
                TypeField::ProjectList(_) => panic!(),
                TypeField::Sample(_) => panic!(),

                TypeField::Effect(it) => TypeFieldKind::Effect(it.into()),
                TypeField::PitchName(it) => TypeFieldKind::PitchName(it.into()),
                TypeField::Key(it) => TypeFieldKind::Key(it.into()),
                TypeField::ScaleValue(it) => TypeFieldKind::ScaleValue(it.into()),
                TypeField::Scale(it) => TypeFieldKind::Scale(ScaleProto::from(it).into()),
                TypeField::TrackPlacement(it) => TypeFieldKind::TrackPlacement(it.into()),
                TypeField::ProjectName(it) => TypeFieldKind::ProjectName(it),
                TypeField::Track(it) => TypeFieldKind::Track(it.into()),
                TypeField::PlacedNote(it) => TypeFieldKind::PlacedNote(it.into()),
                TypeField::Wave(it) => TypeFieldKind::Wave(WaveTypeProto::from(it).into()),
                TypeField::Envelope(it) => {
                    TypeFieldKind::Envelope(AdsrEnvelopeProto::from(it).into())
                }
                TypeField::AntiAliasingMode(it) => {
                    TypeFieldKind::AntiAliasingMode(AntiAliasingModeProto::from(it).into())
                }
                TypeField::EqType(it) => TypeFieldKind::EqType(EqTypeProto::from(it).into()),
                TypeField::ClippedDuration(it) => TypeFieldKind::ClippedDuration(it.into()),

                // Note: if we end up with more bools/primitives, make dedicated types for them so that we
                // don't have to keep expanding the proto.
                TypeField::Mute(it) => TypeFieldKind::Mute(it),
                TypeField::Octave(it) => TypeFieldKind::Octave(it),
            }),
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
        match other.kind.unwrap() {
            IndexFieldKind::Track(it) => IndexField::Track(it as usize),
            IndexFieldKind::PlacedNote(it) => IndexField::PlacedNote(it as usize),
            IndexFieldKind::TrackPlacement(it) => IndexField::TrackPlacement(it as usize),
            IndexFieldKind::Effect(it) => IndexField::Effect(it as usize),
            IndexFieldKind::Generator(it) => IndexField::Generator(it as usize),
        }
    }
}

impl From<IndexField> for IndexFieldProto {
    fn from(other: IndexField) -> Self {
        IndexFieldProto {
            kind: Some(match other {
                IndexField::Track(it) => IndexFieldKind::Track(it as u32),
                IndexField::PlacedNote(it) => IndexFieldKind::PlacedNote(it as u32),
                IndexField::TrackPlacement(it) => IndexFieldKind::TrackPlacement(it as u32),
                IndexField::Effect(it) => IndexFieldKind::Effect(it as u32),
                IndexField::Generator(it) => IndexFieldKind::Generator(it as u32),
            }),
        }
    }
}
