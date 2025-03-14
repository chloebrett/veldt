use super::app::App;
use crate::audio_player::{Handle, play};
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

pub fn key_control(app: &mut App, ui: &mut Ui) {
    egui::ComboBox::from_label("Key")
        .selected_text(app.key.to_string())
        .show_ui(ui, |ui| {
            for scale_note in ScaleValue::iter() {
                ui.selectable_value(&mut app.key, scale_note, scale_note.to_string());
            }
        });
    egui::ComboBox::from_label("Scale")
        .selected_text(app.scale.to_string())
        .show_ui(ui, |ui| {
            for scale in Scale::iter() {
                ui.selectable_value(&mut app.scale, scale, scale.to_string());
            }
        });
}
