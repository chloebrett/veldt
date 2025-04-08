use state::{Action, Selector, Store};

use egui::{Color32, Pos2, Rect, ScrollArea, Ui, pos2, vec2};
use shared::{
    model::{Note, PitchName, PlacedNote, ScaleValue},
    types::PitchValue,
};

use super::Piano;
use crate::widget::Sequencer;

pub fn note_roll(store: &Store, ui: &mut Ui, track_index: usize) {
    new_note_button(store, ui, track_index);
    ScrollArea::vertical()
        .min_scrolled_height(200.0)
        .show(ui, |ui| {
            draw_note_roll(store, ui, track_index);
        });
}

fn new_note_button(store: &Store, ui: &mut Ui, track_index: usize) {
    if ui.button("New note").clicked() {
        store.dispatch(
            &Selector::Track(track_index),
            Action::AddNote(PlacedNote {
                note: Note {
                    pitch_name: PitchName {
                        scale_value: store.get().key,
                        octave: 4,
                    },
                    beats: 1.0,
                },
                offset: 0.0.into(),
            }),
        );
    }
}

fn draw_note_roll(store: &Store, ui: &mut Ui, track_index: usize) {
    let offset = 0.0;
    let bar_length = 4.0;
    let bars = 4.0;
    let max_note: PitchValue = PitchName {
        scale_value: ScaleValue::C,
        octave: 8,
    }
    .into();
    let min_note: PitchValue = PitchName {
        scale_value: ScaleValue::A,
        octave: 1,
    }
    .into();
    let note_rects = make_all_note_rects(
        store.get().project.tracks[track_index].notes.clone(),
        max_note,
        offset,
    );
    let range = Rect::from_min_max(
        pos2(offset, min_note as f32 - 1.0),
        pos2(bars * bar_length, max_note as f32),
    );
    let dispatch_x = move |sel: &Selector, offset: f32| {
        store.dispatch(sel, Action::SetNoteOffset(offset));
    };
    let dispatch_y = move |sel: &Selector, pitch_value: f32| {
        let pitch_name = PitchName::from(max_note - pitch_value as i32);
        store.dispatch(sel, Action::SetNoteOctave(pitch_name.octave));
        store.dispatch(sel, Action::SetNoteScaleValue(pitch_name.scale_value));
    };
    ui.horizontal(|ui| {
        Piano::new(max_note, min_note - 1).render(ui);
        ui.add(
            Sequencer::new(range, dispatch_x, dispatch_y)
                .rects(note_rects)
                .horizontal_rects(2.0, Color32::from_white_alpha(4))
                .vertical_bars(1.0, Color32::from_white_alpha(3))
                .vertical_bars(1.0 / bar_length, Color32::from_white_alpha(1)),
        );
    });
}

pub fn note_to_pos(note: &PlacedNote, max_note: i32, project_offset: f32) -> Pos2 {
    let offset: f32 = note.offset.into();
    let x = offset - project_offset;
    let pitch_value: PitchValue = note.note.pitch_name.into();
    let y = max_note - pitch_value;
    pos2(x, y as f32)
}

pub fn make_note_rect(note: &PlacedNote, note_pos: Pos2) -> Rect {
    let note_size = vec2(note.note.beats, 1.0);
    Rect::from_min_size(note_pos, note_size)
}

pub fn make_all_note_rects(
    notes: Vec<PlacedNote>,
    max_note: i32,
    project_offset: f32,
) -> Vec<Rect> {
    notes
        .iter()
        .map(|note| {
            let note_pos = note_to_pos(note, max_note, project_offset);
            make_note_rect(note, note_pos)
        })
        .collect()
}
