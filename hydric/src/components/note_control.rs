use crate::view::View;
use crate::widget::{get_set, int_slider, selectable_value};
use egui::{Id, Ui};
use shared::model::ScaleValue;
use shared::types::{Beats, Octave};
use state::{Action, FloatField, Selector, Store};
use strum::IntoEnumIterator;

pub struct NoteControl<'a> {
    store: &'a Store,
    track_index: usize,
    note_index: usize,
}

impl<'a> NoteControl<'a> {
    pub fn new(store: &'a Store, track_index: usize, note_index: usize) -> Self {
        NoteControl {
            store,
            track_index,
            note_index,
        }
    }
}

impl View for NoteControl<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let NoteControl {
            store,
            track_index,
            note_index,
        } = *self;
        let note = &store.get().project.tracks[track_index].notes[note_index];
        let on_release = || store.dispatchr(Action::Release);
        let sel = Selector::Note(track_index, note_index);
        egui::ComboBox::from_id_salt(format!("note_{note_index}"))
            .selected_text(note.note.pitch_name.scale_value.to_string())
            .show_ui(ui, |ui| {
                for scale_note in ScaleValue::iter() {
                    let scale_value = note.note.pitch_name.scale_value;
                    selectable_value(
                        ui,
                        get_set(&scale_value, |it| {
                            store.dispatch(&sel, Action::SetScaleValue(*it))
                        }),
                        &scale_note,
                        scale_note.to_string(),
                    );
                }
            });

        let octave = note.note.pitch_name.octave as f64;
        int_slider(
            ui,
            "Octave",
            octave,
            |it| store.dispatch(&sel, Action::SetOctave(it as Octave)),
            0..=8,
            on_release,
        );

        let duration = note.note.beats as f64;
        // TODO: use a float slider, but with quantisation.
        int_slider(
            ui,
            "Beats",
            duration,
            |it| store.dispatch(&sel, Action::SetFloat(FloatField::Duration, it as Beats)),
            0..=10,
            on_release,
        );

        let offset = *note.offset as f64;
        // TODO: use a float slider, but with quantisation.
        int_slider(
            ui,
            "Offset",
            offset,
            |it| store.dispatch(&sel, Action::SetFloat(FloatField::Offset, it as Beats)),
            0..=16,
            on_release,
        );

        if ui.button("Delete").clicked() {
            store.dispatch(
                &Selector::Track(track_index),
                Action::DeleteNote(note_index),
            );
            ui.data_mut(|data| {
                let window_id = Id::new("note_window");
                let active_note = Id::new("active_note_index");
                data.insert_temp(window_id, false);
                data.insert_temp::<Option<usize>>(active_note, None);
            });
        }
    }
}
