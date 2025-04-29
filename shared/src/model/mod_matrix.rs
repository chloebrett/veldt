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

impl ModMatrix {
    pub fn new(num_rows: u8, num_cols: u8) -> Self {
        ModMatrix { rows: num_rows, cols: num_cols, matrix: vec![0.0; num_rows as usize * num_cols as usize] }
    }
}
