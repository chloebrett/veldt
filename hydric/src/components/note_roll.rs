use super::{Piano, PianoOrientation};
use crate::{
    DataState, GetSetOption, LocalState,
    transform::Yx,
    view::View,
    widget::{Sequencer, SequencerObject, StateWindow, default_window},
};
use egui::{
    Color32, CornerRadius, Pos2, Rect, ScrollArea, Shape, Stroke, StrokeKind, Ui, pos2, vec2,
};
use mesic::create_scale_values;
use shared::{
    model::{Note, PitchName, PlacedNote, Scale, ScaleValue},
    types::PitchValue,
};
use state::{
    Action, FloatField, IndexField, NoteSelector, SelectorTrait, Store, TrackSelector, TypeField,
};
use std::collections::BTreeSet;

pub struct NoteRoll<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
    min_note: PitchValue,
    max_note: PitchValue,
    offset: f32,
    bar_length: f32,
}

impl<'a> NoteRoll<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState) -> Self {
        Self {
            store,
            local_state,
            min_note: PitchName {
                scale_value: ScaleValue::A,
                octave: 1,
            }
            .into(),
            max_note: PitchName {
                scale_value: ScaleValue::C,
                octave: 8,
            }
            .into(),
            offset: 0.0,
            bar_length: 4.0,
        }
    }

    fn make_white_note_pattern(&self, max_note: i32) -> impl Fn(i32) -> bool {
        // Return a pattern for Background Rects to display white notes.
        // Account for max note changing.
        let c_value: PitchValue = ScaleValue::C.into();
        let max_scale_value: PitchValue = PitchName::from(max_note).scale_value.into();
        let c_delta = c_value - max_scale_value;
        move |y| {
            let notes = create_scale_values(Scale::Major, ScaleValue::C);
            // Return true for notes in C Major (White notes)
            // Determine if `y` is a white note by checking if the `ScaleValue` of the
            // note is in the C Major scale where `0 => C`, `1 => CSharp` etc.
            // Calculate `y` modulo `12` to account for higher values of y (`ScaleValues` are
            // between 0 and 11).
            // `y` will start at 0 no matter what the `max_note` is. Account for this by
            // adding the difference between the C `ScaleValue` and the `max_note` scale so that
            // `y` will start at the correct `ScaleValue`.
            // Use the negative of `y + c_delta` as `y` starts from the top of the piano
            // and moves down and so moves backwards through the scale.
            let scale_value = ScaleValue::from(((0 - (y + c_delta)) as i32).rem_euclid(12) as i32);
            notes.contains(&scale_value)
        }
    }
}

impl View for NoteRoll<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            store,
            min_note,
            max_note,
            offset,
            bar_length,
            local_state,
            ..
        } = *self;
        let Some(track_sel) = self.local_state.active_track.get() else {
            return;
        };
        let default_note = PlacedNote {
            note: Note {
                pitch_name: PitchName {
                    scale_value: store.get().key,
                    octave: 4,
                },
                beats: 1.0,
            },
            offset: offset.into(),
        };
        let notes = store.select(&track_sel).notes.clone();
        let white_note_pattern = self.make_white_note_pattern(max_note);
        let unclipped_duration = store.select(&track_sel).unclipped_duration();
        let range = Rect::from_min_max(
            pos2(offset, min_note as f32 - 1.0),
            // NoteRoll is at least 1 bar long
            // Extends when notes are dragged or set beyond 1 bar.
            // Add 0.5 to X as a small buffer after max note.
            pos2(
                f32::max(bar_length, *unclipped_duration) + 0.5,
                max_note as f32,
            ),
        );
        let mut select = DataState::NoteRollSelectMode.get_value(ui).unwrap_or(false);
        if !select {
            local_state.selected_notes.set_none();
        }
        let title = format!("Track {}", track_sel.0);
        let window = StateWindow(
            default_window(&title)
                .default_pos(Pos2 { x: 600.0, y: 20.0 })
                .resizable(true),
        );
        window.show(ui, DataState::NoteRollWindow, |ui| {
            ui.horizontal(|ui| {
                if ui.button("New note").clicked() {
                    store.dispatch(
                        &track_sel,
                        Action::AddChild(TypeField::PlacedNote(default_note)),
                    );
                }
                ui.checkbox(&mut select, "Select")
            });
            ScrollArea::vertical()
                .min_scrolled_height(200.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        Piano::new(max_note, min_note - 1, PianoOrientation::Vertical).ui(ui);
                        ui.add(
                            Sequencer::new(store, local_state, range)
                                .objects(notes)
                                .parent_index(track_sel.0)
                                .select(select)
                                .horizontal_rects(white_note_pattern, Color32::from_white_alpha(4))
                                .vertical_bars(bar_length, Color32::from_white_alpha(6))
                                .vertical_bars(1.0, Color32::from_white_alpha(3))
                                .vertical_bars(1.0 / bar_length, Color32::from_white_alpha(1)),
                        );
                    });
                });
        });
        DataState::NoteRollSelectMode.set_value(ui, select);
    }
}

