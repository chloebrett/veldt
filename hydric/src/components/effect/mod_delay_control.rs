use crate::widget::{FloatRange, knob, selectable_value};
use egui::Ui;
use shared::model::{ModDelayConfig, WaveType};
use state::{Action, get_set};
use strum::IntoEnumIterator;

pub fn mod_delay_control<F>(config: &ModDelayConfig, dispatch: F, ui: &mut Ui)
where
    F: Fn(Action),
{
    // TODO: support integer knobs.
    // TODO: clamp the value within each frame to prevent min_depth from exceeding max_depth.
    knob(
        ui,
        "Min depth (samples)",
        config.min_depth as f32,
        |it| dispatch(Action::SetModDelayMinDepth(it as u32)),
        FloatRange(0.0, 1_000.0),
    );
    knob(
        ui,
        "Max depth (samples)",
        config.max_depth as f32,
        |it| dispatch(Action::SetModDelayMaxDepth(it as u32)),
        FloatRange(0.0, 1_000.0),
    );
    // TODO: replace this with a knob once we have logarithmic knobs.
    ui.add(
        egui::Slider::from_get_set(
            0.1..=100.0,
            get_set(config.freq as f64, |it| {
                dispatch(Action::SetModDelayLfoFreq(it as f32))
            }),
        )
        .text("LFO frequency")
        .logarithmic(true),
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
