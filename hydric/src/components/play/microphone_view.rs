use crate::local_state::LocalState;
use crate::playback::Microphone;
use crate::view::View;
use crate::widget::StateWindow;
use crate::window_state::WindowKind;
use egui::Ui;

pub struct MicrophoneView<'a> {
    local_state: &'a LocalState,
    mic: &'a mut Microphone,
}

impl<'a> MicrophoneView<'a> {
    pub fn new(local_state: &'a LocalState, app_mic: &'a mut Microphone) -> Self {
        MicrophoneView {
            local_state,
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
            },
        );
    }
}
