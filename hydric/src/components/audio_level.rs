use egui::{
    Color32, CornerRadius, Frame, Rect, Response, Sense, Shape, Ui, Vec2, Widget,
    emath::RectTransform, pos2, vec2,
};
use egui::{InnerResponse, Rangef};

use crate::playback::AudioPlayer;
use crate::transform::Transform;

/// Widget for rendering audio level in dB.
pub struct AudioLevel<'a> {
    player: &'a AudioPlayer,
    min_level: f32,
    max_level: f32,
    size: Vec2,
}

impl<'a> AudioLevel<'a> {
    pub fn new(player: &'a AudioPlayer) -> Self {
        Self {
            player,
            min_level: -60.0,
            max_level: 6.0,
            size: vec2(20.0, 150.0),
        }
    }

    fn create_level_shapes(&self, range: Rect, left_level: f32, right_level: f32) -> Shape {
        // padding around level line.
        let side_padding = 0.1;
        let top_padding = 5.0;
        let bottom_padding = 10.0;
        let padded_y_size =
            (range.size().y.abs() - (top_padding + bottom_padding)) / range.size().y.abs();
        // marker appears at peak of level.
        // marker will be twice this height.
        let marker_height = 0.5;
        let left_channel = |level, min_value| {
            Rect::from_x_y_ranges(
                Rangef {
                    min: side_padding,
                    max: (range.size().x - side_padding) * 0.5,
                },
                Rangef {
                    min: level * padded_y_size - top_padding,
                    max: min_value * padded_y_size - top_padding,
                },
            )
        };
        let right_channel = |level, min_value| {
            Rect::from_x_y_ranges(
                Rangef {
                    min: (range.size().x + side_padding) * 0.5,
                    max: range.right() - side_padding,
                },
                Rangef {
                    min: level * padded_y_size - top_padding,
                    max: min_value * padded_y_size - top_padding,
                },
            )
        };

        // Shaped behind level for each channel.
        let left_background_rect = left_channel(range.top(), range.bottom());
        let right_background_rect = right_channel(range.top(), range.bottom());
        // Level of each channel.
        let left_level_rect = left_channel(left_level, range.bottom());
        let right_level_rect = right_channel(right_level, range.bottom());
        // Marker at peak of level.
        let left_marker = left_channel(left_level + marker_height, left_level - marker_height);
        let right_marker = right_channel(right_level + marker_height, right_level - marker_height);

        let shapes: Vec<_> = [
            [left_background_rect, left_level_rect, left_marker],
            [right_background_rect, right_level_rect, right_marker],
        ]
        .into_iter()
        .flat_map(|[background, level, marker]| {
            vec![
                Shape::rect_filled(
                    background,
                    CornerRadius::same(0),
                    Color32::from_black_alpha(64),
                ),
                Shape::rect_filled(level, CornerRadius::same(0), Color32::WHITE),
                Shape::rect_filled(marker, CornerRadius::same(0), Color32::BLACK),
            ]
        })
        .collect();
        Shape::Vec(shapes)
    }
}

impl Widget for AudioLevel<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let AudioLevel {
            player,
            min_level,
            max_level,
            size,
        } = self;
        let range = Rect::from_min_max(pos2(0.0, max_level), pos2(1.0, min_level));
        // Get the level of audio channels.
        let [left_level, right_level] = player.level();
        let InnerResponse { inner: _, response } = Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(size, Sense::all());
            let level_shapes = self.create_level_shapes(range, left_level, right_level);
            let to_screen = RectTransform::from_to(range, response.rect);
            // Add background shape.
            painter.add(Shape::rect_filled(
                response.rect,
                CornerRadius::same(0),
                Color32::from_white_alpha(16),
            ));
            painter.add(level_shapes.transform(to_screen));
        });
        response
    }
}
