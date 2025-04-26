use crate::widget::{knob, text_rotator, TextRotation};
use eframe::egui;
use egui::{Color32, Ui};
use shared::model::ModMatrix;
use state::{Action, FloatField};
use wasm_bindgen_futures::js_sys::Array;

pub fn mod_matrix<F, G>(
    config: &ModMatrix,
    ui: &mut egui::Ui,
    row_titles: Vec<&str>,
    col_title: Vec<&str>,
    dispatch: &F,
    on_release: &G,
) -> egui::Response
where
    F: Fn(Action),
    G: Fn(),
{
        let frame = egui::Frame::new()
        .fill(Color32::from_rgb(50, 50, 50))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(60, 60, 60)))
        .corner_radius(8.0)
        .inner_margin(6.0);

        frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for col in 0..config.cols {
                        ui.label(col_title[col as usize]);
                        if col != config.cols - 1 {
                            ui.add_space(10.0);
                        }
                    }
                });
                ui.vertical(|ui| {
                    for row in 0..config.rows {
                        ui.horizontal(|ui| {
                            text_rotator(ui, row_titles[row as usize], 14.0, TextRotation::Anticlockwise);
                            for col in 0..config.cols {
                                let id = format!("{:?}", (row, col));
                                ui.push_id(id, |ui| {
                                    knob(
                                        ui,
                                        "Placeholder",
                                        0.0,
                                        |it| dispatch(Action::SetFloat(FloatField::Volume, it)),
                                        -1.0..=1.0,
                                        0.0,
                                        &on_release,
                                    );
                                });
                                if col != config.cols - 1 {
                                    ui.add_space(10.0);
                                }
                            }
                        });
                        if row != config.rows - 1 {
                            ui.add_space(10.0)
                        }
                    }
                });

            })
        .response
    
}