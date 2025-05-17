use crate::LocalState;
use crate::local_state::GetSet;
use crate::view::View;
use crate::widget::{StateWindow, default_window, get_set, int_slider, selectable_value, slider};
use egui::{Ui, pos2};
use shared::model::ScaleValue;
use shared::types::{Beats, Octave};
use state::{
    Action, FloatField, IndexField, Store, TrackSelector, TypeField,
};
use strum::IntoEnumIterator;

pub struct NoteView<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
}

impl<'a> NoteView<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState) -> Self {
        Self { store, local_state }
    }
}

impl View for NoteView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self { store, local_state } = self;
        let Some(note_index): Option<usize> = local_state.active_note.get() else {
            return;
        };
        let Some(track_sel): Option<TrackSelector> = local_state.active_track.get() else {
            return;
        };
        let on_release = || store.dispatchr(Action::Release);
        let sel = track_sel.downcast_note(note_index);
        let note = &store.select(&sel);
        let window = StateWindow(default_window("Notes").default_pos(pos2(600.0, 20.0)));
        window.show_with_closure(
            ui,
            local_state.note_window.get(),
            |_| local_state.note_window.set(false),
            |ui| {
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
                // TODO: Add quantisation.
                slider(
                    ui,
                    "Beats",
                    duration,
                    |it| store.dispatch(&sel, Action::SetFloat(FloatField::Duration, it as Beats)),
                    0.0..=10.0,
                    on_release,
                );

                let offset = *note.offset as f64;
                // TODO: Add quantisation.
                slider(
                    ui,
                    "Offset",
                    offset,
                    |it| store.dispatch(&sel, Action::SetFloat(FloatField::Offset, it as Beats)),
                    0.0..=16.0,
                    on_release,
                );

                if ui.button("Delete").clicked() {
                    store.dispatch(
                        &track_sel,
                        Action::DeleteChild(IndexField::PlacedNote(note_index)),
                    );
                    self.local_state.active_note.set(None);
                    self.local_state.note_window.set(false);
                    self.local_state.selected_notes.update(|mut it| {
                        it.remove(&note_index);
                        it
                    });
                }
            },
        );
    }
}
