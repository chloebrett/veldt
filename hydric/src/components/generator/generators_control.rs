use super::generator_name;
use crate::WindowState;
use crate::widget::{default_window, int_slider, knob};
use egui::{Button, Pos2};
use state::{Action, FloatField, GeneratorSelector, IndexField, Store, TypeField};

use crate::LocalState;
use crate::local_state::GetSet;

pub fn generators_control(ctx: &egui::Context, window_state: &mut WindowState, store: &Store, local_state: &LocalState) {
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
                let sel = GeneratorSelector(generator_index);
                let on_release = || store.dispatchr(Action::Release);

                let generator = &generators[generator_index];
                let label = generator_name(generator);
                let show = window_state.generators.get(sel);
                let meta = generator.meta.clone();
                ui.horizontal(|ui| {
                    let mute_response = ui.add(Button::new("Mute").selected(meta.mute));
                    if mute_response.clicked() {
                        store.dispatch(&sel, Action::SetChild(TypeField::Mute(!meta.mute)))
                    }

                    let generator_response = ui.add(Button::new(label).selected(show));
                    if generator_response.clicked() {
                        window_state.generators.set(sel, !show);
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

            let mut selected = local_state.new_selected_generator.get();

            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Select one!")
                    .selected_text(format!("{:?}", selected))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, "Simple Wave".to_string(), "Simple Wave Generator");
                        ui.selectable_value(&mut selected, "Subtractive Synth".to_string(), "Stringray (Subtrative Synth)");
                        ui.selectable_value(&mut selected, "Noise Generator".to_string(), "Noise Generator");
                    }
                );

                local_state.new_selected_generator.set(selected);

                if ui.button("Add Generator").clicked(){
                    ui.label("Do something");
                    
                }
            });
        });
}
