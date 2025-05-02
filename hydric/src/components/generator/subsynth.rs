use super::super::{ModMatrixView, Piano, PianoOrientation};
use super::subsynth_oscillator::SubSynthOscillatorView;
use crate::DataState;
use crate::view::View;
use crate::widget::{TabDisplay, TabOrientation};
use eframe::egui;
use egui::{Color32, Frame, Stroke, Ui, Vec2};
use lazy_static::lazy_static;
use shared::model::SubSynthConfig;
use shared::{
    model::{PitchName, ScaleValue},
    types::PitchValue,
};
use state::Action;

pub struct SubSynthView<'a, F: Fn(Action), G: Fn()> {
    config: &'a SubSynthConfig,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> SubSynthView<'a, F, G> {
    pub fn new(config: &'a SubSynthConfig, dispatch: F, on_release: G) -> Self {
        Self {
            config,
            dispatch,
            on_release,
        }
    }
}

const CHART_FILL_ALPHA: u8 = opacity_percentage_to_alpha(44.0);
const HORIZONTAL_SPACE: f32 = 3.0;

const fn opacity_percentage_to_alpha(opacity_percentage: f32) -> u8 {
    ((opacity_percentage / 100.0) * 255.0) as u8
}

lazy_static! {
    pub static ref GREEN_OUTLINE: Color32 = Color32::from_rgb(119, 167, 43);
    pub static ref GREEN_FILL: Color32 =
        Color32::from_rgba_unmultiplied(147, 175, 100, CHART_FILL_ALPHA);
    pub static ref PINK_OUTLINE: Color32 = Color32::from_rgb(246, 83, 192);
    pub static ref PINK_FILL: Color32 =
        Color32::from_rgba_unmultiplied(212, 132, 170, CHART_FILL_ALPHA);
    pub static ref ORANGE_OUTLINE: Color32 = Color32::from_rgb(227, 172, 84);
    pub static ref ORANGE_FILL: Color32 =
        Color32::from_rgba_unmultiplied(215, 171, 53, CHART_FILL_ALPHA);
    pub static ref LINE_COLOURS: [Color32; 3] = [*GREEN_OUTLINE, *PINK_OUTLINE, *ORANGE_OUTLINE];
    pub static ref FILL_COLOURS: [Color32; 3] = [*GREEN_FILL, *PINK_FILL, *ORANGE_FILL];
}

impl<F: Fn(Action), G: Fn()> View for SubSynthView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let config = self.config;
        let dispatch = &self.dispatch;
        let on_release = &self.on_release;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                for oscillator_id in 0..3 {
                    ui.push_id(oscillator_id, |ui| {
                        SubSynthOscillatorView::new(
                            &config.oscillators[oscillator_id],
                            &dispatch,
                            &on_release,
                            LINE_COLOURS[oscillator_id],
                            FILL_COLOURS[oscillator_id],
                        )
                        .ui(ui);
                        ui.add_space(4.0);
                    });
                }
            });

            ui.add_space(HORIZONTAL_SPACE);

            ui.vertical(|ui| {
                draw_lfos(ui);
            });

            ui.add_space(HORIZONTAL_SPACE);

            ui.vertical(|ui| {
                ModMatrixView::new(
                    &config.matrix,
                    vec!["ENV 1", "ENV 2", "ENV 3", "LFO 1", "LFO 2", "LFO 3"],
                    vec!["OSC 1", "OSC 2", "OSC 3"],
                    dispatch,
                    on_release,
                )
                .ui(ui); // must wrap in ui.vertical to stop the matrix from unnecessarily stretching vertically
            });
        });

        fn draw_lfos(ui: &mut Ui) {
            // TODO move to separate lfo file
            let handle_lfo_tab_click: Box<dyn Fn(&mut egui::Ui, usize) + Send + Sync + 'static> =
                Box::new(move |ui, index| {
                    DataState::SubSynthLfoTab.set_value(ui, index);
                });
            let active_lfo_tab = DataState::SubSynthLfoTab
                .get_value::<usize>(ui)
                .unwrap_or_default();

            let outer_frame = Frame::new()
                .fill(Color32::from_rgb(50, 50, 50))
                .stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 50)))
                .corner_radius(8.0)
                .inner_margin(6.0);
            outer_frame.show(ui, |ui| {
                let original_spacing = ui.spacing().item_spacing; // store original spacing
                ui.spacing_mut().item_spacing = Vec2::ZERO; // set spacing to zero so that the tabs and associated content actually touch each other

                ui.horizontal(|ui| {
                    TabDisplay::new(
                        active_lfo_tab,
                        vec!["LFO 1", "LFO 2", "LFO 3"],
                        TabOrientation::Left,
                        handle_lfo_tab_click,
                    )
                    .ui(ui);
                    let inner_frame = Frame::new()
                        .fill(Color32::from_rgb(30, 30, 30))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(30, 30, 30)))
                        .corner_radius(8.0)
                        .inner_margin(15.0);
                    inner_frame.show(ui, |ui| {
                        // TODO show the actual LFO controls depending on active_lfo_tab so everything in this block can be deleted it's just a placeholder to visualise the space
                        let size = egui::Vec2::new(200.0, 180.0);
                        let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
                        let painter = ui.painter_at(rect);
                        let rect_shape = egui::Shape::rect_filled(rect, 5.0, Color32::RED);
                        painter.add(rect_shape);
                    })
                });
                ui.spacing_mut().item_spacing = original_spacing; // reset ui spacing back to original
            });
        }

        fn draw_piano(ui: &mut Ui) {
            let min_note: PitchValue = PitchName {
                scale_value: ScaleValue::A,
                octave: 1,
            }
            .into();
            let max_note: PitchValue = PitchName {
                scale_value: ScaleValue::C,
                octave: 8,
            }
            .into();
            // TODO: piano is only rendering a subset of these notes.
            Piano::new(max_note, min_note)
                .with_orientation(PianoOrientation::Horizontal)
                .ui(ui);
        }

        draw_piano(ui);
    }
}
