use super::app::App;
use egui::{Context, Ui};
use mesic::create_scale_values;
use ordered_float::OrderedFloat;
use shared::model::{Note, PitchName, PlacedNote, Track};
use std::collections::BTreeSet;

pub fn notes_control(app: &mut App, ui: &mut Ui, ctx: &Context) {
    let scale_options = create_scale_values(app.store.scale, app.store.key);
    let mut placed_notes: Vec<PlacedNote> = app.store.project.tracks[0]
        .notes
        .iter()
        .map(|placed_note| placed_note.clone())
        .collect();
    let mut track_length = 0.0;
    for i in 0..placed_notes.len() {
        let placed_note = &mut placed_notes[i];
        let note = &mut placed_note.note;
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
        let mut offset_value = placed_note.offset.into();
        let response = ui.add(egui::Slider::new(&mut offset_value, 0.0..=16.0).text("Offset"));
        if !response.dragged() {
            placed_note.offset = OrderedFloat(offset_value);
        }
        track_length = f32::max(
            track_length,
            Into::<f32>::into(placed_note.offset) + placed_note.note.beats,
        );
        if ui.button("Delete").clicked() {
            placed_notes.remove(i);
            ctx.request_discard("");
            break;
        }
    }
    if ui.button("New note").clicked() {
        placed_notes.push(PlacedNote {
            note: Note {
                pitch_name: PitchName {
                    scale_value: app.store.key,
                    octave: 4,
                },
                beats: 1.0,
            },
            offset: OrderedFloat(track_length.ceil()),
        })
    }
    app.store.project.tracks[0] = Track {
        notes: BTreeSet::from_iter(placed_notes),
    };
}
