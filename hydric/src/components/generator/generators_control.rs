use crate::WindowState;
use crate::widget::{checkbox, default_window, knob};
use egui::Pos2;
use shared::model::GeneratorType;
use state::{Action, FloatField, Selector, Store, TypeField};

pub fn generators_control(ctx: &egui::Context, window_state: &mut WindowState, store: &Store) {
    let generators = &store.get().project.generators;

    default_window("Generators")
        .id("generators".into())
        .default_pos(Pos2 {
            x: 1000.0,
            y: 150.0,
        })
        .open(&mut window_state.generator_list)
        .show(ctx, |ui| {
            for generator_index in 0..generators.len() {
                let sel = Selector::Generator(generator_index);
                let on_release = || store.dispatchr(Action::Release);

                let generator = &generators[generator_index];
                let label = match &generator.kind {
                    GeneratorType::SimpleWave { .. } => "Simple Wave Generator",
                    GeneratorType::Noise { .. } => "Noise Generator",
                    GeneratorType::SubSynth { .. } => "Subtractive Synthesiser",
                };
                ui.label(label);

                let show = &mut window_state.generators[generator_index];
                let text = if *show { "Hide" } else { "Show" };
                if ui.button(text).clicked() {
                    *show = !*show;
                }

                let meta = generator.meta.clone();
                knob(
                    ui,
                    "Volume",
                    meta.volume,
                    |it| store.dispatch(&sel, Action::SetFloat(FloatField::Volume, it)),
                    0.0..=1.0,
                    on_release,
                );
                // TODO: make the pan knob centre at the top since it's bipolar.
                knob(
                    ui,
                    "Pan",
                    meta.pan,
                    |it| store.dispatch(&sel, Action::SetFloat(FloatField::Pan, it)),
                    -1.0..=1.0,
                    on_release,
                );
                checkbox(
                    ui,
                    meta.mute,
                    |it| store.dispatch(&sel, Action::SetChild(TypeField::Mute(it))),
                    "Mute",
                );

                if generator_index < generators.len() - 1 {
                    ui.separator();
                }
            }
        });
}
