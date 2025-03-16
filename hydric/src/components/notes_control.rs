use super::app::App;
use egui::{Context, Ui};
use mesic::{create_scale_values, create_track};
use shared::model::{Note, PitchName};

pub fn notes_control(app: &mut App, ui: &mut Ui, ctx: &Context) {
    let scale_options = create_scale_values(app.scale, app.key);
    let mut notes: Vec<Note> = app
        .track
        .notes
        .iter()
        .map(|placed_note| placed_note.note.clone())
        .collect();
    for i in 0..notes.len() {
        let note = &mut notes[i];
        let scale_value = &mut note.pitch_name.scale_value;
        egui::ComboBox::from_id_salt(i)
            .selected_text(scale_value.to_string())
            .show_ui(ui, |ui| {
                for scale_note in scale_options.iter() {
                    ui.selectable_value(scale_value, *scale_note, scale_note.to_string());
                }
            });
        ui.add(egui::Slider::new(&mut note.pitch_name.octave, 0..=8).text("Octave"));
        ui.add(egui::Slider::new(&mut note.beats, 0.0..=10.0).text("Beats"));
        if ui.button("Delete").clicked() {
            notes.remove(i);
            ctx.request_discard("");
            break;
        }
    }
    if ui.button("New note").clicked() {
        notes.push(Note {
            pitch_name: PitchName {
                scale_value: app.key,
                octave: 4,
            },
            beats: 1.0,
        });
    }
    app.track = create_track(notes);
}
