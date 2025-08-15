use crate::view::View;
use crate::widget::{add_knob, get_set, selectable_value, styled_knob};
use egui::{Color32, Rect, Sense, Shape, Stroke, Ui, Vec2, pos2};
use egui_plot::{Line, Plot, PlotPoints};
use mesic::consts::NYQUIST;
use mesic::eq::eq_display::{FrequencyResponsePoint, calculate_frequency_response};
use mesic::eq::get_eq_filter_coeffs;
use mesic::ilerp;
use shared::model::EqConfig;
use shared::model::EqType;
use state::{Action, FloatField, TypeField};
use strum::IntoEnumIterator;

pub struct EqView<'a, F: Fn(Action), G: Fn()> {
    config: &'a EqConfig,
    dispatch: F,
    on_release: G,
}

const MIN_FREQ: f32 = 20.0;
const MAX_FREQ: f32 = NYQUIST as f32;
const MIN_GAIN: f64 = -25.0;
const MAX_GAIN: f64 = 25.0;

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

    fn draw_logarithmic_response(
        &self,
        frequency_points: Vec<FrequencyResponsePoint>,
        plot_rect: Rect,
    ) -> Shape {
        let plot_x_min = plot_rect.left_top().x;
        let plot_y_min = plot_rect.left_top().y;
        let plot_x_max = plot_rect.right_bottom().x;
        let plot_y_max = plot_rect.right_bottom().y;

        let log_min_freq = MIN_FREQ.log10();
        let log_max_freq = MAX_FREQ.log10();
        let eq_points = frequency_points
            .into_iter()
            .map(|point| {
                // x position calculations
                let log_current_freq = point.frequency.log10();
                let normalised_freq_pos = ilerp(log_min_freq, log_max_freq, log_current_freq);
                let x_position = plot_x_min + normalised_freq_pos * (plot_x_max - plot_x_min);

                // y position calculations
                let normalised_gain_pos = ilerp(MIN_GAIN as f32, MAX_GAIN as f32, point.gain);
                let y_position = plot_y_max - normalised_gain_pos * (plot_y_max - plot_y_min);
                pos2(x_position, y_position)
            })
            .collect();

        Shape::line(eq_points, Stroke::new(1.0, Color32::WHITE))
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

        let frequency_points = self.calculate_eq_points();

        // for displaying chart on 'log scale'
        // TODO: switch to plot with log x axis when implemented OR add grid lines, prevent overflow, add axis labels
        ui.label("Frequency - Response plot on logged x-axis");
        ui.add_space(50.0);
        let (rect, _response) = ui.allocate_exact_size(Vec2::new(300.0, 80.0), Sense::empty());
        let logged_freq_response_shape =
            self.draw_logarithmic_response(frequency_points.clone(), rect);
        let painter = ui.painter();
        painter.add(logged_freq_response_shape);

        ui.add_space(50.0);

        // for displaying chart on plot
        ui.label("Frequency - Response plot on linear x-axis");
        let mut plot_shapes = vec![];
        let eq_plot_points: PlotPoints = frequency_points
            .into_iter()
            .map(|point| [point.frequency as f64, point.gain as f64])
            .collect();
        plot_shapes.push(Line::new("Response", eq_plot_points).color(Color32::WHITE));

        Plot::new("Frequency Response")
            .view_aspect(2.0)
            .default_x_bounds(0.0, NYQUIST as f64)
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
