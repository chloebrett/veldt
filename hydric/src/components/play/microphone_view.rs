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
    mic_sample_name: &'a mut String,
}

impl<'a> MicrophoneView<'a> {
    pub fn new(
        local_state: &'a LocalState,
        async_state: &'a mut AsyncState,
        app_mic: &'a mut Microphone,
        mic_sample_name: &'a mut String,
    ) -> Self {
        MicrophoneView {
            local_state,
            async_state,
            mic: app_mic,
            mic_sample_name,
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
                    if ui.button("Process Sample").clicked() {
                        let _ = self.mic._convert_audio();
                    }
                });
                
                ui.horizontal(|ui| {
            
                    // Text input field for file name.
                    ui.text_edit_singleline(self.mic_sample_name);

                    if ui.button("Save Sample").clicked(){

                        let file_data = self.mic.get_sample_bytes();
                        let mut mic_sample_name_copy = self.mic_sample_name.clone();

                        // SQL injections are scary.
                        if !mic_sample_name_copy.chars().all(|x| x.is_alphanumeric()){
                            error!("No Special Characters thank you.");
                            return;
                        } else {
                            // Add file extension. Hardcoded for now, while ogg is the only compatible option.
                            mic_sample_name_copy = mic_sample_name_copy + ".ogg";
                            
                            // call hydric upload method w bytes + file name
                            spawn(&mut self.async_state.upload_mic_sample, async move {
                                // Upload our sample
                                let result =
                                    upload_sample(mic_sample_name_copy, file_data).await;
                                if let Err(ref e) = result {
                                    error!("[5] Upload failed: {:?}", e);
                                }
                                result
                            });
                        }

                    }
                });
            },
        );
    }
}
