use crate::pmodel::ModMatrixProto;
use local_macro::{FromProto, IntoProto};

#[derive(Clone, Default, Debug, PartialEq, FromProto, IntoProto)]
pub struct ModMatrix {
    #[proto_type_u8]
    pub rows: u8,

    #[proto_type_u8]
    pub cols: u8,

    #[proto_repeated]
    pub matrix: Vec<f32>,
}
