use shared::action_proto::{
    IndexFieldProto, MultiIndexFieldKind, MultiIndexFieldProto,
    index_field_proto::Kind as IndexFieldKind,
};
use strum::{Display, EnumString};

/// Fields that index into a list.
/// Used to distinguish *which* index is being referred to, and also contains the index value.
/// Note that this is only for *indexes* - most entities have an *id* type, and so will use
/// e.g. TypeField::GeneratorId instead of IndexField.
#[derive(EnumString, Display, PartialEq, Clone, Debug)]
pub enum IndexField {
    PlacedNote(usize),
    // Index of an effect ID within a mixer channel.
    EffectId(usize),
    Mixer(usize),
}

impl From<IndexFieldProto> for IndexField {
    fn from(other: IndexFieldProto) -> Self {
        match other.kind.unwrap() {
            IndexFieldKind::PlacedNote(it) => IndexField::PlacedNote(it as usize),
            IndexFieldKind::EffectId(it) => IndexField::EffectId(it as usize),
            IndexFieldKind::Mixer(it) => IndexField::Mixer(it as usize),
        }
    }
}

impl From<IndexField> for IndexFieldProto {
    fn from(other: IndexField) -> Self {
        IndexFieldProto {
            kind: Some(match other {
                IndexField::PlacedNote(it) => IndexFieldKind::PlacedNote(it as u32),
                IndexField::EffectId(it) => IndexFieldKind::EffectId(it as u32),
                IndexField::Mixer(it) => IndexFieldKind::Mixer(it as u32),
            }),
        }
    }
}

#[derive(EnumString, Display, PartialEq, Clone, Debug)]
pub enum MultiIndexField {
    PlacedNote(Vec<usize>),
}

impl From<MultiIndexFieldProto> for MultiIndexField {
    fn from(object: MultiIndexFieldProto) -> Self {
        let indexes = object.values.iter().map(|value| *value as usize).collect();
        match object.kind() {
            MultiIndexFieldKind::UnknownIndexFieldKind => panic!(),
            MultiIndexFieldKind::PlacedNoteIndexFieldKind => Self::PlacedNote(indexes),
        }
    }
}

impl From<MultiIndexField> for MultiIndexFieldProto {
    fn from(object: MultiIndexField) -> Self {
        match object {
            MultiIndexField::PlacedNote(indexes) => MultiIndexFieldProto {
                kind: MultiIndexFieldKind::PlacedNoteIndexFieldKind.into(),
                values: to_u32s(indexes),
            },
        }
    }
}

fn to_u32s(usizes: Vec<usize>) -> Vec<u32> {
    usizes.iter().map(|value| *value as u32).collect()
}