impl SequencerObject<PlacedNote> for PlacedNote {
    fn to_pos(&self, range: Rect) -> Pos2 {
        let offset: f32 = self.offset.into();
        let y = offset - range.top();
        let pitch_value: PitchValue = self.note.pitch_name.into();
        let x = range.right() as i32 - pitch_value;
        pos2(x as f32, y)
    }

    fn to_rect(&self, range: Rect) -> Rect {
        let pos = self.to_pos(range.yx()).yx();
        let note_size = vec2(self.note.beats, 1.0);
        Rect::from_min_size(pos, note_size)
    }

    fn x_action(&self, x: f32, range: Rect) -> Option<Action> {
        Some(Action::SetFloat(FloatField::Offset, x - range.left()))
    }

    fn y_action(&self, y: f32, range: Rect) -> Option<Action> {
        Some(Action::SetChild(TypeField::PitchName(PitchName::from(
            (range.bottom() - y) as i32,
        ))))
    }

    fn resize_action(&self, x: f32, _range: Rect) -> Option<Action> {
        let beats = x - *self.offset;
        Some(Action::SetFloat(FloatField::Duration, beats))
    }

    fn shape(&self, range: Rect) -> egui::Shape {
        Shape::rect_filled(self.to_rect(range), CornerRadius::same(1), Color32::WHITE)
    }

    fn get_active(_ui: &Ui, store: &Store, local_state: &LocalState) -> Option<PlacedNote> {
        let track_sel = local_state.active_track.get()?;
        local_state.active_note.get().map(|note_index| {
            let sel: NoteSelector = track_sel.downcast_note(note_index);
            let note: &PlacedNote = store.select(&sel);
            note.clone()
        })
    }

    fn active_shape(&self, range: Rect) -> Shape {
        Shape::Vec(vec![
            self.shape(range),
            Shape::rect_stroke(
                self.to_rect(range),
                CornerRadius::same(0),
                Stroke {
                    width: 1.0,
                    color: Color32::BLUE,
                },
                StrokeKind::Inside,
            ),
        ])
    }

    fn get_selected(_ui: &Ui, store: &Store, local_state: &LocalState) -> Option<Vec<PlacedNote>> {
        let track_sel = local_state.active_track.get()?;
        let note_indexes = local_state.selected_notes.get()?;
        Some(
            note_indexes
                .into_iter()
                .map(|note_index| {
                    let sel: NoteSelector = track_sel.downcast_note(note_index);
                    let note: &PlacedNote = store.select(&sel);
                    note.clone()
                })
                .collect(),
        )
    }

    fn selected_shape(&self, range: Rect) -> Shape {
        Shape::Vec(vec![
            self.shape(range),
            Shape::rect_stroke(
                self.to_rect(range),
                CornerRadius::same(0),
                Stroke {
                    width: 1.0,
                    color: Color32::RED,
                },
                StrokeKind::Inside,
            ),
        ])
    }

    fn selector(index: usize, parent_index: Option<usize>) -> impl SelectorTrait {
        NoteSelector(
            parent_index.expect("Track index should have been set as parent index"),
            index,
        )
    }

    fn set_active(&self, ui: &mut Ui, local_state: &LocalState, index: usize) {
        local_state.note_window.set(true);
        local_state.active_note.set(index);
    }

    fn set_selected(_ui: &mut Ui, local_state: &LocalState, index: Option<usize>) {
        let Some(index) = index else {
            local_state.selected_notes.set_none();
            return;
        };

        let Some(mut notes) = local_state.selected_notes.get() else {
            local_state
                .selected_notes
                .set(BTreeSet::from_iter(vec![index]));
            return;
        };

        if notes.contains(&index) {
            notes.remove(&index);
        } else {
            notes.insert(index);
        }

        local_state.selected_notes.set(notes);
    }

    fn add_new(&self, store: &Store, parent_index: Option<usize>) {
        store.dispatch(
            &TrackSelector(parent_index.expect("Should have been track index.")),
            Action::AddChild(TypeField::PlacedNote(self.clone())),
        );
    }

    fn from_pos(pos: Pos2, range: Rect) -> PlacedNote {
        let offset = pos.x + range.left();
        let pitch_value: PitchValue = (range.bottom() - pos.y) as i32;
        PlacedNote {
            note: Note {
                pitch_name: pitch_value.into(),
                beats: 1.0,
            },
            offset: offset.into(),
        }
    }

    fn delete(store: &Store, index: usize, parent_index: Option<usize>) {
        store.dispatch(
            &TrackSelector(parent_index.expect("Should have been a parent index")),
            Action::DeleteChild(IndexField::PlacedNote(index)),
        );
    }

    fn delete_selected(
        _ui: &mut Ui,
        store: &Store,
        local_state: &LocalState,
        parent_index: Option<usize>,
    ) {
        // Notes must be deleted in reverse order so that indices for the rest of the selected
        // notes do not change mid-process. E.g., if deleting `3` and `4`, if `3` is deleted first
        // the note that was at `4` will now be at `3` and the algorithm will either delete the wrong note or raise
        // and error.
        // BTreeSet provides an effecient way to keep and get from a sorted list.
        for index in local_state
            .selected_notes
            .get()
            .unwrap_or_default()
            .iter()
            .rev()
        {
            PlacedNote::delete(store, *index, parent_index);
        }
    }
}
