use state::{Action, Selector, Store};

use egui::{Color32, Rect, ScrollArea, Ui, pos2};
use shared::{
    model::{Note, PitchName, PlacedNote, Scale, ScaleValue},
    types::PitchValue,
};

use mesic::create_scale_values;

use super::Piano;
use crate::{view::View, widget::Sequencer};

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
        // Pattern for Background Rects
        // Account for max note changing.
        let c_value: PitchValue = ScaleValue::C.into();
        let max_scale_value: PitchValue = PitchName::from(max_note).scale_value.into();
        let c_delta = c_value - max_scale_value;
        let background_pattern = |y| {
            let notes = create_scale_values(Scale::Major, ScaleValue::C);
            // Return true for notes in C Major (White notes)
            let scale_value = ScaleValue::from(((0 - y - c_delta) as i32).rem_euclid(12) as i32);
            notes.contains(&scale_value)
        };
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
                            .horizontal_rects(background_pattern, Color32::from_white_alpha(4))
                            .vertical_bars(bar_length, Color32::from_white_alpha(6))
                            .vertical_bars(1.0, Color32::from_white_alpha(3))
                            .vertical_bars(1.0 / bar_length, Color32::from_white_alpha(1)),
                    );
                });
            });
    }
}
