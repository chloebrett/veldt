use shared::action_proto::MoveFieldProto;

use super::IndexField;

#[derive(Clone, Debug, PartialEq)]
pub struct MoveField {
    pub from_field: IndexField,
    pub to_field: IndexField,
}

impl From<MoveFieldProto> for MoveField {
    fn from(object: MoveFieldProto) -> Self {
        Self {
            from_field: object.from_field.unwrap().into(),
            to_field: object.to_field.unwrap().into(),
        }
    }
}

impl From<MoveField> for MoveFieldProto {
    fn from(object: MoveField) -> Self {
        Self {
            from_field: Some(object.from_field.into()),
            to_field: Some(object.to_field.into()),
        }
    }
}
