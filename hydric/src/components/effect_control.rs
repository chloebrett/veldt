use super::{delay_control, eq_control};
use egui::Ui;
use shared::model::{Effect, EffectInstance};

pub fn effect_control(effect: &mut EffectInstance, ui: &mut Ui) {
    let inner = &mut effect.effect;
    match inner {
        Effect::SimpleEq { config } => eq_control(config, &mut effect.meta, ui),
        Effect::SimpleDelay { config } => delay_control(config, &mut effect.meta, ui),
        Effect::SimpleCompressor { config } => panic!("Not implemented yet!"),
    }
}
