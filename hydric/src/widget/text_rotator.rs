use egui::{Color32, FontId, Pos2, Shape, Stroke, Ui, epaint};
use std::f32::consts::PI;

pub enum TextRotation {
    Clockwise90,
    Anticlockwise90,
    Neutral, // to draw text without rotating for style consistency (e.g. in the mod matrix rows and col titles must be styled in the same way. Allowing neutral rotation prevents having to restyle horizontal col titles)
}

pub fn text_rotator(
    ui: &mut Ui,
    text: &str,
    font_size: f32,
    rotation: TextRotation,
    text_colour: Color32,
) {
    let font_id = FontId::proportional(font_size);

    // Galley is essentially the layout/container of the text
    let galley =
        ui.fonts(|fonts| fonts.layout_no_wrap(text.to_string(), font_id.clone(), Color32::WHITE));
    let galley_size = galley.size();

    let padded_size = match rotation {
        TextRotation::Anticlockwise90 | TextRotation::Clockwise90 => {
            let rotated_size = galley_size.yx();
            rotated_size + egui::Vec2::splat(4.0)
        }
        TextRotation::Neutral => galley_size + egui::Vec2::splat(4.0),
    };

    // Allocating space in the layout
    let (rect, _response) = ui.allocate_at_least(padded_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    // Determining where to draw the rotated text
    let center = rect.center();
    let draw_pos = match rotation {
        TextRotation::Anticlockwise90 => Pos2::new(
            center.x - galley_size.y * 0.5,
            center.y + galley_size.x * 0.5,
        ),
        TextRotation::Clockwise90 => Pos2::new(
            center.x + galley_size.y * 0.5,
            center.y - galley_size.x * 0.5,
        ),
        TextRotation::Neutral => rect.left_top(),
    };

    let text_angle = match rotation {
        TextRotation::Anticlockwise90 => PI * -0.5,
        TextRotation::Clockwise90 => PI * 0.5,
        TextRotation::Neutral => 0.0,
    };

    let text_shape = epaint::TextShape {
        pos: draw_pos,
        galley,
        underline: Stroke::NONE,
        override_text_color: Some(text_colour),
        angle: text_angle,
        fallback_color: Color32::WHITE,
        opacity_factor: 1.0,
    };

    painter.add(Shape::Text(text_shape));
}
