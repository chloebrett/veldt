use crate::WindowState;
use crate::widget::{default_window, knob};
use egui::{Button, Pos2};
use shared::model::Generator;
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
                let on_release = || store.dispatchr(Action::Release);

                let generator = &generators[generator_index];
                let label = match &generator.it {
                    Generator::SimpleWave { .. } => "Simple Wave Generator",
                    Generator::Noise { .. } => "Noise Generator",
                    Generator::SubSynth { .. } => "Subtractive Synthesiser",
                };
                let show = window_state.generators.get(generator_index);
                let meta = generator.meta.clone();
                ui.horizontal(|ui| {
                    let mute_response = ui.add(Button::new("Mute").selected(meta.mute));
                    if mute_response.clicked() {
                        store.dispatch(&sel, Action::SetChild(TypeField::Mute(!meta.mute)))
                    }

                    let generator_response = ui.add(Button::new(label).selected(show));
                    if generator_response.clicked() {
                        window_state.generators.set(generator_index, !show);
                    }

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
                });
                if generator_index < generators.len() - 1 {
                    ui.separator();
                }
            }
        });
}
