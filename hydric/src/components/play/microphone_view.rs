use crate::AsyncState;
use crate::playback::Microphone;
use crate::promise::spawn;
use crate::rpc::upload_sample;
use crate::view::View;
use crate::widget::default_window;
use egui::{Pos2, Ui};
use log::error;
use state::Store;

pub struct MicrophoneView<'a> {
    store: &'a Store,
    async_state: &'a mut AsyncState,
    visible: &'a mut bool,
    audio_data: Option<Vec<u8>>,
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
            audio_data: None,
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
                // Issue here is with self reference to mic. While mic instance now lives in audio_state in app, we can
                // still only pass it in as an input to MicrophoneView::new(). Which still requires the use of self
                if ui.button("Record").clicked() {
                    spawn(&mut self.async_state.microphone_start, async move {
                        self.mic.start().await
                    });
                }

                if ui.button("Stop").clicked() {
                    spawn(&mut self.async_state.microphone_stop, async move {
                        self.mic.stop().await
                    });
                }

                // If we have stopped, extract the recording.
                if let Some(promise) = self.async_state.microphone_stop.take() {
                    // Check if promise is resolved.
                    if let Some(result) = promise.ready() {
                        match result {
                            Ok(audio_data) => {
                                // Store temporarily in the view struct
                                self.audio_data = Some(audio_data.to_vec());
                            }
                            Err(e) => error!("Recording failed: {:?}", e),
                        }
                    } else {
                        // Promise not ready yet, put it back.
                        self.async_state.microphone_stop = Some(promise);
                    }
                }

                // Upload data to backend, this also clears the audio_data variable
                if let Some(data) = self.audio_data.take() {
                    if ui.button("Upload").clicked() {
                        let file_name = "recording.wav".to_string(); //Need a way to get custom url
                        let data = data.clone();

                        spawn(&mut self.async_state.upload_sample, async move {
                            upload_sample(file_name, data).await.map_err(|e| {
                                error!("Upload failed: {}", e);
                                ()
                            })
                        });
                    }
                }
            });
    }
}
