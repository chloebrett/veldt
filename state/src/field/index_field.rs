use shared::action_proto::{
    IndexFieldProto, MultiIndexFieldKind, MultiIndexFieldProto,
    index_field_proto::Kind as IndexFieldKind,
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
            IndexFieldKind::Track(it) => IndexField::Track(it as usize),
            IndexFieldKind::PlacedNote(it) => IndexField::PlacedNote(it as usize),
            IndexFieldKind::Placement(it) => IndexField::Placement(it as usize),
            IndexFieldKind::Effect(it) => IndexField::Effect(it as usize),
            IndexFieldKind::Generator(it) => IndexField::Generator(it as usize),
            IndexFieldKind::Sample(it) => IndexField::Sample(it as usize),
        }
    }
}

impl From<IndexField> for IndexFieldProto {
    fn from(other: IndexField) -> Self {
        IndexFieldProto {
            kind: Some(match other {
                IndexField::Track(it) => IndexFieldKind::Track(it as u32),
                IndexField::PlacedNote(it) => IndexFieldKind::PlacedNote(it as u32),
                IndexField::Placement(it) => IndexFieldKind::Placement(it as u32),
                IndexField::Effect(it) => IndexFieldKind::Effect(it as u32),
                IndexField::Generator(it) => IndexFieldKind::Generator(it as u32),
                IndexField::Sample(it) => IndexFieldKind::Sample(it as u32),
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
                values: to_u32s(indexes),
            },
            MultiIndexField::PlacedNote(indexes) => MultiIndexFieldProto {
                kind: MultiIndexFieldKind::PlacedNoteIndexFieldKind.into(),
                values: to_u32s(indexes),
            },
            MultiIndexField::Placement(indexes) => MultiIndexFieldProto {
                kind: MultiIndexFieldKind::PlacementIndexFieldKind.into(),
                values: to_u32s(indexes),
            },
            MultiIndexField::Effect(indexes) => MultiIndexFieldProto {
                kind: MultiIndexFieldKind::EffectIndexFieldKind.into(),
                values: to_u32s(indexes),
            },
            MultiIndexField::Generator(indexes) => MultiIndexFieldProto {
                kind: MultiIndexFieldKind::GeneratorIndexFieldKind.into(),
                values: to_u32s(indexes),
            },
            MultiIndexField::Sample(indexes) => MultiIndexFieldProto {
                kind: MultiIndexFieldKind::SampleIndexFieldKind.into(),
                values: to_u32s(indexes),
            },
        }
    }
}

fn to_u32s(usizes: Vec<usize>) -> Vec<u32> {
    usizes.iter().map(|value| *value as u32).collect()
}
