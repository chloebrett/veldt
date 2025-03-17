use shared::model::{Scale, ScaleValue};
use shared::types::Beats;

pub enum Action {
    SetKey(ScaleValue),
    SetScale(Scale),
    SetBpm(Beats),
    SetNote {
        track_index: usize,
        note_index: usize,
        note: ScaleValue,
    },
}
