use super::simple_wave_control;
use crate::widget::default_window;
use egui::{Pos2, Ui};
use shared::model::GeneratorType;
use state::{Action, Selector, Store};

pub fn generator_control(store: &Store, ui: &mut Ui, generator_index: usize, visible: &mut bool) {
    let sel = Selector::Generator(generator_index);
    let generator = &store.get().project.generators[generator_index];

    let title = match &generator.kind {
        GeneratorType::SimpleWave { .. } => "Simple Wave Generator",
        GeneratorType::Noise { .. } => "Noise Generator",
        GeneratorType::SubSynth { .. } => "Subtractive Synth",
    };
    default_window(title)
        .open(visible)
        .default_pos(Pos2 { x: 1100.0, y: 20.0 })
        .show(ui.ctx(), |ui| {
            let generator_type = generator.kind.clone();
            let dispatch = |action| store.dispatch(&sel, action);
            let on_release = || store.dispatchr(Action::Release);

            match generator_type {
                GeneratorType::SimpleWave { config } => {
                    simple_wave_control(&config, dispatch, on_release, ui)
                }
                GeneratorType::Noise { .. } => todo!(),
                GeneratorType::SubSynth { .. } => todo!(),
            };
        });
}
