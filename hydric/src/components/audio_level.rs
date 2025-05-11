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
            size: vec2(20.0, 150.0),
        }
    }

    fn create_level_shapes(&self, range: Rect, left_level: f32, right_level: f32) -> Shape {
        // Padding on either side of the level line.
        let padding = 0.1;
        let left_rect = Rect::from_min_max(
            pos2(range.left() + padding, left_level),
            pos2((range.size().x - padding) * 0.5, range.bottom()),
        );
        let right_rect = Rect::from_min_max(
            pos2((range.size().x + padding) * 0.5, right_level),
            pos2(range.right() - padding, range.bottom()),
        );
        let level_shape = |rect| Shape::rect_filled(rect, CornerRadius::same(0), Color32::WHITE);
        Shape::Vec(vec![level_shape(left_rect), level_shape(right_rect)])
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
            (min_level, min_level)
        } else {
            find_audio_level(audio_state.audio.to_vec())
        };
        let InnerResponse { inner: _, response } = Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(size, Sense::all());
            let level_shapes = self.create_level_shapes(range, left_level, right_level);
            let to_screen = RectTransform::from_to(range, response.rect);
            painter.add(level_shapes.transform(to_screen))
        });
        response
    }
}
