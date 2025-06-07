use shared::action_proto::{TypeFieldProto, type_field_proto::Kind as TypeFieldKind};
use shared::model::{
    AdsrEnvelope, AntiAliasingMode, EffectInstance, EqType, FileTreeConfig, FilenameTree,
    GeneratorId, GeneratorInstance, MixerChannel, NoiseType, PitchName, PlacedNote, Placement,
    PolyphonyMode, Project, Sample, Scale, ScaleValue, Track, WaveType,
};
use shared::pmodel::{
    AntiAliasingModeProto, EqTypeProto, NoiseTypeProto, PolyphonyModeProto, ScaleProto,
    WaveTypeProto,
};

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
    Placement(Placement),
    Project(Project),
    ProjectName(String),
    ProjectList(Vec<String>),
    LoadProjectName(String),
    Sample(Sample),
    SampleTree(FilenameTree),
    Track(Track),
    PlacedNote(PlacedNote),
    Wave(WaveType),
    Envelope(AdsrEnvelope),
    AntiAliasingMode(AntiAliasingMode),
    EqType(EqType),
    ClippedDuration(Option<f32>),
    SampleTreeConfig(FileTreeConfig),
    Generator(GeneratorInstance),
    MixerChannel(MixerChannel),
    PolyphonyMode(PolyphonyMode),
    NoiseType(NoiseType),

    // ID types.
    GeneratorId(GeneratorId),
    PlacementId(PlacementId),

    // Note: if we end up with more bools/primitives, make dedicated types for them so that we
    // don't have to keep expanding the proto.
    Mute(bool),
    Octave(i32),
    // Note: when you add a new type, make sure to configure its broadcast behaviour in broadcast.rs as well.
}

impl From<TypeFieldProto> for TypeField {
    fn from(other: TypeFieldProto) -> Self {
        match other.kind.unwrap() {
            TypeFieldKind::Effect(it) => TypeField::Effect(it.into()),
            TypeFieldKind::PitchName(it) => TypeField::PitchName(it.into()),
            TypeFieldKind::Key(it) => TypeField::Key(it.into()),
            TypeFieldKind::ScaleValue(it) => TypeField::ScaleValue(it.into()),
            TypeFieldKind::Scale(it) => TypeField::Scale(ScaleProto::try_from(it).unwrap().into()),
            TypeFieldKind::Placement(it) => TypeField::Placement(it.into()),
            TypeFieldKind::ProjectName(it) => TypeField::ProjectName(it),
            TypeFieldKind::Track(it) => TypeField::Track(it.into()),
            TypeFieldKind::PlacedNote(it) => TypeField::PlacedNote(it.into()),
            TypeFieldKind::Wave(it) => TypeField::Wave(WaveTypeProto::try_from(it).unwrap().into()),
            TypeFieldKind::Envelope(it) => TypeField::Envelope(it.into()),
            TypeFieldKind::AntiAliasingMode(it) => {
                TypeField::AntiAliasingMode(AntiAliasingModeProto::try_from(it).unwrap().into())
            }
            TypeFieldKind::EqType(it) => {
                TypeField::EqType(EqTypeProto::try_from(it).unwrap().into())
            }
            TypeFieldKind::ClippedDuration(it) => TypeField::ClippedDuration(it.into()),
            TypeFieldKind::SampleTree(it) => TypeField::SampleTree(it.into()),
            TypeFieldKind::SampleTreeConfig(it) => TypeField::SampleTreeConfig(it.into()),
            TypeFieldKind::Generator(it) => TypeField::Generator(it.into()),
            TypeFieldKind::MixerChannel(it) => TypeField::MixerChannel(it.into()),
            TypeFieldKind::PolyphonyMode(it) => {
                TypeField::PolyphonyMode(PolyphonyModeProto::try_from(it).unwrap().into())
            }
            // Note: if we end up with more bools/primitives, make dedicated types for them so that we
            // don't have to keep expanding the proto.
            TypeFieldKind::Mute(it) => TypeField::Mute(it),
            TypeFieldKind::Octave(it) => TypeField::Octave(it),
            TypeFieldKind::NoiseType(it) => {
                TypeField::NoiseType(NoiseTypeProto::try_from(it).unwrap().into())
            }
            TypeFieldKind::GeneratorId(it) => TypeField::GeneratorId(it.into()),
            TypeFieldKind::PlacementId(it) => TypeField::PlacementId(it.into()),
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

                TypeField::SampleTree(it) => TypeFieldKind::SampleTree(it.into()),
                TypeField::Effect(it) => TypeFieldKind::Effect(it.into()),
                TypeField::PitchName(it) => TypeFieldKind::PitchName(it.into()),
                TypeField::Key(it) => TypeFieldKind::Key(it.into()),
                TypeField::ScaleValue(it) => TypeFieldKind::ScaleValue(it.into()),
                TypeField::Scale(it) => TypeFieldKind::Scale(ScaleProto::from(it).into()),
                TypeField::Placement(it) => TypeFieldKind::Placement(it.into()),
                TypeField::ProjectName(it) => TypeFieldKind::ProjectName(it),
                TypeField::Track(it) => TypeFieldKind::Track(it.into()),
                TypeField::PlacedNote(it) => TypeFieldKind::PlacedNote(it.into()),
                TypeField::Wave(it) => TypeFieldKind::Wave(WaveTypeProto::from(it).into()),
                TypeField::Envelope(it) => TypeFieldKind::Envelope(it.into()),
                TypeField::AntiAliasingMode(it) => {
                    TypeFieldKind::AntiAliasingMode(AntiAliasingModeProto::from(it).into())
                }
                TypeField::EqType(it) => TypeFieldKind::EqType(EqTypeProto::from(it).into()),
                TypeField::ClippedDuration(it) => TypeFieldKind::ClippedDuration(it.into()),
                TypeField::SampleTreeConfig(it) => TypeFieldKind::SampleTreeConfig(it.into()),
                TypeField::Generator(it) => TypeFieldKind::Generator(it.into()),
                TypeField::MixerChannel(it) => TypeFieldKind::MixerChannel(it.into()),
                TypeField::PolyphonyMode(it) => {
                    TypeFieldKind::PolyphonyMode(PolyphonyModeProto::from(it).into())
                }

                // Note: if we end up with more bools/primitives, make dedicated types for them so that we
                // don't have to keep expanding the proto.
                TypeField::Mute(it) => TypeFieldKind::Mute(it),
                TypeField::Octave(it) => TypeFieldKind::Octave(it),
                TypeField::NoiseType(it) => {
                    TypeFieldKind::NoiseType(NoiseTypeProto::from(it).into())
                }
                TypeField::GeneratorId(it) => TypeFieldKind::GeneratorId(it.into()),
                TypeField::PlacementId(it) => TypeFieldKind::PlacementId(it.into()),
            }),
        }
    }
}
