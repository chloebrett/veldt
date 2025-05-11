use egui::InnerResponse;
use egui::{
    Color32, CornerRadius, Frame, Rect, Response, Sense, Shape, Ui, Vec2, Widget,
    emath::RectTransform, pos2, vec2,
};
use mesic::level::find_audio_level;

use crate::transform::Transform;

use crate::audio_state::AudioState;

/// Widget for rendering audio level in dB.
pub struct AudioLevel<'a> {
    audio_state: &'a AudioState,
    min_level: f32,
    max_level: f32,
    size: Vec2,
}

impl<'a> AudioLevel<'a> {
    pub fn new(audio_state: &'a AudioState) -> Self {
        Self {
            audio_state,
            min_level: -60.0,
            max_level: 20.0,
            size: vec2(40.0, 150.0),
        }
    }

    fn create_level_shapes(&self, range: Rect, left_level: f32, right_level: f32) -> Shape {
        // padding around level line.
        let side_padding = 0.1;
        let top_padding = 5.0;
        let bottom_padding = 10.0;
        let left_channel = |level, min_value| {
            Rect::from_min_max(
                pos2(range.left() + side_padding, level - top_padding),
                pos2(
                    range.left() + (range.size().x - side_padding) * 0.5,
                    min_value + bottom_padding,
                ),
            )
        };
        let right_channel = |level, min_value| {
            Rect::from_min_max(
                pos2(range.left() + (range.size().x + side_padding) * 0.5, level - top_padding),
                pos2(
                    range.right() - side_padding,
                    min_value + bottom_padding,
                ),
            )
        };
        let left_level_rect = left_channel(left_level, range.bottom());
        let left_background_rect = left_channel(range.top(), range.bottom());
        let right_level_rect = right_channel(right_level, range.bottom());
        let right_background_rect = right_channel(range.top(), range.bottom());
        let left_marker = left_channel(left_level + 1.0, left_level);
        let right_marker = right_channel(right_level + 1.0, right_level - 1.0);
        let level_shape = |rect| Shape::rect_filled(rect, CornerRadius::same(0), Color32::WHITE);
        let background_shape =
            |rect| Shape::rect_filled(rect, CornerRadius::same(0), Color32::from_black_alpha(64));
        let marker_shape = |rect| Shape::rect_filled(rect, CornerRadius::same(0), Color32::BLACK);
        Shape::Vec(vec![
            background_shape(left_background_rect),
            background_shape(right_background_rect),
            level_shape(left_level_rect),
            level_shape(right_level_rect),
            marker_shape(left_marker),
            marker_shape(right_marker),
        ])
    }
}

impl Widget for AudioLevel<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let AudioLevel {
            audio_state,
            min_level,
            max_level,
            size,
        } = self;
        let range = Rect::from_min_max(pos2(0.0, max_level), pos2(1.0, min_level));
        // Get the level of audio channels.
        let (left_level, right_level) = if audio_state.audio.is_empty() {
            (-20.0, 2.0)
        } else {
            find_audio_level(audio_state.audio.to_vec())
        };
        let InnerResponse { inner: _, response } = Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(size, Sense::all());
            let number_level_divide = range.size().x * 0.5;
            let (number_range, level_range) = (
                range.with_max_x(number_level_divide),
                range.with_min_x(number_level_divide),
            );
            log::debug!("{:?}", level_range);
            let level_shapes = self.create_level_shapes(level_range, left_level, right_level);
            log::debug!("{:?}", level_shapes);
            let to_screen = RectTransform::from_to(range, response.rect);
            painter.add(Shape::rect_filled(
                response.rect,
                CornerRadius::same(0),
                Color32::from_white_alpha(16),
            ));
            painter.add(level_shapes.transform(to_screen))
        });
        response
    }
}
