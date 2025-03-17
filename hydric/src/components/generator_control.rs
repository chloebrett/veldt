use crate::state::{Action, Store, get_set};
use crate::widget::selectable_value;
use egui::Ui;
use shared::model::{GeneratorType, WaveType};
use shared::types::KnobPosition;
use strum::IntoEnumIterator;

pub fn generator_control(store: &Store, ui: &mut Ui) {
    let generator_type = store.get().project.generators[0].kind.clone();
    let config = match generator_type {
        GeneratorType::SimpleWave { config } => config,
    };

    egui::ComboBox::from_label("Wave type")
        .selected_text(config.wave.to_string())
        .show_ui(ui, |ui| {
            for wave in WaveType::iter() {
                selectable_value(
                    ui,
                    |it| {
                        it.map(|it| {
                            store.dispatch(Action::SetWave {
                                generator_index: 0,
                                wave: it,
                            })
                        });
                        config.wave
                    },
                    wave,
                    wave.to_string(),
                );
            }
        });
    ui.add(
        egui::Slider::from_get_set(
            0.0..=24.0,
            get_set(config.osc_count as f64, |it| {
                store.dispatch(Action::SetOscCount {
                    generator_index: 0,
                    osc_count: it as u32,
                })
            }),
        )
        .text("Osc count")
        .fixed_decimals(0),
    );
    ui.add(
        egui::Slider::from_get_set(
            0.0..=100.0,
            get_set(config.detune_cents as f64, |it| {
                store.dispatch(Action::SetDetuneCents {
                    generator_index: 0,
                    detune_cents: it as KnobPosition,
                })
            }),
        )
        .text("Osc detune")
        .logarithmic(true),
    );
}
