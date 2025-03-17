use super::{delay_control, eq_control};
use egui::Ui;
use shared::model::{Effect, EffectInstance};

pub fn effect_control(effect: RefMut<'_, EffectInstance>, ui: &mut Ui) {
    let inner = effect.map(|it| it.effect);
    match inner {
        Effect::SimpleEq { config } => eq_control(config, effect.map(|it| it.meta), ui),
        Effect::SimpleDelay { config } => delay_control(config, effect.map(|it| it.meta), ui),
        Effect::SimpleCompressor { .. } => panic!("Not implemented yet!"),
    }
}
