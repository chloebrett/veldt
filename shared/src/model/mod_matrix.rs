use crate::pmodel::ModMatrixProto;
use local_macro::{FromProto, IntoProto};

use super::MatrixCell;

#[derive(Clone, Default, Debug, PartialEq, FromProto, IntoProto)]
pub struct ModMatrix {
    #[proto_type_u8]
    pub rows: u8,

    #[proto_type_u8]
    pub cols: u8,

    #[proto_repeated]
    pub matrix: Vec<MatrixCell>,
}

impl ModMatrix {
    pub fn new(num_rows: u8, num_cols: u8) -> Self {
        ModMatrix {
            rows: num_rows,
            cols: num_cols,
            matrix: vec![MatrixCell(0.0); num_rows as usize * num_cols as usize],
        }
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&MatrixCell> {
        self.matrix.get(row * self.cols as usize + col)
    }

    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut MatrixCell> {
        self.matrix.get_mut(row * self.cols as usize + col)
    }
}
