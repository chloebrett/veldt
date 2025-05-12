use crate::view::View;
use crate::widget::{get_set, knob, selectable_value};
use egui::{Color32, Ui};
use egui_plot::{Line, Plot, PlotPoints};
use mesic::SAMPLE_RATE;
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
        let min_freq = 20.0;
        let max_freq = SAMPLE_RATE as f32 / 2.0;
        let eq_response_points: Vec<FrequencyResponsePoint> =
            calculate_frequency_response(&biquad_coeffs.unwrap(), num_points, min_freq, max_freq);
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

        ui.add_space(5.0);

        let frequency_points = self.calculate_eq_points();
        let mut plot_shapes = vec![];
        let eq_plot_points: PlotPoints = frequency_points
            .into_iter()
            .map(|point| [point.frequency as f64, point.gain as f64])
            .collect();
        plot_shapes.push(Line::new("Response", eq_plot_points).color(Color32::WHITE));

        Plot::new("Frequency Response")
            .view_aspect(2.0)
            .default_x_bounds(0.0, SAMPLE_RATE as f64 / 2.0)
            .default_y_bounds(-20.0, 20.0)
            .allow_drag(false)
            .x_axis_label("Frequency (Hz)")
            .y_axis_label("Response (dB)")
            .show(ui, |plot_ui| {
                for shape in plot_shapes {
                    plot_ui.line(shape)
                }
            });
    }
}
