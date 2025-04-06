use crate::widget::{FloatRange, knob, selectable_value};
use egui::Ui;
use shared::model::{AntiAliasingMode, SimpleWaveConfig, WaveType};
use state::{Action, get_set};
use strum::IntoEnumIterator;

pub fn simple_wave_control<F>(config: &SimpleWaveConfig, dispatch: F, ui: &mut Ui)
where
    F: Fn(Action),
{
    egui::ComboBox::from_label("Wave type")
        .selected_text(config.wave.to_string())
        .show_ui(ui, |ui| {
            for wave in WaveType::iter() {
                selectable_value(
                    ui,
                    get_set(config.wave, |it| dispatch(Action::SetWave(it))),
                    wave,
                    wave.to_string(),
                );
            }
        });
    ui.add(
        egui::Slider::from_get_set(
            1.0..=24.0,
            get_set(config.osc_count as f64, |it| {
                dispatch(Action::SetOscCount(it as u32))
            }),
        )
        .text("Osc count")
        .fixed_decimals(0),
    );
    knob(
        ui,
        "Osc detune",
        config.detune_cents,
        |it| dispatch(Action::SetDetuneCents(it)),
        FloatRange(0.0, 100.0),
    );

    egui::ComboBox::from_label("Anti aliasing mode")
        .selected_text(config.anti_aliasing_mode.to_string())
        .show_ui(ui, |ui| {
            for mode in AntiAliasingMode::iter() {
                selectable_value(
                    ui,
                    get_set(config.anti_aliasing_mode, |it| {
                        dispatch(Action::SetAntiAliasingMode(it))
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
                    dispatch(Action::SetOversampleFactor(it as u32))
                }),
            )
            .text("Oversample factor")
            .fixed_decimals(0),
        );
    }
}
