use crate::view::View;
use crate::widget::{
    TextRotation, add_disabled_knob, add_typable_knob, disabled_knob, for_each_with_separator,
    styled_knob, text_rotator,
};
use eframe::egui;
use egui::{Color32, Ui};
use shared::model::{MatrixCell, MixerMatrix};
use state::{Action, FloatField, MixerMatrixCellSelector, Store};

pub struct MixerMatrixView<'a, G: Fn()> {
    matrix: &'a MixerMatrix,
    row_titles: Vec<String>,
    col_titles: Vec<String>,
    store: &'a Store,
    on_release: G,
}

impl<'a, G: Fn()> MixerMatrixView<'a, G> {
    pub fn new(
        matrix: &'a MixerMatrix,
        row_titles: Vec<String>,
        col_titles: Vec<String>,
        store: &'a Store,
        on_release: G,
    ) -> Self {
        MixerMatrixView {
            matrix,
            row_titles,
            col_titles,
            store,
            on_release,
        }
    }
}

impl<G: Fn()> View for MixerMatrixView<'_, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            matrix,
            ref row_titles,
            ref col_titles,
            store,
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
                        // Skip the first row (main channel in).
                        // The main channel can't be directed into any other channels.
                        if row == 0 {
                            return;
                        }

                        ui.horizontal(|ui| {
                            text_rotator(
                                ui,
                                row_title,
                                12.0,
                                TextRotation::Anticlockwise90,
                                TEXT_COLOUR,
                            );
                            for col in 0..matrix.channels {
                                let id = format!("mixer_matrix_{}_{}", row, col);

                                let sel = MixerMatrixCellSelector(row, col);
                                let value: &MatrixCell = store.select(&sel);
                                let value: f32 = (*value).into();

                                let transpose_sel = MixerMatrixCellSelector(col, row); // flipped!
                                let transpose_value: &MatrixCell = store.select(&transpose_sel);
                                let transpose_value: f32 = (*transpose_value).into();

                                // Don't allow the mixer knob to be non-zero when:
                                // (a) the row == col (self cycle)
                                // (b) the transpose (swap row/col) value is non-zero (direct cycle).
                                // TODO: also consider transitive cycles!
                                let disabled = (row == col) || (transpose_value != 0.0);

                                let disabled_knob = disabled_knob(0.0, 0.0..=1.0);
                                let enabled_knob = styled_knob(
                                    value,
                                    |it| {
                                        store.dispatch(
                                            &sel,
                                            Action::SetFloat(FloatField::ModFactor, it),
                                        )
                                    },
                                    0.0..=1.0,
                                )
                                .with_neutral(0.0);
                                ui.push_id(id, |ui| {
                                    if disabled {
                                        add_disabled_knob(
                                            ui,
                                            disabled_knob,
                                            "",
                                            0.0,
                                            0.0..=1.0,
                                            on_release,
                                            40.0,
                                        );
                                    } else {
                                        add_typable_knob(
                                            ui,
                                            enabled_knob,
                                            "",
                                            value,
                                            |it| {
                                                store.dispatch(
                                                    &sel,
                                                    Action::SetFloat(FloatField::ModFactor, it),
                                                )
                                            },
                                            0.0..=1.0,
                                            on_release,
                                            40.0,
                                        );
                                    }
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
