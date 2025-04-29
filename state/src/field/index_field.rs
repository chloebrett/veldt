use shared::action_proto::{IndexFieldProto, index_field_proto::Kind as IndexFieldKind};
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
