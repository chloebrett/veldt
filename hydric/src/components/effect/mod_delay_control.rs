use crate::widget::{knob, log_slider, selectable_value};
use egui::Ui;
use shared::model::{ModDelayConfig, WaveType};
use shared::state::Action;
use state::get_set;
use strum::IntoEnumIterator;

pub fn mod_delay_control<F, G>(config: &ModDelayConfig, dispatch: F, on_release: G, ui: &mut Ui)
where
    F: Fn(Action),
    G: Fn(),
{
    // TODO: support integer knobs.
    // TODO: clamp the value within each frame to prevent min_depth from exceeding max_depth.
    knob(
        ui,
        "Min depth (samples)",
        config.min_depth as f32,
        |it| dispatch(Action::SetModDelayMinDepth(it as u32)),
        0.0..=1_000.0,
        &on_release,
    );
    knob(
        ui,
        "Max depth (samples)",
        config.max_depth as f32,
        |it| dispatch(Action::SetModDelayMaxDepth(it as u32)),
        0.0..=1_000.0,
        &on_release,
    );
    // TODO: replace this with a knob once we have logarithmic knobs.
    log_slider(
        ui,
        "LFO frequency",
        config.freq as f64,
        |it| dispatch(Action::SetModDelayLfoFreq(it as f32)),
        0.1..=100.0,
        &on_release,
    );

    egui::ComboBox::from_label("LFO wave type")
        .selected_text(config.lfo_type.to_string())
        .show_ui(ui, |ui| {
            for wave in WaveType::iter() {
                selectable_value(
                    ui,
                    get_set(config.lfo_type, |it| {
                        dispatch(Action::SetModDelayLfoType(it))
                    }),
                    wave,
                    wave.to_string(),
                );
            }
        });
}
