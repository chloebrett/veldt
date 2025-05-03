use shared::action_proto::{
    MultiTypeFieldProto, TypeFieldKind, TypeFieldProto, type_field_proto::Kind as ProtoKind,
};
use shared::model::{
    AdsrEnvelope, AntiAliasingMode, EffectInstance, EqType, FileTreeConfig, FilenameTree,
    GeneratorInstance, PitchName, PlacedNote, Placement, Project, Sample, Scale, ScaleValue, Track,
    WaveType,
};
use shared::pmodel::{AntiAliasingModeProto, EqTypeProto, ScaleProto, WaveTypeProto};
use strum::{Display, EnumString};

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

    // Note: if we end up with more bools/primitives, make dedicated types for them so that we
    // don't have to keep expanding the proto.
    Mute(bool),
    Octave(i32),
    // Note: when you add a new type, make sure to configure its broadcast behaviour in broadcast.rs as well.
}

impl From<TypeFieldProto> for TypeField {
    fn from(other: TypeFieldProto) -> Self {
        match other.kind.unwrap() {
            ProtoKind::Effect(it) => TypeField::Effect(it.into()),
            ProtoKind::PitchName(it) => TypeField::PitchName(it.into()),
            ProtoKind::Key(it) => TypeField::Key(it.into()),
            ProtoKind::ScaleValue(it) => TypeField::ScaleValue(it.into()),
            ProtoKind::Scale(it) => TypeField::Scale(ScaleProto::try_from(it).unwrap().into()),
            ProtoKind::Placement(it) => TypeField::Placement(it.into()),
            ProtoKind::ProjectName(it) => TypeField::ProjectName(it),
            ProtoKind::Track(it) => TypeField::Track(it.into()),
            ProtoKind::PlacedNote(it) => TypeField::PlacedNote(it.into()),
            ProtoKind::Wave(it) => TypeField::Wave(WaveTypeProto::try_from(it).unwrap().into()),
            ProtoKind::Envelope(it) => TypeField::Envelope(it.into()),
            ProtoKind::AntiAliasingMode(it) => {
                TypeField::AntiAliasingMode(AntiAliasingModeProto::try_from(it).unwrap().into())
            }
            ProtoKind::EqType(it) => TypeField::EqType(EqTypeProto::try_from(it).unwrap().into()),
            ProtoKind::ClippedDuration(it) => TypeField::ClippedDuration(it.into()),
            ProtoKind::SampleTree(it) => TypeField::SampleTree(it.into()),
            ProtoKind::SampleTreeConfig(it) => TypeField::SampleTreeConfig(it.into()),
            ProtoKind::Generator(it) => TypeField::Generator(it.into()),
            // Note: if we end up with more bools/primitives, make dedicated types for them so that we
            // don't have to keep expanding the proto.
            ProtoKind::Mute(it) => TypeField::Mute(it),
            ProtoKind::Octave(it) => TypeField::Octave(it),
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

                TypeField::SampleTree(it) => ProtoKind::SampleTree(it.into()),
                TypeField::Effect(it) => ProtoKind::Effect(it.into()),
                TypeField::PitchName(it) => ProtoKind::PitchName(it.into()),
                TypeField::Key(it) => ProtoKind::Key(it.into()),
                TypeField::ScaleValue(it) => ProtoKind::ScaleValue(it.into()),
                TypeField::Scale(it) => ProtoKind::Scale(ScaleProto::from(it).into()),
                TypeField::Placement(it) => ProtoKind::Placement(it.into()),
                TypeField::ProjectName(it) => ProtoKind::ProjectName(it),
                TypeField::Track(it) => ProtoKind::Track(it.into()),
                TypeField::PlacedNote(it) => ProtoKind::PlacedNote(it.into()),
                TypeField::Wave(it) => ProtoKind::Wave(WaveTypeProto::from(it).into()),
                TypeField::Envelope(it) => ProtoKind::Envelope(it.into()),
                TypeField::AntiAliasingMode(it) => {
                    ProtoKind::AntiAliasingMode(AntiAliasingModeProto::from(it).into())
                }
                TypeField::EqType(it) => ProtoKind::EqType(EqTypeProto::from(it).into()),
                TypeField::ClippedDuration(it) => ProtoKind::ClippedDuration(it.into()),
                TypeField::SampleTreeConfig(it) => ProtoKind::SampleTreeConfig(it.into()),
                TypeField::Generator(it) => ProtoKind::Generator(it.into()),

                // Note: if we end up with more bools/primitives, make dedicated types for them so that we
                // don't have to keep expanding the proto.
                TypeField::Mute(it) => ProtoKind::Mute(it),
                TypeField::Octave(it) => ProtoKind::Octave(it),
            }),
        }
    }
}

