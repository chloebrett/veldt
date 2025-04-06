use super::{compressor_control::compressor_control, delay_control, eq_control};
use egui::{Pos2, Ui};
use shared::model::Effect;
use shared::types::KnobPosition;
use state::{Action, Selector, Store, get_set};

pub fn effect_control(ctx: &egui::Context, store: &Store, mixer_index: usize, effect_index: usize) {
    let sel = Selector::Effect(mixer_index, effect_index);
    let dispatch = |action| store.dispatch(&sel, action);
    let effect = store.get().project.mixer[mixer_index].effects[effect_index].clone();

    let title = match effect.effect {
        Effect::SimpleEq { .. } => "EQ",
        Effect::SimpleDelay { .. } => "Delay",
        Effect::SimpleCompressor { .. } => "Compressor",
    };

    egui::Window::new(title)
        .id(format!("effects_{mixer_index}_{effect_index}").into())
        .default_pos(Pos2 {
            x: 1000.0 + 50.0 * effect_index as f32,
            y: 150.0 + 50.0 * effect_index as f32,
        })
        .resizable(false)
        .show(ctx, |ui| {
            match effect.effect {
                Effect::SimpleEq { config } => eq_control(&config, dispatch, ui),
                Effect::SimpleDelay { config } => delay_control(&config, dispatch, ui),
                Effect::SimpleCompressor { config } => compressor_control(&config, dispatch, ui),
            }

            let meta = effect.meta;
            ui.add(
                egui::Slider::from_get_set(
                    0.0..=1.0,
                    get_set(meta.wet.into(), |it| {
                        dispatch(Action::SetEffectWet(it as KnobPosition))
                    }),
                )
                .text("Wet"),
            );
            checkbox_get_set(
                ui,
                meta.mute,
                |it| dispatch(Action::SetEffectMute(it)),
                "Mute",
            );

            ui.separator();
            ui.label(format!(
                "Mixer {} | Effect {}",
                mixer_index + 1,
                effect_index + 1
            ));
        });
}

fn checkbox_get_set(ui: &mut Ui, value: bool, setter: impl Fn(bool), text: &str) {
    let mut temp = value;
    ui.checkbox(&mut temp, text);
    if value != temp {
        setter(temp);
    }
}
