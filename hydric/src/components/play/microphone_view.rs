use crate::{components::play::Microphone, AsyncState};
use crate::promise::spawn;
use crate::view::View;
use crate::widget::default_window;
use egui::{Pos2, Ui};
use state::Store;
use log::info;
use super::microphone::*;

pub struct MicrophoneView<'a> {
    store: &'a Store,
    async_state: &'a mut AsyncState,
    visible: &'a mut bool,
}

impl<'a> MicrophoneView<'a> {
    pub fn new(store: &'a Store, async_state: &'a mut AsyncState, visible: &'a mut bool) -> Self {
        MicrophoneView {
            store,
            async_state,
            visible,
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
                if ui.button("test").clicked(){
                    info!("hi");
                    spawn(&mut self.async_state.microphone, async move{

                        let mut mic = Microphone::new();

                        mic.start()
                        .await
                        .map(|stream| {
                            let mic_stream = Some(stream);
                            //let mic_error = None;
                        })
                        .map_err(|err| {
                            let mic_error = Some(err.as_string().unwrap_or("Unknown error".into()));
                        })
                    });
                }
            });
    }
}
