use super::{compressor_control::compressor_control, delay_control, eq_control, mod_delay_control};
use crate::components::WindowState;
use crate::widget::default_window;
use egui::Pos2;
use shared::model::Effect;
use state::{Action, Selector, Store};

pub fn effect_name(effect: &Effect) -> &str {
    match effect {
        Effect::SimpleEq { .. } => "EQ",
        Effect::SimpleDelay { .. } => "Delay",
        Effect::SimpleCompressor { .. } => "Compressor",
        Effect::ModDelay { .. } => "Modulated Delay",
    }
}

pub fn effect_control(
    ctx: &egui::Context,
    window_state: &mut WindowState,
    store: &Store,
    mixer_index: usize,
    effect_index: usize,
) {
    let sel = Selector::Effect(mixer_index, effect_index);
    let dispatch = |action| store.dispatch(&sel, action);
    let on_release = || store.dispatchr(Action::Release);
    let effect = &store.get().project.mixer[mixer_index].effects[effect_index];
    let title = effect_name(&effect.effect);

    default_window(title)
        .id(format!("effects_{mixer_index}_{effect_index}").into())
        .default_pos(Pos2 {
            x: 1000.0 + 50.0 * effect_index as f32,
            y: 150.0 + 50.0 * effect_index as f32,
        })
        .open(&mut window_state.effects[mixer_index][effect_index])
        .show(ctx, |ui| {
            match &effect.effect {
                Effect::SimpleEq { config } => eq_control(config, dispatch, on_release, ui),
                Effect::SimpleDelay { config } => delay_control(config, dispatch, on_release, ui),
                Effect::SimpleCompressor { config } => {
                    compressor_control(config, dispatch, on_release, ui)
                }
                Effect::ModDelay { config } => mod_delay_control(config, dispatch, on_release, ui),
            }

            ui.separator();
            ui.label(format!(
                "Mixer {} | Effect {}",
                mixer_index + 1,
                effect_index + 1
            ));
        });
}
