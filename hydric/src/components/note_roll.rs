use super::{Piano, PianoOrientation};
use crate::components::NoteSequencer;
use crate::{GetSet, LocalState};
use crate::{
    playback::AudioPlayer, transform::Yx, view::View, widget::StateWindow, window_state::WindowKind,
};
use egui::{
    Color32, CornerRadius, Pos2, Rect, ScrollArea, Shape, Stroke, StrokeKind, Ui, Vec2, pos2, vec2,
};
use mesic::create_scale_values;
use shared::{
    model::{Note, PitchName, PlacedNote, PlacementType, Scale, ScaleValue, TrackId},
    types::PitchValue,
};
use state::{
    Action, FloatField, GeneratorSelector, IndexField, MultiIndexField, NoteSelector, Store,
    TrackSelector, TypeField,
};
use std::collections::HashSet;

// The max number of bars the NoteSequencer will allow placement on.
// TODO: Where is the best place for this definition? Should it be user changeable?
const MAX_BARS: f32 = 16.0;

pub struct NoteRoll<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
    audio_player: &'a mut AudioPlayer,
    min_note: PitchValue,
    max_note: PitchValue,
    offset: f32,
    bar_length: f32,
}

impl<'a> NoteRoll<'a> {
    pub fn new(
        store: &'a Store,
        local_state: &'a LocalState,
        audio_player: &'a mut AudioPlayer,
    ) -> Self {
        Self {
            store,
            local_state,
            audio_player,
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

    fn make_white_note_pattern(&self, max_note: i32) -> impl Fn(i32) -> bool + use<> {
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
        log::info!(
            "there should be an active track {:?}",
            self.local_state.active_track.get()
        );
        let Some(track_sel) = self.local_state.active_track.get() else {
            return;
        };
        let notes = store.select(&track_sel).notes.clone();
        let white_note_pattern = self.make_white_note_pattern(max_note);
        let range = Rect::from_min_max(
            pos2(offset, min_note as f32 - 1.0),
            pos2(bar_length * MAX_BARS, max_note as f32),
        );
        let title = format!("Track {}", *track_sel.0);
        StateWindow::show_from_window_state_resizable(
            ui,
            &local_state.window_state,
            WindowKind::NoteRoll,
            &title,
            |ui| {
                ui.separator();
                ScrollArea::vertical()
                    .min_scrolled_height(200.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // TODO: use the correct generator for the track placement that
                            // actually opened this UI - not just the first track placement we can
                            // find that matches this track.
                            let gen_sel = store
                                .get()
                                .project
                                .placements
                                .values()
                                .filter_map(|placement| match &placement.kind {
                                    PlacementType::Track(it) if it.track_id == track_sel.0 => {
                                        Some(it.generator_id)
                                    }
                                    _ => None,
                                })
                                .next()
                                .map(GeneratorSelector);

                            Piano::new(
                                max_note,
                                min_note - 1,
                                PianoOrientation::Vertical,
                                Vec2::new(600.0, 50.0),
                                Some(self.audio_player),
                                gen_sel,
                            )
                            .ui(ui);
                            ScrollArea::horizontal()
                                .min_scrolled_width(400.0)
                                .show(ui, |ui| {
                                    ui.add(
                                        NoteSequencer::new(store, local_state, range, track_sel.0)
                                            .objects(
                                                notes
                                                    .into_iter()
                                                    .map(NoteSequencerObject)
                                                    .collect(),
                                            )
                                            .horizontal_rects(
                                                white_note_pattern,
                                                Color32::from_white_alpha(4),
                                            )
                                            .vertical_bars(bar_length, Color32::from_white_alpha(6))
                                            .vertical_bars(1.0, Color32::from_white_alpha(3))
                                            .vertical_bars(
                                                1.0 / bar_length,
                                                Color32::from_white_alpha(1),
                                            ),
                                    );
                                });
                        });
                    });
            },
        );
    }
}

pub struct NoteSequencerObject(pub PlacedNote);

