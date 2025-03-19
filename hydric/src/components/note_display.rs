use crate::state::Store;

use egui::{emath, Color32, CornerRadius, Frame, Pos2, Rect, Sense, Shape, Ui, Vec2};
use shared::types::PitchValue;
use web_sys::console;

pub fn note_display(store: &Store, ui: &mut Ui) {
    Frame::canvas(ui.style()).show(ui, |ui| {
        let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), 200.0), Sense::hover());
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect
        );
        let y_size = to_screen.to().max.y - to_screen.to().min.y;
        console::log_1(&format!("{:?}", y_size).into());
        let note_shapes: Vec<Shape> = store
            .get()
            .project
            .tracks[0]
            .notes
            .clone()
            .iter_mut()
            .enumerate()
            .map(|(_i, note)| {
                let x1: f32 = note.offset.into();
                let y1 = y_size - Into::<PitchValue>::into(note.note.pitch_name) as f32 - 5.0;
                let note_pos = Pos2 {x:x1, y:y1};
                // Closure to allow for moving notes in future.
                let note_corner = || {
                    let x2 = note_pos.x + note.note.beats;
                    let y2 = note_pos.y + 5.0;
                    Pos2 {x:x2, y:y2}
                };
                let min_corner = to_screen.transform_pos(note_pos);
                let max_corner = to_screen.transform_pos(note_corner()); 
                let note_rect = Rect::from_min_max(min_corner, max_corner);
                Shape::rect_filled(note_rect, CornerRadius::same(1), Color32::WHITE)
            })
            .collect();

        painter.extend(note_shapes);
        response
    });
}

