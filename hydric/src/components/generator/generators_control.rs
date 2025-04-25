use crate::WindowState;
use crate::widget::{checkbox, default_window, knob};
use egui::Pos2;
use shared::model::GeneratorType;
use state::{Action, FloatField, Selector, Store, TypeField};

pub fn generators_control(ctx: &egui::Context, window_state: &mut WindowState, store: &Store) {
    let generators = &store.get().project.generators;
    let visible = &mut window_state.generator_list;

    default_window("Generators")
        .id("generators".into())
        .default_pos(Pos2 {
            x: 1000.0,
            y: 150.0,
        })
        .open(visible)
        .show(ctx, |ui| {
            for generator_index in 0..generators.len() {
                let sel = Selector::Generator(generator_index);
                let on_release = || store.(Action::Release);

                let generator = &generators[generator_index];
                let label = match &generator.kind {
                    GeneratorType::SimpleWave { .. } => "Simple Wave Generator",
                    GeneratorType::Noise { .. } => "Noise Generator",
                    GeneratorType::SubSynth { .. } => "Subtractive Synthesiser",
                };
                ui.label(label);

                let show = window_state.generators.get(generator_index);
                let text = if show { "Hide" } else { "Show" };
                if ui.button(text).clicked() {
                    window_state.generators.set(generator_index, !show);
                }

                if ui.button("Add").clicked() { //is there a limit to the no. of generators we want?
                    store.dispatch(
                        sel,
                        Action::AddChild(TypeField::Effect(EffectInstance)), 
                    );
                }

                if generators.len() > 1 && ui.button("Delete").clicked() {
                    store.(Action::DeleteChild(IndexField::Generator(
                        sel,
                    )));
                    break;
                }

                let meta = generator.meta.clone();
                knob(
                    ui,
                    "Volume",
                    meta.volume,
                    |it| store.dispatch(&sel, Action::SetFloat(FloatField::Volume, it)),
                    // TODO: let this go up a bit past 1?
                    0.0..=1.0,
                    /* neutral= */ 0.8,
                    on_release,
                );
                // TODO: make the pan knob centre at the top since it's bipolar.
                knob(
                    ui,
                    "Pan",
                    meta.pan,
                    |it| store.dispatch(&sel, Action::SetFloat(FloatField::Pan, it)),
                    -1.0..=1.0,
                    /* neutral= */ 0.0,
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