impl NoteSequencerObject {
    pub fn to_pos(&self, range: Rect) -> Pos2 {
        let offset: f32 = self.0.offset.into();
        let y = offset - range.top();
        let pitch_value: PitchValue = self.0.note.pitch_name.into();
        let x = range.right() as i32 - pitch_value;
        pos2(x as f32, y)
    }

    pub fn to_rect(&self, range: Rect) -> Rect {
        let pos = self.to_pos(range.yx()).yx();
        let note_size = vec2(self.0.note.beats, 1.0);
        Rect::from_min_size(pos, note_size)
    }

    pub fn resize_action(&self, x: f32) -> Option<Action> {
        let beats = x - *self.0.offset;
        Some(Action::SetFloat(FloatField::Duration, beats))
    }

    pub fn shape(&self, range: Rect) -> egui::Shape {
        Shape::rect_filled(self.to_rect(range), CornerRadius::same(1), Color32::WHITE)
    }

    pub fn get_active(store: &Store, local_state: &LocalState) -> Option<Self> {
        let track_sel = local_state.active_track.get()?;
        local_state.active_note.get().map(|note_index| {
            let sel: NoteSelector = track_sel.downcast_note(note_index);
            let note: &PlacedNote = store.select(&sel);
            NoteSequencerObject(note.clone())
        })
    }

    pub fn active_shape(&self, range: Rect) -> Shape {
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

    pub fn get_selected(store: &Store, local_state: &LocalState) -> Vec<Self> {
        let Some(track_sel) = local_state.active_track.get() else {
            return vec![];
        };
        local_state
            .selected_notes
            .get()
            .into_iter()
            .map(|note_index| {
                let sel: NoteSelector = track_sel.downcast_note(note_index);
                let note: &PlacedNote = store.select(&sel);
                NoteSequencerObject(note.clone())
            })
            .collect()
    }

    pub fn selected_shape(&self, range: Rect) -> Shape {
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

    pub fn set_active(&self, local_state: &LocalState, index: usize) {
        local_state.window_state.set_visible(WindowKind::Note, true);
        local_state.active_note.set(Some(index));
    }

    pub fn set_selected(local_state: &LocalState, index: Option<usize>) {
        let Some(index) = index else {
            local_state.selected_notes.set(HashSet::default());
            return;
        };

        let mut notes = local_state.selected_notes.get();

        if notes.contains(&index) {
            notes.remove(&index);
        } else {
            notes.insert(index);
        }

        local_state.selected_notes.set(notes);
    }

    pub fn add_new(&self, store: &Store, track_id: TrackId) {
        store.dispatch(
            &TrackSelector(track_id),
            Action::AddChild(TypeField::PlacedNote(self.0.clone())),
        );
    }

    pub fn from_pos(pos: Pos2, range: Rect) -> Self {
        let offset = pos.x + range.left();
        let pitch_value: PitchValue = (range.bottom() - pos.y) as i32;
        NoteSequencerObject(PlacedNote {
            note: Note {
                pitch_name: pitch_value.into(),
                beats: 1.0,
            },
            offset: offset.into(),
        })
    }

    pub fn delete_selected(store: &Store, local_state: &LocalState, track_id: TrackId) {
        // De-activate active note.
        // TODO: correctly handle the active note.
        // Currently if the deleted note index is less than the active note index, the active note
        // will either change or the index will be out of bounds and panic.
        local_state.active_note.update(|_| None);
        store.dispatch(
            &TrackSelector(track_id),
            Action::DeleteChildren(MultiIndexField::PlacedNote(
                local_state.selected_notes.get().into_iter().collect(),
            )),
        );
    }

    pub fn delete_self(
        &self,
        store: &Store,
        local_state: &LocalState,
        track_id: TrackId,
        note_index: usize,
    ) {
        // De-activate active note.
        // TODO: correctly handle the active note.
        // Currently if the deleted note index is less than the active note index, the active note
        // will either change or the index will be out of bounds and panic.
        local_state.active_note.update(|_| None);
        store.dispatch(
            &TrackSelector(track_id),
            Action::DeleteChild(IndexField::PlacedNote(note_index)),
        )
    }
}
