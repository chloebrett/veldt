use super::App;
use egui::{emath, Frame, Pos2, Rect, Sense, Shape, Ui, Vec2};

pub fn note_display(app: &mut App, ui: &mut Ui) {
    Frame::canvas(ui.style()).show(ui, |ui| {
        let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), 200.0), Sense::hover());
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect
        );
        response
    });
}

