use crate::view::View;
use crate::widget::{add_typable_knob, get_set, selectable_value, styled_knob};
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
        // Currently this is only clamped the following frame, which looks janky.
        let min_depth_knob = styled_knob(
            config.min_depth as f32,
            |it| dispatch(Action::SetUint(UintField::MinDepth, it as u32)),
            0.0..=1_000.0,
        )
        .with_neutral(100.0)
        .with_step(1.0);
        let max_depth_knob = styled_knob(
            config.max_depth as f32,
            |it| dispatch(Action::SetUint(UintField::MaxDepth, it as u32)),
            0.0..=1_000.0,
        )
        .with_neutral(200.0)
        .with_step(1.0);
        let lfo_knob = styled_knob(
            config.freq,
            |it| dispatch(Action::SetFloat(FloatField::LfoFreq, it)),
            0.1..=100.0,
        )
        .logarithmic(true)
        .with_neutral(1.0);
        add_typable_knob(
            ui,
            min_depth_knob,
            "Min depth (samples)",
            config.min_depth as f32,
            |it| dispatch(Action::SetUint(UintField::MinDepth, it as u32)),
            0.0..=1_000.0,
            &self.on_release,
            50.0,
        );
        add_typable_knob(
            ui,
            max_depth_knob,
            "Max depth (samples)",
            config.max_depth as f32,
            |it| dispatch(Action::SetUint(UintField::MaxDepth, it as u32)),
            0.0..=1_000.0,
            &self.on_release,
            50.0,
        );
        add_typable_knob(
            ui,
            lfo_knob,
            "LFO frequency (Hz)",
            config.freq,
            |it| dispatch(Action::SetFloat(FloatField::LfoFreq, it)),
            0.1..=100.0,
            &self.on_release,
            40.0,
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
