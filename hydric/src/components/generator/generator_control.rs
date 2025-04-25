use super::{simple_wave_control, subsynth_control};
use crate::widget::StateWindow;
use crate::widget::default_window;
use egui::{Pos2, Ui};
use shared::model::GeneratorType;
use state::{Action, Selector, Store};

pub fn generator_control(
    store: &Store,
    ui: &mut Ui,
    generator_index: usize,
    visible: bool,
    mut on_close: impl FnMut(),
) {
    let sel = Selector::Generator(generator_index);
    let generator = &store.get().project.generators[generator_index];

    let title = match &generator.kind {
        GeneratorType::SimpleWave { .. } => "Simple Wave Generator",
        GeneratorType::Noise { .. } => "Noise Generator",
        GeneratorType::SubSynth { .. } => "Subtractive Synth",
    };

    StateWindow(default_window(title).default_pos(Pos2 { x: 1100.0, y: 20.0 })).show_with_closure(
        ui,
        visible,
        |_| on_close(),
        |ui| {
            let generator_type = generator.kind.clone();
            let dispatch = |action| store.dispatch(&sel, action);
            let on_release = || store.dispatchr(Action::Release);

            match generator_type {
                GeneratorType::SimpleWave { config } => {
                    simple_wave_control(&config, dispatch, on_release, ui)
                }
                GeneratorType::Noise { .. } => todo!(),
                GeneratorType::SubSynth { config } => {
                    subsynth_control(&config, dispatch, on_release, ui)
                }
            };
        },
    );
}
