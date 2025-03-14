use super::app::App;
use egui::Ui;
use mesic::create_scale_values;
use shared::model::{Note, PitchName, ScaleValue};

pub fn notes_control(app: &mut App, ui: &mut Ui) {
    let scale_options = create_scale_values(app.scale, app.key);
    for i in 0..app.notes.len() {
        let note = &mut app.notes[i];
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
            app.notes.remove(i);
        }
    }
    if ui.button("New note").clicked() {
        app.notes.push(Note {
            pitch_name: PitchName {
                scale_value: ScaleValue::A,
                octave: 4,
            },
            beats: 1.0,
        });
    }
}
