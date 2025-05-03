use shared::action_proto::{
    IndexFieldKind, IndexFieldProto, MultiIndexFieldProto, index_field_proto::Kind as ProtoKind,
};
use strum::{Display, EnumString};

/// Fields that index into a list.
/// Used to distinguish *which* index is being referred to, and also contains the index value.
#[derive(EnumString, Display, PartialEq, Clone, Debug)]
pub enum IndexField {
    Track(usize),
    PlacedNote(usize),
    Placement(usize),
    Effect(usize),
    Generator(usize),
    Sample(usize),
}

impl From<IndexFieldProto> for IndexField {
    fn from(other: IndexFieldProto) -> Self {
        match other.kind.unwrap() {
            ProtoKind::Track(it) => IndexField::Track(it as usize),
            ProtoKind::PlacedNote(it) => IndexField::PlacedNote(it as usize),
            ProtoKind::Placement(it) => IndexField::Placement(it as usize),
            ProtoKind::Effect(it) => IndexField::Effect(it as usize),
            ProtoKind::Generator(it) => IndexField::Generator(it as usize),
            ProtoKind::Sample(it) => IndexField::Sample(it as usize),
        }
    }
}

impl From<IndexField> for IndexFieldProto {
    fn from(other: IndexField) -> Self {
        IndexFieldProto {
            kind: Some(match other {
                IndexField::Track(it) => ProtoKind::Track(it as u32),
                IndexField::PlacedNote(it) => ProtoKind::PlacedNote(it as u32),
                IndexField::Placement(it) => ProtoKind::Placement(it as u32),
                IndexField::Effect(it) => ProtoKind::Effect(it as u32),
                IndexField::Generator(it) => ProtoKind::Generator(it as u32),
                IndexField::Sample(it) => ProtoKind::Sample(it as u32),
            }),
        }
    }
}

#[derive(EnumString, Display, PartialEq, Clone, Debug)]
pub enum MultiIndexField {
    Track(Vec<usize>),
    PlacedNote(Vec<usize>),
    Placement(Vec<usize>),
    Effect(Vec<usize>),
    Generator(Vec<usize>),
    Sample(Vec<usize>),
}

impl From<MultiIndexFieldProto> for MultiIndexField {
    fn from(object: MultiIndexFieldProto) -> Self {
        let indexes = object.values.iter().map(|value| *value as usize).collect();
        match object.kind() {
            IndexFieldKind::UnknownIndexFieldKind => panic!(),
            IndexFieldKind::TrackIndexFieldKind => Self::Track(indexes),
            IndexFieldKind::PlacedNoteIndexFieldKind => Self::PlacedNote(indexes),
            IndexFieldKind::PlacementIndexFieldKind => Self::Placement(indexes),
            IndexFieldKind::EffectIndexFieldKind => Self::Effect(indexes),
            IndexFieldKind::GeneratorIndexFieldKind => Self::Generator(indexes),
            IndexFieldKind::SampleIndexFieldKind => Self::Sample(indexes),
        }
    }
}

impl From<MultiIndexField> for MultiIndexFieldProto {
    fn from(object: MultiIndexField) -> Self {
        match object {
            MultiIndexField::Track(indexes) => MultiIndexFieldProto {
                kind: IndexFieldKind::TrackIndexFieldKind.into(),
                values: indexes.iter().map(|value| *value as u32).collect(),
            },
            MultiIndexField::PlacedNote(indexes) => MultiIndexFieldProto {
                kind: IndexFieldKind::PlacedNoteIndexFieldKind.into(),
                values: indexes.iter().map(|value| *value as u32).collect(),
            },
            MultiIndexField::Placement(indexes) => MultiIndexFieldProto {
                kind: IndexFieldKind::PlacementIndexFieldKind.into(),
                values: indexes.iter().map(|value| *value as u32).collect(),
            },
            MultiIndexField::Effect(indexes) => MultiIndexFieldProto {
                kind: IndexFieldKind::EffectIndexFieldKind.into(),
                values: indexes.iter().map(|value| *value as u32).collect(),
            },
            MultiIndexField::Generator(indexes) => MultiIndexFieldProto {
                kind: IndexFieldKind::GeneratorIndexFieldKind.into(),
                values: indexes.iter().map(|value| *value as u32).collect(),
            },
            MultiIndexField::Sample(indexes) => MultiIndexFieldProto {
                kind: IndexFieldKind::SampleIndexFieldKind.into(),
                values: indexes.iter().map(|value| *value as u32).collect(),
            },
        }
    }
}
