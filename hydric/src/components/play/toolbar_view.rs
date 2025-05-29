use crate::promise::{poll, spawn};
use crate::rpc::{load_sample, upload_sample};
use crate::view::View;
use crate::widget::{default_window, knob, slider};
use crate::{AsyncState, playback::AudioPlayer};
use egui::{Pos2, Ui};
use log::error;
use shared::types::Beats;
use state::{Action, FloatField, Store, TypeField};
use tonic::Status;

use super::play_control::*;

pub struct ToolbarView<'a> {
    store: &'a mut Store,
    async_state: &'a mut AsyncState,
    player: &'a mut AudioPlayer,
}

impl<'a> ToolbarView<'a> {
    pub fn new(
        store: &'a mut Store,
        async_state: &'a mut AsyncState,
        player: &'a mut AudioPlayer,
    ) -> Self {
        ToolbarView {
            store,
            async_state,
            player,
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
                play_control(self.store, self.async_state, self.player, ui);
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Upload Sample").clicked() {
                        /*
                        In future it is worth considering extending the async_state expected result to handle
                        current upload progress or errors.
                        */
                        spawn(&mut self.async_state.upload_sample, async move {
                            let Some(file) = rfd::AsyncFileDialog::new()
                                .add_filter("Sound Sample", &["wav", "mp3"])
                                .pick_file()
                                .await
                            else {
                                return Err(Status::aborted("User cancelled upload"));
                            };

                            let file_data = file.read().await;

                            // Upload our sample
                            let result = upload_sample(file.file_name(), file_data).await;
                            if let Err(ref e) = result {
                                error!("[5] Upload failed: {:?}", e);
                            }
                            result
                        });

                        ui.close_menu();
                    }
                })
            });

        poll(&mut self.async_state.upload_sample, |file_name| {
            if !file_name.is_empty() {
                let file_name = file_name.clone();
                // immediately load sample upon upload to avoid async mess if trying to preview
                spawn(&mut self.async_state.load_sample, async move {
                    load_sample(file_name).await
                });
            }
        });

        poll(&mut self.async_state.load_sample, |sample| {
            self.store
                .dispatchr(Action::AddChild(TypeField::Sample(sample.clone())));
        });
    }
}
