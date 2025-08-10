use crate::view::View;
use crate::widget::{add_knob, get_set, selectable_value, styled_knob};
use crate::widget::FrequencyPlot;
use egui::{Ui, pos2};
use mesic::consts::NYQUIST;
use mesic::eq::eq_display::{FrequencyResponsePoint, calculate_frequency_response};
use mesic::eq::get_eq_filter_coeffs;
use shared::model::EqConfig;
use shared::model::EqType;
use state::{Action, FloatField, TypeField};
use strum::IntoEnumIterator;

pub struct EqView<'a, F: Fn(Action), G: Fn()> {
    config: &'a EqConfig,
    dispatch: F,
    on_release: G,
}

const MIN_FREQ: f32 = 1.0;
const MAX_FREQ: f32 = NYQUIST as f32;

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
        let num_points = 250;
        let eq_response_points: Vec<FrequencyResponsePoint> =
            calculate_frequency_response(&biquad_coeffs.unwrap(), num_points, MIN_FREQ, MAX_FREQ);
        eq_response_points
    }

}

impl<F: Fn(Action), G: Fn()> View for EqView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            config, dispatch, ..
        } = self;
        ui.horizontal(|ui| {
            add_knob(
                ui,
                styled_knob(
                    "Freq",
                    config.fc,
                    |it| dispatch(Action::SetFloat(FloatField::Fc, it)),
                    20.0..=20000.0,
                )
                .logarithmic(true)
                .with_neutral(2000.0),
                &self.on_release,
            );

            add_knob(
                ui,
                styled_knob(
                    "Q",
                    config.q,
                    |it| dispatch(Action::SetFloat(FloatField::Q, it)),
                    0.1..=100.0,
                )
                .logarithmic(true)
                .with_neutral(1.0),
                &self.on_release,
            );

            add_knob(
                ui,
                styled_knob(
                    "Gain",
                    config.gain,
                    |it| dispatch(Action::SetFloat(FloatField::Gain, it)),
                    -60.0..=60.0,
                )
                .with_neutral(0.0),
                &self.on_release,
            );
        });

        ui.add_space(5.0);

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

        let freq_plot_points: Vec<egui::Pos2> = self.calculate_eq_points()
            .into_iter()
            .map(|point| pos2(point.frequency, point.gain))
            .collect();

        ui.add_space(12.0);

        ui.add(FrequencyPlot::new(&freq_plot_points).logarithmic(true).set_y_range(std::ops::RangeInclusive::new(60.0, -60.0)));


    }
}
