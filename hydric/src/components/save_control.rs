use crate::audio_player::{Handle, play};
use crate::audio_render::render;
use crate::envelope_control;
use crate::note_save::{load_note_list, load_notes, save_notes};
use egui::{
    Color32, Rect, ScrollArea, Ui, containers::Frame, emath, epaint, epaint::PathStroke, pos2,
    scroll_area::ScrollBarVisibility, vec2,
};
use mesic::{SAMPLE_RATE, create_scale_values, create_track, render as local_render};
use poll_promise::Promise;
use shared::model::{
    AdsrEnvelope, DelayConfig, Effect, EffectInstance, EffectMeta, EqConfig, EqType,
    GeneratorInstance, GeneratorMeta, GeneratorType, Note, PitchName, Scale, ScaleValue,
    SimpleWaveConfig, WaveType,
};
use strum::IntoEnumIterator;

fn save_control(app: &mut App, ui: &mut Ui) {
    if ui.button("Save").clicked() {
        let save_name = self.track_name.clone();
        let notes_to_save = self.notes.clone();
        self.save_notes_promise = Some(Promise::spawn_local(async move {
            save_notes(save_name, notes_to_save).await
        }));
        self.notes_list_promise = Promise::spawn_local(async move { load_note_list().await });
    }

    if let Some(list) = self.notes_list_promise.ready() {
        match list {
            Some(values) => self.track_list = values.to_vec(),
            None => self.track_list = vec![],
        }
    }
    egui::ComboBox::from_label("Saved Tracks")
        .selected_text(self.track_name.clone())
        .show_ui(ui, |ui| {
            for name in self.track_list.iter() {
                ui.selectable_value(&mut self.track_name, name.clone(), name);
            }
        });
    if ui.button("Load").clicked() {
        let load_name = self.track_name.clone();
        self.notes_promise = Some(Promise::spawn_local(
            async move { load_notes(load_name).await },
        ))
    }
    if let Some(notes_promise) = &self.notes_promise {
        if let Some(notes) = notes_promise.ready() {
            match notes {
                Some(values) => self.notes = values.to_vec(),
                None => {}
            }
        }
    }
}
