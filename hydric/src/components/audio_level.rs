use egui::{
    Color32, CornerRadius, Frame, Rect, Response, Sense, Shape, Ui, Vec2, Widget,
    emath::RectTransform, pos2, vec2,
};
use egui::{InnerResponse, Rangef};
use mesic::level::calc_audio_level;

use crate::playback::AudioPlayer;
use crate::transform::Transform;

use ringbuffer::RingBuffer;

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
        // Padding on either side of the level line.
        let padding = 0.1;
        let left_rect = Rect::from_x_y_ranges(
            Rangef {
                min: range.left() + padding,
                max: (range.size().x - padding) * 0.5,
            },
            Rangef {
                min: left_level,
                max: range.bottom(),
            },
        );
        let right_rect = Rect::from_x_y_ranges(
            Rangef {
                min: (range.size().x + padding) * 0.5,
                max: range.right() - padding,
            },
            Rangef {
                min: right_level,
                max: range.bottom(),
            },
        );
        let level_shape: Vec<_> = vec![left_rect, right_rect]
            .into_iter()
            .map(|rect| Shape::rect_filled(rect, CornerRadius::ZERO, Color32::WHITE))
            .collect();
        Shape::Vec(level_shape)
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
        // Take last 64 samples from audio.
        let audio: Vec<[f32; 2]> = player.recent_buf().iter().map(|it| *it).collect();
        let window = &audio[audio.len() - 64..];
        // Get the level of audio channels.
        let (left_level, right_level) = calc_audio_level(window);
        let InnerResponse { inner: _, response } = Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(size, Sense::all());
            let level_shapes = self.create_level_shapes(range, left_level, right_level);
            let to_screen = RectTransform::from_to(range, response.rect);
            painter.add(level_shapes.transform(to_screen))
        });
        response
    }
}
