use crate::view::View;
use crate::widget::{TextRotation, add_typable_knob, for_each_with_separator, styled_knob, text_rotator};
use eframe::egui;
use egui::{Color32, Stroke, Ui};
use shared::model::{MatrixCell, ModMatrix};
use state::{Action, FloatField, GeneratorSelector, Store};

pub struct ModMatrixView<'a, G: Fn()> {
    matrix: &'a ModMatrix,
    row_titles: Vec<&'a str>,
    col_titles: Vec<&'a str>,
    store: &'a Store,
    generator_sel: &'a GeneratorSelector,
    on_release: G,
}

impl<'a, G: Fn()> ModMatrixView<'a, G> {
    pub fn new(
        matrix: &'a ModMatrix,
        row_titles: Vec<&'a str>,
        col_titles: Vec<&'a str>,
        store: &'a Store,
        generator_sel: &'a GeneratorSelector,
        on_release: G,
    ) -> Self {
        ModMatrixView {
            matrix,
            row_titles,
            col_titles,
            store,
            generator_sel,
            on_release,
        }
    }
}

impl<G: Fn()> View for ModMatrixView<'_, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            matrix,
            ref row_titles,
            ref col_titles,
            store,
            generator_sel,
            ref on_release,
        } = *self;

        const TEXT_COLOUR: Color32 = Color32::from_gray(180);

        let frame = egui::Frame::new()
            .fill(Color32::from_gray(50))
            .stroke(Stroke::new(1.0, Color32::from_gray(60)))
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
                            for col in 0..matrix.cols as usize {
                                let id = format!("{:?}", (row, col));
                                let cell_sel = generator_sel.downcast_mod_matrix_cell(row, col);
                                let value: &MatrixCell = store.select(&cell_sel);
                                let value = **value;
                                let cell_knob = styled_knob(
                                            value,
                                            |it| {
                                                store.dispatch(
                                                    &cell_sel,
                                                    Action::SetFloat(FloatField::ModFactor, it),
                                                );
                                            },
                                            0.0..=1.0,
                                        )
                                        .with_neutral(0.0);

                                ui.push_id(id, |ui| {
                                    add_typable_knob(
                                        ui,
                                        cell_knob,
                                        "",
                                        value,
                                        |it| {
                                            store.dispatch(
                                                &cell_sel,
                                                Action::SetFloat(FloatField::ModFactor, it),
                                            );
                                        },
                                        0.0..=1.0,
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
