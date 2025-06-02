use super::generator_name;
use crate::local_state::LocalState;
use crate::widget::{StateWindow, add_knob, int_slider, styled_knob};
use crate::window_state::WindowKind;
use egui::{Button, Ui};
use shared::model::{GeneratorId, GeneratorInstance};
use state::{Action, FloatField, GeneratorSelector, IndexField, Store, TypeField};

pub fn generators_control(ui: &mut Ui, local_state: &LocalState, store: &Store) {
    let generators: Vec<(&GeneratorId, &GeneratorInstance)> =
        store.get().project.generators.iter().collect();

    StateWindow::show_from_window_state(
        ui,
        &local_state.window_state,
        WindowKind::GeneratorList,
        "Generators",
        |ui| {
            for (index, (generator_id, generator)) in generators.clone().into_iter().enumerate() {
                let sel = GeneratorSelector(*generator_id);
                let on_release = || store.dispatchr(Action::Release);

                let label = generator_name(generator);
                let show = local_state
                    .window_state
                    .get_visible(WindowKind::Generator(sel));
                let meta = generator.meta.clone();
                ui.horizontal(|ui| {
                    let mute_response = ui.add(Button::new("Mute").selected(meta.mute));
                    if mute_response.clicked() {
                        store.dispatch(&sel, Action::SetChild(TypeField::Mute(!meta.mute)))
                    }

                    let generator_response = ui.add(Button::new(label).selected(show));
                    if generator_response.clicked() {
                        local_state
                            .window_state
                            .set_visible(WindowKind::Generator(sel), !show);
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

                if index < generators.len() - 1 {
                    ui.separator();
                }
            }
        },
    );
}
