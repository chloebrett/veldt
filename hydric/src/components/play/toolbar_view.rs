use crate::WindowState;
use crate::app_state::{AsyncState, AudioState};
use crate::components::{App, undo_redo_control};
use crate::view::View;
use crate::widget::{default_window, knob, slider};
use egui::{Context, Pos2, Ui};
use shared::types::Beats;
use state::{Action, FloatField, Store};

use super::{play_control::*, sample_control::*};

pub struct ToolBarView<'a> {
    window_state: &'a mut WindowState,
    store: &'a mut Store,
    async_state: &'a mut AsyncState,
    audio_state: &'a mut AudioState,
}

impl<'a> ToolBarView<'a> {
    pub fn new(
        window_state: &'a mut WindowState,
        store: &'a mut Store,
        async_state: &'a mut AsyncState,
        audio_state: &'a mut AudioState,
    ) -> Self {
        ToolBarView {
            window_state,
            store,
            async_state,
            audio_state,
        }
    }
}

impl View for ToolBarView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let ToolBarView {
            window_state,
            store,
            ..
        } = self;

        default_window("Toolbar")
            .default_pos(Pos2 { x: 600.0, y: 20.0 })
            .show(ui.ctx(), |ui| {
                let on_release = || self.store.dispatchr(Action::Release);

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
                undo_redo_control(&mut self.store, ui);
                ui.separator();
                play_control(
                    &self.store,
                    &mut self.async_state,
                    &mut self.audio_state,
                    ui,
                );
                ui.separator();
                sample_control(
                    &self.store,
                    &mut self.audio_state,
                    &mut self.async_state,
                    ui,
                );
            });
    }
}
