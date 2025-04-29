use std::collections::BTreeSet;

use shared::action_proto::{IndexFieldKindProto, IndexFieldProto, MultiIndexFieldProto};

/// Fields that index into a list.
/// Used to distinguish *which* index is being referred to, and also contains the index value.
#[derive(PartialEq, Clone, Debug, PartialOrd, Ord, Eq)]
pub enum IndexField {
    Track(usize),
    PlacedNote(usize),
    TrackPlacement(usize),
    Effect(usize),
    Generator(usize),
}

impl From<IndexFieldProto> for IndexField {
    fn from(other: IndexFieldProto) -> Self {
        match other.kind() {
            IndexFieldKindProto::UnknownIndexFieldKind => panic!(),
            IndexFieldKindProto::TrackIndexField => IndexField::Track(other.index as usize),
            IndexFieldKindProto::PlacedNoteIndexField => {
                IndexField::PlacedNote(other.index as usize)
            }
            IndexFieldKindProto::TrackPlacementIndexField => {
                IndexField::TrackPlacement(other.index as usize)
            }
            IndexFieldKindProto::EffectIndexField => IndexField::Effect(other.index as usize),
            IndexFieldKindProto::GeneratorIndexField => IndexField::Generator(other.index as usize),
        }
    }
}

impl From<IndexField> for IndexFieldProto {
    fn from(other: IndexField) -> Self {
        match other {
            IndexField::Track(it) => IndexFieldProto {
                kind: IndexFieldKindProto::TrackIndexField.into(),
                index: it as u32,
            },
            IndexField::PlacedNote(it) => IndexFieldProto {
                kind: IndexFieldKindProto::PlacedNoteIndexField.into(),
                index: it as u32,
            },
            IndexField::TrackPlacement(it) => IndexFieldProto {
                kind: IndexFieldKindProto::TrackPlacementIndexField.into(),
                index: it as u32,
            },
            IndexField::Effect(it) => IndexFieldProto {
                kind: IndexFieldKindProto::EffectIndexField.into(),
                index: it as u32,
            },
            IndexField::Generator(it) => IndexFieldProto {
                kind: IndexFieldKindProto::GeneratorIndexField.into(),
                index: it as u32,
            },
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum MultiIndexField {
    Track(Vec<usize>),
    PlacedNote(Vec<usize>),
    TrackPlacement(Vec<usize>),
    Effect(Vec<usize>),
    Generator(Vec<usize>),
}

impl From<MultiIndexFieldProto> for MultiIndexField {
    fn from(object: MultiIndexFieldProto) -> Self {
        let indexes: Vec<usize> = object.values.iter().map(|it| it.index as usize).collect();
        match object.kind() {
            IndexFieldKindProto::UnknownIndexFieldKind => panic!(),
            IndexFieldKindProto::TrackIndexField => MultiIndexField::Track(indexes),
            IndexFieldKindProto::PlacedNoteIndexField => MultiIndexField::PlacedNote(indexes),
            IndexFieldKindProto::TrackPlacementIndexField => {
                MultiIndexField::TrackPlacement(indexes)
            }
            IndexFieldKindProto::EffectIndexField => MultiIndexField::Effect(indexes),
            IndexFieldKindProto::GeneratorIndexField => MultiIndexField::Generator(indexes),
        }
    }
}

impl From<MultiIndexField> for MultiIndexFieldProto {
    fn from(object: MultiIndexField) -> Self {
        match object {
            MultiIndexField::Track(it) => MultiIndexFieldProto {
                values: it
                    .iter()
                    .map(|&index| IndexFieldProto {
                        kind: IndexFieldKindProto::TrackIndexField.into(),
                        index: index as u32,
                    })
                    .collect(),
                kind: IndexFieldKindProto::TrackIndexField.into(),
            },
            MultiIndexField::PlacedNote(it) => MultiIndexFieldProto {
                values: it
                    .iter()
                    .map(|&index| IndexFieldProto {
                        kind: IndexFieldKindProto::PlacedNoteIndexField.into(),
                        index: index as u32,
                    })
                    .collect(),
                kind: IndexFieldKindProto::PlacedNoteIndexField.into(),
            },
            MultiIndexField::TrackPlacement(it) => MultiIndexFieldProto {
                values: it
                    .iter()
                    .map(|&index| IndexFieldProto {
                        kind: IndexFieldKindProto::TrackPlacementIndexField.into(),
                        index: index as u32,
                    })
                    .collect(),
                kind: IndexFieldKindProto::TrackPlacementIndexField.into(),
            },
            MultiIndexField::Effect(it) => MultiIndexFieldProto {
                values: it
                    .iter()
                    .map(|&index| IndexFieldProto {
                        kind: IndexFieldKindProto::EffectIndexField.into(),
                        index: index as u32,
                    })
                    .collect(),
                kind: IndexFieldKindProto::EffectIndexField.into(),
            },
            MultiIndexField::Generator(it) => MultiIndexFieldProto {
                values: it
                    .iter()
                    .map(|&index| IndexFieldProto {
                        kind: IndexFieldKindProto::GeneratorIndexField.into(),
                        index: index as u32,
                    })
                    .collect(),
                kind: IndexFieldKindProto::GeneratorIndexField.into(),
            },
        }
    }
}
