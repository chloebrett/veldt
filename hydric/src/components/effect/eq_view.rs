use crate::view::View;
use crate::widget::{get_set, knob, selectable_value};
use egui::Ui;
use shared::model::EqConfig;
use shared::model::EqType;
use state::{Action, FloatField, TypeField};
use strum::IntoEnumIterator;
use crate::components::play::FrequencyDisplay;
use mesic::eq::{get_eq_filter_coeffs, calculate_frequency_response, FrequencyResponsePoint};
use mesic::SAMPLE_RATE;

pub struct EqView<'a, F: Fn(Action), G: Fn()> {
    config: &'a EqConfig,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> EqView<'a, F, G> {
    pub fn new(config: &'a EqConfig, dispatch: F, on_release: G) -> Self {
        Self {
            config,
            dispatch,
            on_release,
        }
    }

    fn calculate_eq_points(&self) -> Vec<FrequencyResponsePoint> {
        let biquad_coeffs = get_eq_filter_coeffs(self.config);
        let eq_response_points: Vec<FrequencyResponsePoint> = calculate_frequency_response(&biquad_coeffs.unwrap(), 250, 20.0, SAMPLE_RATE as f32 / 2.0);
        eq_response_points
    }
}

impl<F: Fn(Action), G: Fn()> View for EqView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            config, dispatch, ..
        } = self;
        ui.horizontal(|ui| {
            knob(
                ui,
                "Freq",
                config.fc,
                |it| dispatch(Action::SetFloat(FloatField::Fc, it)),
                20.0..=20000.0, // TODO: logarithmic
                /* neutral= */ 2000.0,
                &self.on_release,
            );
    
            knob(
                ui,
                "Q",
                config.q,
                |it| dispatch(Action::SetFloat(FloatField::Q, it)),
                0.1..=100.0, // TODO: logarithmic
                /* neutral= */ 1.0,
                &self.on_release,
            );
    
            knob(
                ui,
                "Gain",
                config.gain,
                |it| dispatch(Action::SetFloat(FloatField::Gain, it)),
                -60.0..=60.0,
                /* neutral= */ 0.0,
                &self.on_release,
            );
        });

        let eq_type = config.kind.clone();
        egui::ComboBox::from_label("EQ type")
            .selected_text(eq_type.to_string())
            .show_ui(ui, |ui| {
                for eq_type in EqType::iter() {
                    selectable_value(
                        ui,
                        get_set(config.kind.clone(), |it| {
                            dispatch(Action::SetChild(TypeField::EqType(it)))
                        }),
                        eq_type.clone(),
                        eq_type.to_string(),
                    );
                }
            });
        
        let frequency_points = self.calculate_eq_points();
        FrequencyDisplay::new(None, Some(frequency_points), [-15.0, 15.0]).ui(ui);
    }
}
