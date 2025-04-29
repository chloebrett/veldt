use crate::view::View;
use crate::widget::{get_set, knob, log_slider, selectable_value};
use egui::Ui;
use shared::model::{ModDelayConfig, WaveType};
use state::{Action, FloatField, TypeField, UintField};
use strum::IntoEnumIterator;

pub struct ModDelayView<'a, F: Fn(Action), G: Fn()> {
    config: &'a ModDelayConfig,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> ModDelayView<'a, F, G> {
    pub fn new(config: &'a ModDelayConfig, dispatch: F, on_release: G) -> Self {
        Self {
            config,
            dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn()> View for ModDelayView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            config, dispatch, ..
        } = self;

        // TODO: support integer knobs.
        // TODO: clamp the value within each frame to prevent min_depth from exceeding max_depth.
        knob(
            ui,
            "Min depth (samples)",
            config.min_depth as f32,
            |it| dispatch(Action::SetUint(UintField::MinDepth, it as u32)),
            0.0..=1_000.0,
            /* neutral= */ 100.0,
            &self.on_release,
        );
        knob(
            ui,
            "Max depth (samples)",
            config.max_depth as f32,
            |it| dispatch(Action::SetUint(UintField::MaxDepth, it as u32)),
            0.0..=1_000.0,
            /* neutral= */ 200.0,
            &self.on_release,
        );
        // TODO: replace this with a knob once we have logarithmic knobs.
        log_slider(
            ui,
            "LFO frequency (Hz)",
            config.freq as f64,
            |it| dispatch(Action::SetFloat(FloatField::LfoFreq, it as f32)),
            0.1..=100.0,
            &self.on_release,
        );

        egui::ComboBox::from_label("LFO wave type")
            .selected_text(config.lfo_type.to_string())
            .show_ui(ui, |ui| {
                for wave in WaveType::iter() {
                    selectable_value(
                        ui,
                        get_set(config.lfo_type, |it| {
                            dispatch(Action::SetChild(TypeField::Wave(it)))
                        }),
                        wave,
                        wave.to_string(),
                    );
                }
            });
    }
}
