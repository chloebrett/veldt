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
use super::app::App;

pub fn generator_control(config: &mut SimpleWaveConfig, ui: &mut Ui) {
    egui::ComboBox::from_label("Wave type")
        .selected_text(config.wave.to_string())
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut config.wave, WaveType::Sine, "Sine");
            ui.selectable_value(&mut config.wave, WaveType::Square, "Square");
            ui.selectable_value(&mut config.wave, WaveType::Saw, "Saw");
            ui.selectable_value(&mut config.wave, WaveType::Triangle, "Triangle");
        });
    ui.add(
        egui::Slider::new(&mut config.osc_count, 1..=24)
            .text("Osc count")
            .logarithmic(true),
    );
    ui.add(
        egui::Slider::new(&mut config.detune_cents, 0.0..=100.0)
            .text("Osc detune")
            .logarithmic(true),
    );
}
