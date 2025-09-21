use crate::LocalState;
use crate::components::drum_placement_view::get_sample_name;
use crate::local_state::GetSet;
use crate::view::View;
use crate::widget::{StateWindow, get_set, int_slider, selectable_value, slider};
use crate::window_state::WindowKind;
use egui::{Ui, Window};
use shared::model::PlacementType;
use shared::model::ScaleValue;
use shared::types::{Beats, Octave};
use state::{Action, FloatField, IndexField, PlacementSelector, Store, TrackSelector, TypeField};
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
        StateWindow(
            Window::new("Note")
                .default_pos(local_state.window_state.get_pos(WindowKind::Note))
                .id(local_state.window_state.get_id(WindowKind::Note)),
        )
        .show_with_closure(
            ui,
            local_state.window_state.get_visible(WindowKind::Note),
            |_| {
                local_state.active_note.set(None);
                local_state
                    .window_state
                    .set_visible(WindowKind::Note, false);
            },
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

                if let Some(placement) = local_state
                    .active_placement
                    .get()
                    .and_then(|id| self.store.get().project.placements.get(&id))
                {
                    match &placement.kind {
                        PlacementType::DrumTrack(drum_placement) => {
                            let placement_id = local_state.active_placement.get().unwrap();
                            let samples_exist = !self.store.get().project.samples.is_empty();
                            let pitch_value: i32 = note.note.pitch_name.into();
                            let selected_text = if samples_exist {
                                if let Some(sample_id) =
                                    &drum_placement.pitch_sample_map.get(&pitch_value)
                                {
                                    get_sample_name(self.store, *sample_id)
                                } else {
                                    "No sample set for this pitch".to_string()
                                }
                            } else {
                                "No samples".to_string()
                            };
                            egui::ComboBox::from_id_salt(format!(
                                "drum_placement_pitch_sample{:?}-{}",
                                placement_id, note.note.pitch_name
                            ))
                            .selected_text(selected_text)
                            .show_ui(ui, |ui| {
                                if samples_exist {
                                    let drum_placement_sel = PlacementSelector(placement_id);
                                    for sample_id in self.store.get().project.samples.keys() {
                                        let is_selected = drum_placement
                                            .pitch_sample_map
                                            .get(&pitch_value)
                                            .map_or(false, |selected_sample| {
                                                *sample_id == *selected_sample
                                            });

                                        let response = ui.selectable_label(
                                            is_selected,
                                            get_sample_name(store, sample_id),
                                        );

                                        if response.clicked() {
                                            self.store.dispatch(
                                                &drum_placement_sel,
                                                Action::SetChildById(
                                                    TypeField::PitchName(note.note.pitch_name),
                                                    TypeField::SampleId(*sample_id),
                                                ),
                                            );
                                        }
                                    }
                                } else {
                                    ui.label("No samples to select");
                                }
                            });
                            slider(
                                ui,
                                "Pitch offset (semitones)",
                                note.pitch_offset as f64,
                                |it| {
                                    store.dispatch(
                                        &sel,
                                        Action::SetFloat(FloatField::Semitones, it as f32),
                                    )
                                },
                                -24.0..=24.0, // can repitch plus or minus two octaves
                                on_release,
                            );
                        }
                        _ => {}
                    }
                }

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
                    self.local_state
                        .window_state
                        .set_visible(WindowKind::Note, false);
                    self.local_state.selected_notes.update(|mut it| {
                        it.remove(&note_index);
                        it
                    });
                }
            },
        );
    }
}
