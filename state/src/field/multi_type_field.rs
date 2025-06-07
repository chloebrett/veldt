use shared::action_proto::{
    MultiTypeFieldKind, MultiTypeFieldProto, TypeFieldProto,
    type_field_proto::Kind as TypeFieldKind,
};
use shared::model::{EffectInstance, GeneratorInstance, PlacedNote, Placement, PlacementId, Track};
use strum::{Display, EnumString};

/// Extension of TypeField for working with multiple values.
#[derive(EnumString, Display, PartialEq, Clone, Debug)]
pub enum MultiTypeField {
    Track(Vec<Track>),
    PlacedNote(Vec<PlacedNote>),
    Placement(Vec<Placement>),
    Effect(Vec<EffectInstance>),
    Generator(Vec<GeneratorInstance>),
    PlacementId(Vec<PlacementId>),
}

impl From<MultiTypeFieldProto> for MultiTypeField {
    fn from(object: MultiTypeFieldProto) -> Self {
        match object.kind() {
            MultiTypeFieldKind::UnknownTypeFieldKind => panic!(),
            MultiTypeFieldKind::TrackTypeFieldKind => MultiTypeField::Track(
                object
                    .values
                    .iter()
                    .filter_map(|value| {
                        if let Some(TypeFieldKind::Track(track_proto)) = &value.kind {
                            Some(track_proto.clone().into())
                        } else {
                            None
                        }
                    })
                    .collect(),
            ),
            MultiTypeFieldKind::PlacedNoteTypeFieldKind => MultiTypeField::PlacedNote(
                object
                    .values
                    .iter()
                    .filter_map(|value| {
                        if let Some(TypeFieldKind::PlacedNote(note)) = value.kind {
                            Some(note.into())
                        } else {
                            None
                        }
                    })
                    .collect(),
            ),
            MultiTypeFieldKind::PlacementTypeFieldKind => MultiTypeField::Placement(
                object
                    .values
                    .iter()
                    .filter_map(|value| {
                        if let Some(TypeFieldKind::Placement(placement)) = value.kind {
                            Some(placement.into())
                        } else {
                            None
                        }
                    })
                    .collect(),
            ),
            MultiTypeFieldKind::EffectTypeFieldKind => MultiTypeField::Effect(
                object
                    .values
                    .iter()
                    .filter_map(|value| {
                        if let Some(TypeFieldKind::Effect(effect)) = value.kind {
                            Some(effect.into())
                        } else {
                            None
                        }
                    })
                    .collect(),
            ),
            MultiTypeFieldKind::GeneratorTypeFieldKind => MultiTypeField::Generator(
                object
                    .values
                    .iter()
                    .filter_map(|value| {
                        if let Some(TypeFieldKind::Generator(generator)) = &value.kind {
                            Some(generator.clone().into())
                        } else {
                            None
                        }
                    })
                    .collect(),
            ),
            MultiTypeFieldKind::PlacementIdTypeFieldKind => MultiTypeField::PlacementId(
                object
                    .values
                    .iter()
                    .filter_map(|value| {
                        if let Some(TypeFieldKind::PlacementId(placement_id)) = &value.kind {
                            Some((*placement_id).into())
                        } else {
                            None
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
                kind: MultiTypeFieldKind::TrackTypeFieldKind.into(),
                values: tracks
                    .into_iter()
                    .map(|value| TypeFieldProto {
                        kind: Some(TypeFieldKind::Track(value.into())),
                    })
                    .collect(),
            },
            MultiTypeField::PlacedNote(notes) => Self {
                kind: MultiTypeFieldKind::PlacedNoteTypeFieldKind.into(),
                values: notes
                    .into_iter()
                    .map(|value| TypeFieldProto {
                        kind: Some(TypeFieldKind::PlacedNote(value.into())),
                    })
                    .collect(),
            },
            MultiTypeField::Placement(placements) => Self {
                kind: MultiTypeFieldKind::PlacementTypeFieldKind.into(),
                values: placements
                    .into_iter()
                    .map(|value| TypeFieldProto {
                        kind: Some(TypeFieldKind::Placement(value.into())),
                    })
                    .collect(),
            },
            MultiTypeField::Effect(effects) => Self {
                kind: MultiTypeFieldKind::EffectTypeFieldKind.into(),
                values: effects
                    .into_iter()
                    .map(|value| TypeFieldProto {
                        kind: Some(TypeFieldKind::Effect(value.into())),
                    })
                    .collect(),
            },
            MultiTypeField::Generator(generators) => Self {
                kind: MultiTypeFieldKind::GeneratorTypeFieldKind.into(),
                values: generators
                    .into_iter()
                    .map(|value| TypeFieldProto {
                        kind: Some(TypeFieldKind::Generator(value.into())),
                    })
                    .collect(),
            },
            MultiTypeField::PlacementId(placement_ids) => Self {
                kind: MultiTypeFieldKind::PlacementIdTypeFieldKind.into(),
                values: placement_ids
                    .into_iter()
                    .map(|value| TypeFieldProto {
                        kind: Some(TypeFieldKind::PlacementId(value.into())),
                    })
                    .collect(),
            },
        }
    }
}
