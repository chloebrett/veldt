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

fn delay_control(app: &mut App, ui: &mut Ui) {
    ui.add(egui::Slider::new(&mut self.delay_amplitude, 0.0..=1.0).text("Delay amplitude"));
    ui.add(
        egui::Slider::new(&mut self.delay_ms, 1.0..=1000.0)
            .text("Delay ms")
            .logarithmic(true),
    );
    ui.add(egui::Slider::new(&mut self.delay_wet, 0.0..=1.0).text("Delay wet"));
}
