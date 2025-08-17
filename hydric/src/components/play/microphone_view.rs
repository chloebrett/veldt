use crate::AsyncState;
use crate::local_state::LocalState;
use crate::playback::Microphone;
use crate::promise::spawn;
use crate::rpc::upload_sample;
use crate::view::View;
use crate::widget::StateWindow;
use crate::window_state::WindowKind;
use egui::Ui;
use log::error;

pub struct MicrophoneView<'a> {
    local_state: &'a LocalState,
    async_state: &'a mut AsyncState,
    mic: &'a mut Microphone,
}

impl<'a> MicrophoneView<'a> {
    pub fn new(
        local_state: &'a LocalState,
        async_state: &'a mut AsyncState,
        app_mic: &'a mut Microphone,
    ) -> Self {
        MicrophoneView {
            local_state,
            async_state,
            mic: app_mic,
        }
    }
}

impl View for MicrophoneView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        StateWindow::show_from_window_state(
            ui,
            &self.local_state.window_state,
            WindowKind::Microphone,
            "Microphone",
            |ui| {
                if ui.button("Permissions").clicked() {
                    let _ = self.mic.get_permissions();
                }

                if self.mic.has_permissions() {
                    ui.label("Has permissions");
                }

                if ui.button("Record").clicked()
                    && !self.mic.is_recording()
                    && !self.mic.has_recording()
                {
                    let _ = self.mic.start();
                }

                if ui.button("Stop Recording").clicked() {
                    self.mic.stop();
                }

                ui.horizontal(|ui| {
                    if ui.button("Play mic Audio").clicked() {
                        let _ = self.mic.play_mic_audio();
                    }

                    if ui.button("pause").clicked() {
                        let _ = self.mic.pause_mic_audio();
                    }

                    if ui.button("stop").clicked() {
                        let _ = self.mic.stop_mic_audio();
                    }

                    if ui.button("clear").clicked() {
                        let _ = self.mic.clear_mic();
                    }
                });

                ui.horizontal(|ui| {
                    // text input
                    //let mut text = String::new();
                    //let mut output = egui::TextEdit::singleline(&mut text).show(ui);

                    let mut file_data: Option<Vec<u8>> = None;
                    if ui.button("Process Sample").clicked() {
                        let _ = self.mic._convert_audio();
                        file_data = Some(self.mic.get_sample_bytes());
                    }

                    if ui.button("Save Sample").clicked() && !file_data.is_none() {
                        // read text input

                        // call convert_audio to get bytes (MAY NEED TO AWAIT)

                        // call hydric upload method w bytes + file name
                        spawn(&mut self.async_state.upload_mic_sample, async move {
                            // Upload our sample
                            let result =
                                upload_sample("test.ogg".to_string(), file_data.unwrap()).await;
                            if let Err(ref e) = result {
                                error!("[5] Upload failed: {:?}", e);
                            }
                            result
                        });
                    }
                });
            },
        );
    }
}
