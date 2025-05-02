use crate::DataState;
use crate::view::View;
use crate::widget::{StateWindow, default_window, get_set, int_slider, selectable_value};
use egui::{Ui, pos2};
use shared::model::ScaleValue;
use shared::types::{Beats, Octave};
use state::{Action, FloatField, IndexField, NoteSelector, Store, TrackSelector, TypeField};
use strum::IntoEnumIterator;

pub struct NoteView<'a> {
    store: &'a Store,
}

impl<'a> NoteView<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }
}

impl View for NoteView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self { store } = self;
        let Some(note_index): Option<usize> = DataState::ActiveNoteIndex.get_value(ui) else {
            return;
        };
        let Some(track_index): Option<usize> = DataState::ActiveTrackIndex.get_value(ui) else {
            return;
        };
        let on_release = || store.dispatchr(Action::Release);
        let sel = NoteSelector(track_index, note_index);
        let note = &store.select(&sel);
        let window = StateWindow(default_window("Notes").default_pos(pos2(600.0, 20.0)));
        window.show(ui, DataState::NoteWindow, |ui| {
            egui::ComboBox::from_id_salt(format!("note_{note_index}"))
                .selected_text(note.note.pitch_name.scale_value.to_string())
                .show_ui(ui, |ui| {
                    for scale_note in ScaleValue::iter() {
                        let scale_value = note.note.pitch_name.scale_value;
                        selectable_value(
                            ui,
                            get_set(&scale_value, |it| {
                                store.dispatch2(&sel, Action::SetChild(TypeField::ScaleValue(*it)))
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
                |it| store.dispatch2(&sel, Action::SetChild(TypeField::Octave(it as Octave))),
                0..=8,
                on_release,
            );

            let duration = note.note.beats as f64;
            // TODO: use a float slider, but with quantisation.
            int_slider(
                ui,
                "Beats",
                duration,
                |it| store.dispatch2(&sel, Action::SetFloat(FloatField::Duration, it as Beats)),
                0..=10,
                on_release,
            );

            let offset = *note.offset as f64;
            // TODO: use a float slider, but with quantisation.
            int_slider(
                ui,
                "Offset",
                offset,
                |it| store.dispatch2(&sel, Action::SetFloat(FloatField::Offset, it as Beats)),
                0..=16,
                on_release,
            );

            if ui.button("Delete").clicked() {
                store.dispatch2(
                    &TrackSelector(track_index),
                    Action::DeleteChild(IndexField::PlacedNote(note_index)),
                );
                DataState::ActiveNoteIndex.remove_value(ui);
                DataState::NoteWindow.set_value(ui, false);
            }
        });
    }
}
