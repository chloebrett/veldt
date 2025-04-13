use crate::widget::{get_set, int_slider, knob, selectable_value};
use egui::Ui;
use shared::model::{AntiAliasingMode, SimpleWaveConfig, WaveType};
use state::Action;
use strum::IntoEnumIterator;

pub fn simple_wave_control<F, G>(config: &SimpleWaveConfig, dispatch: F, on_release: G, ui: &mut Ui)
where
    F: Fn(Action),
    G: Fn(),
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

    int_slider(
        ui,
        "Unison",
        config.osc_count as f64,
        |it| dispatch(Action::SetOscCount(it as u32)),
        1..=24,
        &on_release,
    );
    knob(
        ui,
        "Osc detune",
        config.detune_cents,
        |it| dispatch(Action::SetDetuneCents(it)),
        0.0..=100.0,
        &on_release,
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
        int_slider(
            ui,
            "Oversample factor",
            config.oversample_factor as f64,
            |it| dispatch(Action::SetOversampleFactor(it as u32)),
            2..=10,
            &on_release,
        );
    }
}
