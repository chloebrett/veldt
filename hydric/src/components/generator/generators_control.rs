use crate::components::WindowState;
use crate::widget::{FloatRange, checkbox, default_window, knob};
use egui::Pos2;
use shared::model::GeneratorType;
use state::{Action, Selector, Store};

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

                let generator = &generators[generator_index];
                let label = match &generator.kind {
                    GeneratorType::SimpleWave { .. } => "Simple Wave Generator",
                    GeneratorType::Noise { .. } => "Noise Generator",
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
                    |it| store.dispatch(&sel, Action::SetGeneratorVolume(it)),
                    FloatRange(0.0, 1.0),
                );
                // TODO: make the pan knob centre at the top since it's bipolar.
                knob(
                    ui,
                    "Pan",
                    meta.pan,
                    |it| store.dispatch(&sel, Action::SetGeneratorPan(it)),
                    FloatRange(-1.0, 1.0),
                );
                checkbox(
                    ui,
                    meta.mute,
                    |it| store.dispatch(&sel, Action::SetGeneratorMute(it)),
                    "Mute",
                );

                if generator_index < generators.len() - 1 {
                    ui.separator();
                }
            }
        });
}
