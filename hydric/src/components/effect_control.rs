use super::{compressor_control::compressor_control, delay_control, eq_control};
use egui::Ui;
use shared::model::Effect;
use state::{Selector, Store};

pub fn effect_control(store: &Store, effect_index: usize, ui: &mut Ui) {
    let mixer_index = 0;
    let sel = Selector::Effect(mixer_index, effect_index);
    let dispatch_effect = |action| {
        store.dispatch(&sel, action);
    };
    let effect = store.get().project.mixer[0].effects[effect_index].clone();
    let meta = effect.meta;
    match effect.effect {
        Effect::SimpleEq { config } => eq_control(config, meta, dispatch_effect, ui),
        Effect::SimpleDelay { config } => delay_control(config, meta, dispatch_effect, ui),
        Effect::SimpleCompressor { config } => {
            compressor_control(config, meta, dispatch_effect, ui)
        }
    }
}
