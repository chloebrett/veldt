use state::{Action, Selector, Store};

use egui::{Color32, Pos2, Rect, ScrollArea, Ui, pos2, vec2};
use shared::{
    model::{Note, PitchName, PlacedNote, ScaleValue},
    types::PitchValue,
};

use super::Piano;
use crate::{
    view::View,
    widget::{Sequencer, SequencerObject},
};

pub struct NoteRoll {
    track_index: usize,
    min_note: PitchValue,
    max_note: PitchValue,
    offset: f32,
    bars: f32,
    bar_length: f32,
}

impl NoteRoll {
    pub fn new(track_index: usize) -> Self {
        NoteRoll {
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
            bars: 4.0,
            bar_length: 4.0,
        }
    }
}

impl View for NoteRoll {
    fn ui(&self, store: &Store, ui: &mut Ui) {
        let NoteRoll {
            track_index,
            min_note,
            max_note,
            offset,
            bars,
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
        if ui.button("New note").clicked() {
            store.dispatch(&Selector::Track(track_index), Action::AddNote(default_note));
        }
        ScrollArea::vertical()
            .min_scrolled_height(200.0)
            .show(ui, |ui| {
                let range = Rect::from_min_max(
                    pos2(offset, min_note as f32 - 1.0),
                    pos2(bars * bar_length, max_note as f32),
                );
                let dispatch = move |note_index: usize, action: Action| {
                    store.dispatch(&Selector::Note(track_index, note_index), action)
                };
                ui.horizontal(|ui| {
                    Piano::new(max_note, min_note - 1).ui(store, ui);
                    ui.add(
                        Sequencer::new(range, dispatch)
                            .objects(notes)
                            .horizontal_rects(2.0, Color32::from_white_alpha(4))
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

    fn x_action(&self, x: f32, range: Rect) -> Action {
        Action::SetNoteOffset(x - range.left())
    }

    fn y_action(&self, y: f32, range: Rect) -> Action {
        Action::SetNotePitchName(PitchName::from((range.bottom() - y) as i32))
    }
}
