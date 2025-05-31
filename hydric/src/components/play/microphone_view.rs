use crate::view::View;
use crate::widget::{StateWindow, default_window};
use crate::window_state::WindowKind;
use crate::{playback::Microphone, window_state::WindowState};
use egui::Ui;

pub struct MicrophoneView<'a> {
    window_state: &'a WindowState,
    mic: &'a mut Microphone,
}

impl<'a> MicrophoneView<'a> {
    pub fn new(window_state: &'a WindowState, app_mic: &'a mut Microphone) -> Self {
        MicrophoneView {
            window_state,
            mic: app_mic,
        }
    }
}

impl View for MicrophoneView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        StateWindow(
            default_window("Microphone")
                .resizable(true)
                .default_pos(self.window_state.get_pos(WindowKind::Microphone)),
        )
        .show_with_closure(
            ui,
            self.window_state.get_visible(WindowKind::Microphone),
            |_| self.window_state.set_visible(WindowKind::Microphone, false),
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
                    self.mic.start();
                }

                if ui.button("Stop Recording").clicked() {
                    self.mic.stop();
                }

                ui.horizontal(|ui| {
                    if ui.button("Play mic Audio").clicked() && self.mic.has_recording() {
                        let _ = self.mic.play_mic_audio();
                    }

                    if ui.button("pause").clicked() && self.mic.is_playing() {
                        let _ = self.mic.pause_mic_audio();
                    }

                    if ui.button("stop").clicked() && self.mic.is_playing() {
                        let _ = self.mic.stop_mic_audio();
                    }

                    if ui.button("clear").clicked() && self.mic.has_recording() {
                        let _ = self.mic.clear_mic();
                    }
                });
            },
        );
    }
}
