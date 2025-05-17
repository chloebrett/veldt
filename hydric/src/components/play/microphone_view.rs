use crate::AsyncState;
use crate::playback::Microphone;
use crate::view::View;
use crate::widget::default_window;
use egui::{Pos2, Ui};
use log::error;
use state::Store;

pub struct MicrophoneView<'a> {
    store: &'a Store,
    async_state: &'a mut AsyncState,
    visible: &'a mut bool,
    mic: &'a mut Microphone,
}

impl<'a> MicrophoneView<'a> {
    pub fn new(
        store: &'a Store,
        async_state: &'a mut AsyncState,
        visible: &'a mut bool,
        app_mic: &'a mut Microphone,
    ) -> Self {
        MicrophoneView {
            store,
            async_state,
            visible,
            mic: app_mic,
        }
    }
}

impl View for MicrophoneView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        default_window("Microphone")
            .resizable(true)
            .open(self.visible)
            .default_pos(Pos2 { x: 600.0, y: 20.0 })
            .show(ui.ctx(), |ui| {
                if ui.button("Permissions").clicked() {
                    let _ = self.mic.get_permissions();
                }

                if self.mic.has_permissions() {
                    ui.label("Has permissions");
                }

                if ui.button("Record").clicked() {
                    self.mic.start();
                }

                if ui.button("Stop").clicked() {
                    self.mic.stop();
                }

                ui.label(format!("Blob count: {}", self.mic.blob_count()));

                if let Some(recording) = self.mic.recording() {
                    ui.label(format!("Recording exists: {}", recording.len()));
                }
            });
    }
}
