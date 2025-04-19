use super::{simple_wave_control, subsynth_control};
use egui::Ui;
use shared::model::GeneratorType;
use state::{Action, Selector, Store};

pub fn generator_control(store: &Store, ui: &mut Ui, generator_index: usize) {
    let sel = Selector::Generator(generator_index);
    let generator = &store.get().project.generators[generator_index];
    let generator_type = generator.kind.clone();
    let dispatch = |action| store.dispatch(&sel, action);
    let on_release = || store.dispatchr(Action::Release);

    match generator_type {
        GeneratorType::SimpleWave { config } => {
            simple_wave_control(&config, dispatch, on_release, ui)
        }
        GeneratorType::Noise { .. } => todo!(),
        GeneratorType::SubSynth { config } => {
            subsynth_control(&config, dispatch, on_release, ui, store)
        },
    };
}
