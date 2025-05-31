use super::generator_name;
use crate::WindowState;
use crate::widget::{StateWindow, add_knob, default_window, int_slider, styled_knob};
use crate::window_state::WindowKind;
use egui::{Button, Pos2, Ui};
use state::{Action, FloatField, GeneratorSelector, IndexField, Store, TypeField};

pub fn generators_control(ui: &mut Ui, window_state: &mut WindowState, store: &Store) {
    let generators = &store.get().project.generators;
    StateWindow(
        default_window("Generators")
            .id("generators".into())
            .default_pos(Pos2 {
                x: 1000.0,
                y: 150.0,
            }),
    )
    .show_with_closure(
        ui,
        window_state.get_visible(WindowKind::GeneratorList),
        |_| window_state.set_visible(WindowKind::GeneratorList, false),
        |ui| {
            for generator_index in 0..generators.len() {
                let sel = GeneratorSelector(generator_index);
                let on_release = || store.dispatchr(Action::Release);

                let generator = &generators[generator_index];
                let label = generator_name(generator);
                let show = window_state.get_visible(WindowKind::Generator(sel));
                let meta = generator.meta.clone();
                ui.horizontal(|ui| {
                    let mute_response = ui.add(Button::new("Mute").selected(meta.mute));
                    if mute_response.clicked() {
                        store.dispatch(&sel, Action::SetChild(TypeField::Mute(!meta.mute)))
                    }

                    let generator_response = ui.add(Button::new(label).selected(show));
                    if generator_response.clicked() {
                        window_state.set_visible(WindowKind::Generator(sel), !show);
                    }

                    add_knob(
                        ui,
                        styled_knob(
                            "Volume",
                            meta.volume,
                            |it| store.dispatch(&sel, Action::SetFloat(FloatField::Volume, it)),
                            // TODO: let this go up a bit past 1?
                            0.0..=1.0,
                        )
                        .with_neutral(0.8),
                        on_release,
                    );
                    add_knob(
                        ui,
                        styled_knob(
                            "Pan",
                            meta.pan,
                            |it| store.dispatch(&sel, Action::SetFloat(FloatField::Pan, it)),
                            -1.0..=1.0,
                        )
                        .with_neutral(0.0),
                        on_release,
                    );
                });

                // TODO: better UI than a slider for this!
                let max_channel_index = (store.get().project.mixer.channels.len() - 1) as i32;
                int_slider(
                    ui,
                    "Mixer channel",
                    meta.mixer_channel as f64,
                    |it| store.dispatch(&sel, Action::SetIndex(IndexField::Mixer(it as usize))),
                    0..=max_channel_index,
                    on_release,
                );

                if generator_index < generators.len() - 1 {
                    ui.separator();
                }
            }
        },
    );
}