/// Extension of TypeField for working with multiple values.
#[derive(EnumString, Display, PartialEq, Clone, Debug)]
pub enum MultiTypeField {
    Track(Vec<Track>),
    PlacedNote(Vec<PlacedNote>),
    Placement(Vec<Placement>),
    Effect(Vec<EffectInstance>),
    Generator(Vec<GeneratorInstance>),
}

impl From<MultiTypeFieldProto> for MultiTypeField {
    fn from(object: MultiTypeFieldProto) -> Self {
        match object.kind() {
            TypeFieldKind::UnknownTypeFieldKind => panic!(),
            TypeFieldKind::TrackTypeFieldKind => MultiTypeField::Track(
                object
                    .values
                    .iter()
                    .map(|value| {
                        if let Some(ProtoKind::Track(track_proto)) = &value.kind {
                            track_proto.clone().into()
                        } else {
                            panic!()
                        }
                    })
                    .collect(),
            ),
            TypeFieldKind::PlacedNoteTypeFieldKind => MultiTypeField::PlacedNote(
                object
                    .values
                    .iter()
                    .map(|value| {
                        if let Some(ProtoKind::PlacedNote(note)) = value.kind {
                            note.into()
                        } else {
                            panic!()
                        }
                    })
                    .collect(),
            ),
            TypeFieldKind::PlacementTypeFieldKind => MultiTypeField::Placement(
                object
                    .values
                    .iter()
                    .map(|value| {
                        if let Some(ProtoKind::Placement(placement)) = value.kind {
                            placement.into()
                        } else {
                            panic!()
                        }
                    })
                    .collect(),
            ),
            TypeFieldKind::EffectTypeFieldKind => MultiTypeField::Effect(
                object
                    .values
                    .iter()
                    .map(|value| {
                        if let Some(ProtoKind::Effect(effect)) = value.kind {
                            effect.into()
                        } else {
                            panic!()
                        }
                    })
                    .collect(),
            ),
            TypeFieldKind::GeneratorTypeFieldKind => MultiTypeField::Generator(
                object
                    .values
                    .iter()
                    .map(|value| {
                        if let Some(ProtoKind::Generator(generator)) = &value.kind {
                            generator.clone().into()
                        } else {
                            panic!()
                        }
                    })
                    .collect(),
            ),
        }
    }
}

impl From<MultiTypeField> for MultiTypeFieldProto {
    fn from(object: MultiTypeField) -> Self {
        match object {
            MultiTypeField::Track(tracks) => Self {
                kind: TypeFieldKind::TrackTypeFieldKind.into(),
                values: tracks
                    .into_iter()
                    .map(|value| TypeFieldProto {
                        kind: Some(ProtoKind::Track(value.into())),
                    })
                    .collect(),
            },
            MultiTypeField::PlacedNote(notes) => Self {
                kind: TypeFieldKind::PlacedNoteTypeFieldKind.into(),
                values: notes
                    .into_iter()
                    .map(|value| TypeFieldProto {
                        kind: Some(ProtoKind::PlacedNote(value.into())),
                    })
                    .collect(),
            },
            MultiTypeField::Placement(placements) => Self {
                kind: TypeFieldKind::PlacementTypeFieldKind.into(),
                values: placements
                    .into_iter()
                    .map(|value| TypeFieldProto {
                        kind: Some(ProtoKind::Placement(value.into())),
                    })
                    .collect(),
            },
            MultiTypeField::Effect(effects) => Self {
                kind: TypeFieldKind::EffectTypeFieldKind.into(),
                values: effects
                    .into_iter()
                    .map(|value| TypeFieldProto {
                        kind: Some(ProtoKind::Effect(value.into())),
                    })
                    .collect(),
            },
            MultiTypeField::Generator(generators) => Self {
                kind: TypeFieldKind::GeneratorTypeFieldKind.into(),
                values: generators
                    .into_iter()
                    .map(|value| TypeFieldProto {
                        kind: Some(ProtoKind::Generator(value.into())),
                    })
                    .collect(),
            },
        }
    }
}
