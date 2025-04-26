use egui::{Color32, FontId, Pos2, Shape, Stroke, Ui, epaint};
use std::f32::consts::PI;

pub enum TextRotation {
    Clockwise,
    Anticlockwise,
}

pub fn rotate_text(ui: &mut Ui, text: &str, font_size: f32, rotation: TextRotation) {
    let font_id = FontId::proportional(font_size);

    // Galley is essentially the layout/container of the text
    let galley =
        ui.fonts(|fonts| fonts.layout_no_wrap(text.to_string(), font_id.clone(), Color32::WHITE));
    let galley_size = galley.size();
    let rotated_size = galley_size.yx();
    let padded_size = rotated_size + egui::Vec2::splat(4.0); // small padding to prevent clipping

    // Allocating space in the layout
    let (rect, _response) = ui.allocate_at_least(padded_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    // Determining where to draw the rotated text
    let center = rect.center();
    let draw_pos = match rotation {
        TextRotation::Anticlockwise => Pos2::new(
            center.x - galley_size.y * 0.5,
            center.y + galley_size.x * 0.5,
        ),
        TextRotation::Clockwise => Pos2::new(
            center.x + galley_size.y * 0.5,
            center.y - galley_size.x * 0.5,
        ),
    };

    let text_angle = match rotation {
        TextRotation::Anticlockwise => PI * -0.5,
        TextRotation::Clockwise => PI * 0.5,
    };

    let text_shape = epaint::TextShape {
        pos: draw_pos,
        galley,
        underline: Stroke::NONE,
        override_text_color: Some(Color32::WHITE),
        angle: text_angle,
        fallback_color: Color32::WHITE,
        opacity_factor: 1.0,
    };

    painter.add(Shape::Text(text_shape));
}
