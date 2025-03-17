use super::{delay_control, eq_control};
use crate::state::Store;
use egui::Ui;
use shared::model::Effect;

pub fn effect_control(store: &Store, effect_index: usize, ui: &mut Ui) {
    let effect = store.get().project.mixer[0].effects[effect_index].clone();
    match effect.effect {
        Effect::SimpleEq { .. } => eq_control(store, effect_index, ui),
        Effect::SimpleDelay { .. } => delay_control(store, effect_index, ui),
        Effect::SimpleCompressor { .. } => panic!("Not implemented yet!"),
    }
}
