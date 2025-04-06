use crate::widget::selectable_value;
use egui::Ui;
use shared::model::{AntiAliasingMode, GeneratorType, WaveType};
use shared::types::KnobPosition;
use state::{Action, Selector, Store, get_set};
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
            1.0..=24.0,
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

    egui::ComboBox::from_label("Anti aliasing mode")
        .selected_text(config.anti_aliasing_mode.to_string())
        .show_ui(ui, |ui| {
            for mode in AntiAliasingMode::iter() {
                selectable_value(
                    ui,
                    get_set(config.anti_aliasing_mode.clone(), |it| {
                        store.dispatch(&sel, Action::SetAntiAliasingMode(it))
                    }),
                    mode.clone(),
                    mode.clone().to_string(),
                );
            }
        });

    // Only show oversample factor if the anti-aliasing mode is oversample.
    if let AntiAliasingMode::Oversample = config.anti_aliasing_mode.clone() {
        ui.add(
            egui::Slider::from_get_set(
                2.0..=10.0,
                get_set(config.oversample_factor as f64, |it| {
                    store.dispatch(&sel, Action::SetOversampleFactor(it as u32))
                }),
            )
            .text("Oversample factor")
            .fixed_decimals(0),
        );
    }
}
