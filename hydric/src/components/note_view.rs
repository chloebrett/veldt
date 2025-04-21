use crate::app_state::DataState;
use crate::view::View;
use crate::widget::{default_window, get_set, int_slider, selectable_value};
use egui::{Ui, pos2};
use shared::model::ScaleValue;
use shared::types::{Beats, Octave};
use state::{Action, FloatField, IndexField, Selector, Store, TypeField};
use strum::IntoEnumIterator;

pub struct NoteView<'a> {
    store: &'a Store,
}

impl<'a> NoteView<'a> {
    pub fn new(store: &'a Store) -> Self {
        NoteView { store }
    }
}

impl View for NoteView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let NoteView { store } = *self;
        let Some(note_index): Option<usize> = DataState::ActiveNoteIndex.get_value(ui) else {
            return;
        };
        let Some(track_index): Option<usize> = DataState::ActiveTrackIndex.get_value(ui) else {
            return;
        };
        let window_state = DataState::NoteWindow.get_value(ui).unwrap_or(false);
        if !window_state {
            return;
        };
        let mut open = window_state;
        let note = &store.get().project.tracks[track_index].notes[note_index];
        let on_release = || store.dispatchr(Action::Release);
        let sel = Selector::Note(track_index, note_index);
        default_window("Notes")
            .open(&mut open)
            .default_pos(pos2(600.0, 20.0))
            .show(ui.ctx(), |ui| {
                egui::ComboBox::from_id_salt(format!("note_{note_index}"))
                    .selected_text(note.note.pitch_name.scale_value.to_string())
                    .show_ui(ui, |ui| {
                        for scale_note in ScaleValue::iter() {
                            let scale_value = note.note.pitch_name.scale_value;
                            selectable_value(
                                ui,
                                get_set(&scale_value, |it| {
                                    store.dispatch(
                                        &sel,
                                        Action::SetChild(TypeField::ScaleValue(*it)),
                                    )
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
                    |it| store.dispatch(&sel, Action::SetChild(TypeField::Octave(it as Octave))),
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
                        Action::DeleteChild(IndexField::PlacedNote(note_index)),
                    );
                    DataState::ActiveNoteIndex.remove_value(ui);
                    DataState::NoteWindow.set_value(false, ui);
                }
            });
        if window_state != open {
            DataState::NoteWindow.set_value(false, ui);
        }
    }
}
