use state::{Action, Selector, Store};

use egui::{
    Color32, Frame, Pos2, Rect, Response, ScrollArea, Sense, Shape, Ui, Vec2, emath::RectTransform,
    pos2, vec2,
};
use ordered_float::OrderedFloat;
use shared::{
    model::{Note, PitchName, PlacedNote, ScaleValue},
    types::PitchValue,
};

use super::{piano::Piano, roll::Roll};

use crate::{transform::Transform, widget::Sequencer};

// TODO Integrate into Store and project.
struct ProjectConfig {
    pub max_note: PitchValue,
    pub min_note: PitchValue,
    pub offset: f32,
    pub bars: f32,
    pub bar_length: f32,
}

pub fn note_roll_display(store: &Store, ui: &mut Ui) {
    let track_index = 0;
    new_note_button(store, ui, track_index);
    ScrollArea::vertical()
        .min_scrolled_height(200.0)
        .show(ui, |ui| {
            draw_note_roll_canvas(store, ui, track_index);
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

fn draw_note_roll_canvas(store: &Store, ui: &mut Ui, track_index: usize) {
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
    let piano_width = 50.0;
    let canvas_height = 600.0;
    let note_rects = make_all_note_rects(
        store.get().project.tracks[track_index].notes.clone(),
        max_note,
        offset,
    );
    let range = Rect::from_min_max(
        pos2(offset, min_note as f32 - 1.0),
        pos2(bars * bar_length, max_note as f32),
    );
    let dispatch = move |sel: &Selector, pos: Pos2| {
        let offset = pos.x;
        let pitch_name = PitchName::from(max_note - pos.y as i32);
        store.dispatch(sel, Action::SetNoteOffset(offset));
        store.dispatch(sel, Action::SetNoteOctave(pitch_name.octave));
        store.dispatch(sel, Action::SetNoteScaleValue(pitch_name.scale_value));
    };
    ui.horizontal(|ui| {
        ui.add(Piano::new(max_note, min_note - 1));
        ui.add(
            Sequencer::new(range, dispatch)
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
