use crate::state::{Action, Selector, Store, get_set};
use crate::widget::selectable_value;
use egui::Ui;
use shared::model::{GeneratorType, WaveType};
use shared::types::KnobPosition;
use strum::IntoEnumIterator;

pub fn generator_control(store: &Store, ui: &mut Ui) {
    let generator_index = 0;
    let sel = Selector::Generator(generator_index);
    let generator_type = store.get().project.generators[generator_index].kind.clone();
    let config = match generator_type {
        GeneratorType::SimpleWave { config } => config,
    };

    egui::ComboBox::from_label("Wave type")
        .selected_text(config.wave.to_string())
        .show_ui(ui, |ui| {
            for wave in WaveType::iter() {
                selectable_value(
                    ui,
                    get_set(config.wave, |it| store.dispatch(&sel, Action::SetWave(it))),
                    wave,
                    wave.to_string(),
                );
            }
        });
    ui.add(
        egui::Slider::from_get_set(
            0.0..=24.0,
            get_set(config.osc_count as f64, |it| {
                store.dispatch(&sel, Action::SetOscCount(it as u32))
            }),
        )
        .text("Osc count")
        .fixed_decimals(0),
    );
    ui.add(
        egui::Slider::from_get_set(
            0.0..=100.0,
            get_set(config.detune_cents as f64, |it| {
                store.dispatch(&sel, Action::SetDetuneCents(it as KnobPosition))
            }),
        )
        .text("Osc detune")
        .logarithmic(true),
    );
}
