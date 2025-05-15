use std::f32;

use egui::text::Fonts;
use egui::{
    Align2, FontDefinitions, FontId, FontSelection, InnerResponse, Rangef, RichText, WidgetText,
};
use egui::{
    Color32, CornerRadius, Frame, Rect, Response, Sense, Shape, Ui, Vec2, Widget,
    emath::RectTransform, pos2, vec2,
};

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
            max_level: 2.0,
            size: vec2(40.0, 150.0),
        }
    }

    fn create_level_shapes(&self, range: Rect, left_level: f32, right_level: f32) -> Shape {
        // padding around level line.
        let side_padding = 0.1;
        let top_padding = 5.0;
        let bottom_padding = 10.0;
        let padded_y_size =
            (range.size().y.abs() - (top_padding + bottom_padding)) / range.size().y.abs();
        log::debug!("{:?}", padded_y_size);
        let left_channel = |level, min_value| {
            Rect::from_x_y_ranges(
                Rangef {
                    min: range.left() + side_padding,
                    max: range.size().x + (range.size().x - side_padding) * 0.5,
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
                    min: range.size().x + (range.size().x + side_padding) * 0.5,
                    max: range.right() - side_padding,
                },
                Rangef {
                    min: level * padded_y_size - top_padding,
                    max: min_value * padded_y_size - top_padding,
                },
            )
        };
        let left_level_rect = left_channel(left_level, range.bottom());
        let left_background_rect = left_channel(range.top(), range.bottom());
        let right_level_rect = right_channel(right_level, range.bottom());
        let right_background_rect = right_channel(range.top(), range.bottom());
        let left_marker = left_channel(left_level + 0.5, left_level - 0.5);
        let right_marker = right_channel(right_level + 0.5, right_level - 0.5);
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

    fn create_increment_text(&self, to_screen: RectTransform, range: Rect) -> Shape {
        let fonts = Fonts::new(1.0, 100, FontDefinitions::default());
        Shape::text(
            &fonts,
            range.left_center().transform(to_screen),
            Align2::CENTER_TOP,
            "*",
            FontId::default(),
            Color32::WHITE,
        )
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
            let number_level_divide = range.size().x * 0.5;
            let (number_range, level_range) = (
                range.with_max_x(number_level_divide),
                range.with_min_x(number_level_divide),
            );
            let level_shapes = self.create_level_shapes(level_range, left_level, right_level);
            let to_screen = RectTransform::from_to(range, response.rect);
            let text_shape = self.create_increment_text(to_screen, number_range);
            painter.add(Shape::rect_filled(
                response.rect,
                CornerRadius::same(0),
                Color32::from_white_alpha(16),
            ));
            painter.add(level_shapes.transform(to_screen));
            painter.add(text_shape);
        });
        response
    }
}
