use shared::action_proto::{
    MultiIndexFieldKind, IndexFieldProto, MultiIndexFieldProto, index_field_proto::Kind as IndexfieldKind,
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
            IndexfieldKind::Track(it) => IndexField::Track(it as usize),
            IndexfieldKind::PlacedNote(it) => IndexField::PlacedNote(it as usize),
            IndexfieldKind::Placement(it) => IndexField::Placement(it as usize),
            IndexfieldKind::Effect(it) => IndexField::Effect(it as usize),
            IndexfieldKind::Generator(it) => IndexField::Generator(it as usize),
            IndexfieldKind::Sample(it) => IndexField::Sample(it as usize),
        }
    }
}

impl From<IndexField> for IndexFieldProto {
    fn from(other: IndexField) -> Self {
        IndexFieldProto {
            kind: Some(match other {
                IndexField::Track(it) => IndexfieldKind::Track(it as u32),
                IndexField::PlacedNote(it) => IndexfieldKind::PlacedNote(it as u32),
                IndexField::Placement(it) => IndexfieldKind::Placement(it as u32),
                IndexField::Effect(it) => IndexfieldKind::Effect(it as u32),
                IndexField::Generator(it) => IndexfieldKind::Generator(it as u32),
                IndexField::Sample(it) => IndexfieldKind::Sample(it as u32),
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
            MultiIndexFieldKind::UnknownIndexFieldKind => panic!(),
            MultiIndexFieldKind::TrackIndexFieldKind => Self::Track(indexes),
            MultiIndexFieldKind::PlacedNoteIndexFieldKind => Self::PlacedNote(indexes),
            MultiIndexFieldKind::PlacementIndexFieldKind => Self::Placement(indexes),
            MultiIndexFieldKind::EffectIndexFieldKind => Self::Effect(indexes),
            MultiIndexFieldKind::GeneratorIndexFieldKind => Self::Generator(indexes),
            MultiIndexFieldKind::SampleIndexFieldKind => Self::Sample(indexes),
        }
    }
}

impl From<MultiIndexField> for MultiIndexFieldProto {
    fn from(object: MultiIndexField) -> Self {
        match object {
            MultiIndexField::Track(indexes) => MultiIndexFieldProto {
                kind: MultiIndexFieldKind::TrackIndexFieldKind.into(),
                values: indexes.iter().map(|value| *value as u32).collect(),
            },
            MultiIndexField::PlacedNote(indexes) => MultiIndexFieldProto {
                kind: MultiIndexFieldKind::PlacedNoteIndexFieldKind.into(),
                values: indexes.iter().map(|value| *value as u32).collect(),
            },
            MultiIndexField::Placement(indexes) => MultiIndexFieldProto {
                kind: MultiIndexFieldKind::PlacementIndexFieldKind.into(),
                values: indexes.iter().map(|value| *value as u32).collect(),
            },
            MultiIndexField::Effect(indexes) => MultiIndexFieldProto {
                kind: MultiIndexFieldKind::EffectIndexFieldKind.into(),
                values: indexes.iter().map(|value| *value as u32).collect(),
            },
            MultiIndexField::Generator(indexes) => MultiIndexFieldProto {
                kind: MultiIndexFieldKind::GeneratorIndexFieldKind.into(),
                values: indexes.iter().map(|value| *value as u32).collect(),
            },
            MultiIndexField::Sample(indexes) => MultiIndexFieldProto {
                kind: MultiIndexFieldKind::SampleIndexFieldKind.into(),
                values: indexes.iter().map(|value| *value as u32).collect(),
            },
        }
    }
}
