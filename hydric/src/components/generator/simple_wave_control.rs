use crate::widget::selectable_value;
use egui::Ui;
use shared::model::{AntiAliasingMode, SimpleWaveConfig, WaveType};
use shared::types::KnobPosition;
use state::{Action, get_set};
use strum::IntoEnumIterator;

pub fn simple_wave_control<F>(config: &SimpleWaveConfig, dispatch_generator: F, ui: &mut Ui)
where
    F: Fn(Action),
{
    egui::ComboBox::from_label("Wave type")
        .selected_text(config.wave.to_string())
        .show_ui(ui, |ui| {
            for wave in WaveType::iter() {
                selectable_value(
                    ui,
                    get_set(config.wave, |it| dispatch_generator(Action::SetWave(it))),
                    wave,
                    wave.to_string(),
                );
            }
        });
    ui.add(
        egui::Slider::from_get_set(
            1.0..=24.0,
            get_set(config.osc_count as f64, |it| {
                dispatch_generator(Action::SetOscCount(it as u32))
            }),
        )
        .text("Osc count")
        .fixed_decimals(0),
    );
    ui.add(
        egui::Slider::from_get_set(
            0.0..=100.0,
            get_set(config.detune_cents as f64, |it| {
                dispatch_generator(Action::SetDetuneCents(it as KnobPosition))
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
                    get_set(config.anti_aliasing_mode, |it| {
                        dispatch_generator(Action::SetAntiAliasingMode(it))
                    }),
                    mode,
                    mode.to_string(),
                );
            }
        });

    // Only show oversample factor if the anti-aliasing mode is oversample.
    if let AntiAliasingMode::Oversample = config.anti_aliasing_mode {
        ui.add(
            egui::Slider::from_get_set(
                2.0..=10.0,
                get_set(config.oversample_factor as f64, |it| {
                    dispatch_generator(Action::SetOversampleFactor(it as u32))
                }),
            )
            .text("Oversample factor")
            .fixed_decimals(0),
        );
    }
}
