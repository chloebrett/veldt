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

    fn create_channel_shape(&self, range: Rect, level: f32) -> Shape {
        // Padding around channel shapes.
        let side_padding = 0.2;
        let top_padding = 5.0;
        let bottom_padding = 10.0;
        let padded_y_size =
            (range.size().y.abs() - (top_padding + bottom_padding)) / range.size().y.abs();
        // Marker will be twice this height.
        let marker_height = 0.5;
        let channel_rect = |top, bottom| {
            Rect::from_x_y_ranges(
                Rangef {
                    min: range.left() + side_padding,
                    max: range.right() - side_padding,
                },
                Rangef {
                    min: top * padded_y_size - top_padding,
                    max: bottom * padded_y_size - top_padding,
                },
            )
        };
        Shape::Vec(vec![
            // Background of channel.
            Shape::rect_filled(
                channel_rect(range.top(), range.bottom()),
                CornerRadius::same(0),
                Color32::from_black_alpha(64),
            ),
            // Level of channel.
            Shape::rect_filled(
                channel_rect(level, range.bottom()),
                CornerRadius::same(0),
                Color32::WHITE,
            ),
            // Peak marker.
            Shape::rect_filled(
                channel_rect(level - marker_height, level + marker_height),
                CornerRadius::same(0),
                Color32::BLACK,
            ),
        ])
    }

    fn create_level_shape(&self, range: Rect, left_level: f32, right_level: f32) -> Shape {
        // Divide range for left and right channels.
        let split_range_size = range.size() * vec2(0.5, 1.0);
        let left_range = Rect::from_min_size(range.left_top(), split_range_size);
        let right_range = Rect::from_min_size(
            range.left_top() + vec2(split_range_size.x, 0.0),
            split_range_size,
        );
        Shape::Vec(vec![
            self.create_channel_shape(left_range, left_level),
            self.create_channel_shape(right_range, right_level),
        ])
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
            let level_shapes = self.create_level_shape(range, left_level, right_level);
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
