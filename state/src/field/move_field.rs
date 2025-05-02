use local_macro::{FromProto, IntoProto};
use shared::action_proto::MoveFieldProto;

use super::IndexField;

#[derive(IntoProto, FromProto, Clone, Debug, PartialEq)]
pub struct MoveField {
    #[proto_optional]
    pub from_field: IndexField,
    #[proto_optional]
    pub to_field: IndexField,
}
