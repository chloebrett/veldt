use super::Piano;
use crate::{
    view::View,
    widget::{Sequencer, SequencerObject},
};
use egui::{Color32, CornerRadius, Id, Pos2, Rect, ScrollArea, Shape, Ui, pos2, vec2};
use mesic::create_scale_values;
use shared::{
    model::{Note, PitchName, PlacedNote, Scale, ScaleValue},
    types::PitchValue,
};
use state::{Action, FloatField, Selector, Store};

pub struct NoteRoll<'a> {
    store: &'a Store,
    track_index: usize,
    min_note: PitchValue,
    max_note: PitchValue,
    offset: f32,
    bar_length: f32,
}

impl<'a> NoteRoll<'a> {
    pub fn new(store: &'a Store, track_index: usize) -> Self {
        NoteRoll {
            store,
            track_index,
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
        let NoteRoll {
            store,
            track_index,
            min_note,
            max_note,
            offset,
            bar_length,
        } = *self;
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
        let notes = store.get().project.tracks[track_index].notes.clone();
        let white_note_pattern = self.make_white_note_pattern(max_note);
        if ui.button("New note").clicked() {
            store.dispatch(&Selector::Track(track_index), Action::AddNote(default_note));
        }
        let unclipped_duration = store.get().project.tracks[track_index].unclipped_duration();
        ScrollArea::vertical()
            .min_scrolled_height(200.0)
            .show(ui, |ui| {
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
                let dispatch = move |note_index: usize, action: Action| {
                    store.dispatch(&Selector::Note(track_index, note_index), action)
                };
                let on_release = || store.dispatchr(Action::Release);
                let on_click = |ui: &mut Ui, index: usize| {
                    let note_id = Id::new("note_window");
                    let note_index_id = Id::new("active_note_index");
                    ui.data_mut(|data| data.insert_temp(note_id, true));
                    ui.data_mut(|data| data.insert_temp(note_index_id, Some(index)));
                };
                ui.horizontal(|ui| {
                    Piano::new(max_note, min_note - 1).ui(ui);
                    ui.add(
                        Sequencer::new(range, dispatch, on_release, on_click)
                            .objects(notes)
                            .horizontal_rects(white_note_pattern, Color32::from_white_alpha(4))
                            .vertical_bars(bar_length, Color32::from_white_alpha(6))
                            .vertical_bars(1.0, Color32::from_white_alpha(3))
                            .vertical_bars(1.0 / bar_length, Color32::from_white_alpha(1)),
                    );
                });
            });
    }
}

impl SequencerObject<PlacedNote> for PlacedNote {
    fn to_pos(&self, range: Rect) -> Pos2 {
        let offset: f32 = self.offset.into();
        let x = offset - range.left();
        let pitch_value: PitchValue = self.note.pitch_name.into();
        let y = range.bottom() as i32 - pitch_value;
        pos2(x, y as f32)
    }

    fn to_rect(&self, range: Rect) -> Rect {
        let pos = self.to_pos(range);
        let note_size = vec2(self.note.beats, 1.0);
        Rect::from_min_size(pos, note_size)
    }

    fn x_action(&self, x: f32, range: Rect) -> Option<Action> {
        Some(Action::SetFloat(FloatField::Offset, x - range.left()))
    }

    fn y_action(&self, y: f32, range: Rect) -> Option<Action> {
        Some(Action::SetNotePitchName(PitchName::from(
            (range.bottom() - y) as i32,
        )))
    }

    fn resize_action(&self, x: f32, _range: Rect) -> Option<Action> {
        let beats = x - *self.offset;
        Some(Action::SetFloat(FloatField::Duration, beats))
    }

    fn shape(&self, range: Rect) -> egui::Shape {
        Shape::rect_filled(self.to_rect(range), CornerRadius::same(1), Color32::WHITE)
    }
}
