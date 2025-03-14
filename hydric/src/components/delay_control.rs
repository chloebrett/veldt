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

pub fn delay_control(app: &mut App, ui: &mut Ui) {
    ui.add(egui::Slider::new(&mut app.delay_amplitude, 0.0..=1.0).text("Delay amplitude"));
    ui.add(
        egui::Slider::new(&mut app.delay_ms, 1.0..=1000.0)
            .text("Delay ms")
            .logarithmic(true),
    );
    ui.add(egui::Slider::new(&mut app.delay_wet, 0.0..=1.0).text("Delay wet"));
}
