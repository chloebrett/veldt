use crate::state::{Action, Selector, Store};

use egui::{Color32, CornerRadius, Frame, Pos2, Rect, Sense, Shape, Ui, Vec2, emath};
use shared::{
    model::{Note, PitchName, PlacedNote},
    types::PitchValue,
};
use web_sys::console;

pub fn note_display(store: &Store, ui: &mut Ui) {
    Frame::canvas(ui.style()).show(ui, |ui| {
        let (response, painter) =
            ui.allocate_painter(Vec2::new(ui.available_width(), 315.0), Sense::hover());
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );
        let y_size = to_screen.to().max.y - to_screen.to().min.y;
        let x_size = to_screen.to().max.x - to_screen.to().min.x;
        let inv_x_size = 1.0 / x_size;
        let inv_y_size = 1.0 / y_size;
        let max_pitch_value: PitchValue = PitchName {
            scale_value: shared::model::ScaleValue::GSharp,
            octave: 8,
        }
        .into();
        let inv_max_pitch_value: f32 = 1.0 / max_pitch_value as f32;
        let project_length: f32 = 16.0; // TODO integrate into project
        let inv_project_length: f32 = 1.0 / project_length;
        let note_shapes: Vec<Shape> = store.get().project.tracks[0]
            .notes
            .clone()
            .iter_mut()
            .enumerate()
            .map(|(i, note)| {
                let x1: f32 = Into::<f32>::into(note.offset) * inv_project_length * x_size;
                let pitch_proportion = 1.0
                    - Into::<PitchValue>::into(note.note.pitch_name) as f32 * inv_max_pitch_value;
                let y1 = y_size * pitch_proportion - 5.0;
                let note_pos = Pos2 { x: x1, y: y1 };
                // Closure to allow for moving notes in future.
                let note_corner = || {
                    let x2 = note_pos.x + note.note.beats * inv_project_length * x_size;
                    let y2 = note_pos.y + 5.0;
                    Pos2 { x: x2, y: y2 }
                };
                let min_corner = to_screen.transform_pos(note_pos);
                let max_corner = to_screen.transform_pos(note_corner());
                let note_rect = Rect::from_min_max(min_corner, max_corner);
                let note_id = response.id.with(i);
                let note_response = ui.interact(note_rect, note_id, Sense::drag());
                let note_delta = note_response.drag_delta();
                let scaled_note_delta = Vec2 {
                    x: note_delta.x * inv_x_size * project_length,
                    y: 0.0, //y: note_delta.y * inv_y_size * max_pitch_value as f32,
                };
                let offset_delta = ordered_float::OrderedFloat(scaled_note_delta.x);
                let pitch_delta = Into::<PitchName>::into(scaled_note_delta.y.round() as i32);

                *note = PlacedNote {
                    note: Note {
                        pitch_name: note.note.pitch_name + pitch_delta.into(),
                        beats: note.note.beats,
                    },
                    offset: note.offset + offset_delta,
                };
                let sel = Selector::Note(0, i);
                store.dispatch(&sel, Action::SetNoteOffset(*note.offset));
                console::log_1(&format!("{:?}", store.get().project.tracks[0].notes[0]).into());
                Shape::rect_filled(note_rect, CornerRadius::same(1), Color32::WHITE)
            })
            .collect();

        painter.extend(note_shapes);
        response
    });
}
