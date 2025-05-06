use crate::view::View;
use crate::widget::{TextRotation, for_each_with_separator, knob, text_rotator};
use eframe::egui;
use egui::{Color32, Ui};
use shared::model::ModMatrix;
use state::{Action, FloatField};

pub struct ModMatrixView<'a, F: Fn(Action), G: Fn()> {
    matrix: &'a ModMatrix,
    row_titles: Vec<&'a str>,
    col_titles: Vec<&'a str>,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> ModMatrixView<'a, F, G> {
    pub fn new(
        matrix: &'a ModMatrix,
        row_titles: Vec<&'a str>,
        col_titles: Vec<&'a str>,
        dispatch: F,
        on_release: G,
    ) -> Self {
        ModMatrixView {
            matrix,
            row_titles,
            col_titles,
            dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn()> View for ModMatrixView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            matrix,
            ref row_titles,
            ref col_titles,
            ref dispatch,
            ref on_release,
        } = *self;

        const TEXT_COLOUR: Color32 = Color32::from_gray(180);

        let frame = egui::Frame::new()
            .fill(Color32::from_rgb(50, 50, 50))
            .stroke(egui::Stroke::new(1.0, Color32::from_gray(60)))
            .corner_radius(8.0)
            .inner_margin(6.0);

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(32.0);
                for_each_with_separator(
                    ui,
                    col_titles,
                    |ui, col_title| {
                        text_rotator(ui, col_title, 12.0, TextRotation::Neutral, TEXT_COLOUR);
                    },
                    |ui| {
                        ui.add_space(36.0);
                    },
                );
            });
            ui.vertical(|ui| {
                for_each_with_separator(
                    ui,
                    row_titles.iter().enumerate(),
                    |ui, (row, row_title)| {
                        ui.horizontal(|ui| {
                            text_rotator(
                                ui,
                                row_title,
                                12.0,
                                TextRotation::Anticlockwise90,
                                TEXT_COLOUR,
                            );
                            for col in 0..matrix.cols {
                                let id = format!("{:?}", (row, col));
                                ui.push_id(id, |ui| {
                                    knob(
                                        ui,
                                        "",
                                        0.0,
                                        |it| dispatch(Action::SetFloat(FloatField::ModFactor, it)), // TODO need a selector for each knob
                                        -1.0..=1.0,
                                        0.0,
                                        on_release,
                                    );
                                });
                            }
                        });
                    },
                    |ui| {
                        ui.add_space(5.0);
                    },
                );
            });
        });
    }
}
