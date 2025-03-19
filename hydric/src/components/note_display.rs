use crate::state::Store;

use egui::{emath, epaint::RectShape, Color32, CornerRadius, Frame, Pos2, Rect, Sense, Shape, Ui, Vec2};
use shared::types::PitchValue;
use web_sys::console;

pub fn note_display(store: &Store, ui: &mut Ui) {
    Frame::canvas(ui.style()).show(ui, |ui| {
        let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), 200.0), Sense::hover());
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect
        );
        let position = ui.available_rect_before_wrap().min;
        let note_shapes: Vec<Shape> = store
            .get()
            .project
            .tracks[0]
            .notes
            .clone()
            .iter_mut()
            .enumerate()
            .map(|(i, note)| {
                let x1: f32 = note.offset.into();
                let x2 = x1 + note.note.beats;
                let y1 = Into::<PitchValue>::into(note.note.pitch_name) as f32;
                let y2 = y1 + 2.0;
                let pos_1 = Pos2 {x:x1, y:y1};
                let pos_2 = Pos2 {x:x2, y:y2};
                let min_corner = to_screen.transform_pos(pos_1);
                let max_corner = to_screen.transform_pos(pos_2); 
                let x_range = min_corner.x..=max_corner.x;
                let y_range = min_corner.y..=max_corner.y;
                let note_rect = Rect::from_x_y_ranges(x_range, y_range);
                Shape::rect_filled(note_rect, CornerRadius::same(1), Color32::WHITE)
            })
            .collect();
            let notes_in_screen = 

        painter.extend(note_shapes);
        response
    });
}

