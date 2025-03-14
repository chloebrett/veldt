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

fn notes_control(&mut self, ui: &mut Ui) {
    let scale_options = create_scale_values(self.scale, self.key);
    for i in 0..self.notes.len() {
        let note = &mut self.notes[i];
        let scale_value = &mut note.pitch_name.scale_value;
        egui::ComboBox::from_id_salt(i)
            .selected_text(scale_value.to_string())
            .show_ui(ui, |ui| {
                for scale_note in scale_options.iter() {
                    ui.selectable_value(scale_value, *scale_note, scale_note.to_string());
                }
            });
        ui.add(egui::Slider::new(&mut note.pitch_name.octave, 0..=8).text("Octave"));
        if ui.button("Delete").clicked() {
            self.notes.remove(i);
        }
    }
    if ui.button("New note").clicked() {
        self.notes.push(Note {
            pitch_name: PitchName {
                scale_value: ScaleValue::A,
                octave: 4,
            },
            beats: 1.0,
        });
    }
}
