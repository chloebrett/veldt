use crate::app_state::{AsyncState, AudioState};
use crate::promise::spawn;
use crate::rpc::upload_sample;
use crate::view::View;
use crate::widget::{default_window, knob, slider};
use egui::{Pos2, Ui};
use shared::types::Beats;
use state::{Action, FloatField, Store};

use super::{play_control::*, sample_control::*};

pub struct ToolbarView<'a> {
    store: &'a mut Store,
    async_state: &'a mut AsyncState,
    audio_state: &'a mut AudioState,
}

impl<'a> ToolbarView<'a> {
    pub fn new(
        store: &'a mut Store,
        async_state: &'a mut AsyncState,
        audio_state: &'a mut AudioState,
    ) -> Self {
        ToolbarView {
            store,
            async_state,
            audio_state,
        }
    }
}

impl View for ToolbarView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        default_window("Toolbar")
            .default_pos(Pos2 { x: 600.0, y: 20.0 })
            .show(ui.ctx(), |ui| {
                let on_release = || self.store.dispatchr(Action::Release);
                ui.horizontal(|ui| {
                    let volume = self.store.get().volume;
                    knob(
                        ui,
                        "Volume",
                        volume,
                        |it| {
                            self.store
                                .dispatchr(Action::SetFloat(FloatField::Volume, it))
                        },
                        0.0..=1.0,
                        /* neutral= */ 1.0,
                        on_release,
                    );

                    let bpm = self.store.get().project.bpm as f64;
                    slider(
                        ui,
                        "BPM",
                        bpm,
                        |it| {
                            self.store
                                .dispatchr(Action::SetFloat(FloatField::Bpm, it as Beats))
                        },
                        20.0..=200.0,
                        on_release,
                    );
                });
                ui.separator();
                play_control(self.store, self.async_state, self.audio_state, ui);
                ui.separator();

                ui.horizontal(|ui| {
                    sample_control(self.store, self.audio_state, self.async_state, ui);
                    if ui.button("Upload Sample").clicked() {
                        /*
                        In future it is worth considering extending the async_state expected result to handle
                        current upload progress or errors.
                        */
                        spawn(&mut self.async_state.upload_sample, async move {
                            let Some(file) = rfd::AsyncFileDialog::new()
                                .add_filter("Sound Sample", &["wav"])
                                .pick_file()
                                .await
                            else {
                                // No proper error handling as a user canceling the action is typical.
                                return Ok(());
                            };

                            let file_name = file.file_name();
                            let file_data = file.read().await;
                            upload_sample(file_name, file_data).await
                        });

                        ui.close_menu();
                    }
                })
            });
    }
}
