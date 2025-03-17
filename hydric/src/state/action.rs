use shared::model::{Scale, ScaleValue};

pub enum Action {
    SetKey(ScaleValue),
    SetScale(Scale),
    SetProjectName(String),
}
