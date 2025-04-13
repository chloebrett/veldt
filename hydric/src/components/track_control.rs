use crate::widget::{int_slider, selectable_value};
use egui::Ui;
use mesic::create_scale_values;
use ordered_float::OrderedFloat;
use shared::model::{Note, PitchName, PlacedNote};
use shared::state::{Action, Selector};
use shared::types::{Beats, Octave};
use state::{Store, get_set};

pub fn track_control(store: &Store, ui: &mut Ui) {
    let scale_options = create_scale_values(store.get().scale, store.get().key);
    let track_index = 0;
    let track = &store.get().project.tracks[track_index];
    let on_release = || store.dispatchr(Action::Release);

    for note_index in 0..track.notes.len() {
        let note = &track.notes[note_index];
        let sel = Selector::Note(track_index, note_index);

        egui::ComboBox::from_id_salt(format!("note_{note_index}"))
            .selected_text(note.note.pitch_name.scale_value.to_string())
            .show_ui(ui, |ui| {
                for scale_note in scale_options.iter() {
                    let scale_value = note.note.pitch_name.scale_value;
                    selectable_value(
                        ui,
                        get_set(&scale_value, |it| {
                            store.dispatch(&sel, Action::SetNoteScaleValue(*it))
                        }),
                        scale_note,
                        scale_note.to_string(),
                    );
                }
            });

        let octave = note.note.pitch_name.octave as f64;
        int_slider(
            ui,
            "Octave",
            octave,
            |it| store.dispatch(&sel, Action::SetNoteOctave(it as Octave)),
            0..=8,
            on_release,
        );

        let duration = note.note.beats as f64;
        // TODO: use a float slider, but with quantisation.
        int_slider(
            ui,
            "Beats",
            duration,
            |it| store.dispatch(&sel, Action::SetNoteDuration(it as Beats)),
            0..=10,
            on_release,
        );

        let offset = *note.offset as f64;
        // TODO: use a float slider, but with quantisation.
        int_slider(
            ui,
            "Offset",
            offset,
            |it| store.dispatch(&sel, Action::SetNoteOffset(it as Beats)),
            0..=16,
            on_release,
        );

        if ui.button("Delete").clicked() {
            store.dispatch(
                &Selector::Track(track_index),
                Action::DeleteNote { note_index },
            );
            break;
        }
    }

    let track_length = track
        .notes
        .clone()
        .into_iter()
        .map(|it| it.offset + it.note.beats)
        .max()
        .unwrap_or(OrderedFloat(0.0));

    if ui.button("New note").clicked() {
        store.dispatch(
            &Selector::Track(track_index),
            Action::AddNote(PlacedNote {
                note: Note {
                    pitch_name: PitchName {
                        scale_value: store.get().key,
                        octave: 4,
                    },
                    beats: 1.0,
                },
                offset: track_length,
            }),
        );
    }
}
