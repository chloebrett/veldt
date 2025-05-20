use egui::epaint::TextShape;
use egui::text::TextWrapping;
use egui::{Align, InnerResponse, Pos2, Rangef, Stroke, TextStyle, WidgetText};
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
            max_level: 6.0,
            size: vec2(50.0, 200.0),
        }
    }

    fn create_channel_shape(
        &self,
        range: Rect,
        level: f32,
        left_padding: f32,
        right_padding: f32,
    ) -> Shape {
        // Marker will be twice this height.
        let marker_height = 0.5;
        let channel_rect = |top, bottom| {
            Rect::from_x_y_ranges(
                Rangef {
                    min: range.left() + left_padding,
                    max: range.right() - right_padding,
                },
                Rangef {
                    min: top,
                    max: bottom,
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

    fn create_channel_label(
        &self,
        ui: &mut Ui,
        to_screen: RectTransform,
        pos: Pos2,
        label: String,
    ) -> Shape {
        let text: WidgetText = label.clone().into();
        let galley = text.into_galley_impl(
            ui.ctx(),
            ui.style(),
            TextWrapping::no_max_width(),
            TextStyle::Small.into(),
            Align::Center,
        );
        // Offset pos so text appears with value in middle.
        // Galley size is in actual size units and is not scaled by to_screen transform.
        // Inverse transform size to get offset ammount.
        let half_galley_size = vec2(
            0.5 * galley.rect.transform(to_screen.inverse()).size().x,
            0.0,
        );
        TextShape::new(pos - half_galley_size, galley, ui.visuals().text_color()).into()
    }

    /// Create level shapes that show level in dB.
    fn create_level_shape(
        &self,
        ui: &mut Ui,
        to_screen: RectTransform,
        range: Rect,
        left_level: f32,
        right_level: f32,
    ) -> Shape {
        // Divide range for left and right channels.
        let (left_range, right_range) = range.split_left_right_at_fraction(0.5);
        // Padding on left and right of channels.
        let padding = 0.2 * range.size().x;
        // Add L and R labels to channels.
        let left_label = self.create_channel_label(
            ui,
            to_screen,
            left_range.center_bottom() + vec2(padding * 0.25, -2.5),
            "L".into(),
        );
        let right_label = self.create_channel_label(
            ui,
            to_screen,
            right_range.center_bottom() + vec2(0.0 - padding * 0.25, -2.5),
            "R".into(),
        );
        Shape::Vec(vec![
            self.create_channel_shape(left_range, left_level, padding, padding * 0.5),
            self.create_channel_shape(right_range, right_level, padding * 0.5, padding),
            left_label,
            right_label,
        ])
    }

    /// Add text and lines to mark level value.
    // TODO: should level lines be made in their own method?
    fn create_text_shape(&self, ui: &mut Ui, to_screen: RectTransform, range: Rect) -> Shape {
        let left_padding = 0.1;
        let right_padding = 0.0;
        // Divide range for text and level line.
        let (text_range, line_range) = range.split_left_right_at_fraction(0.85);
        let label_step = 10;
        let shapes: Vec<Shape> = (range.bottom() as i32..=range.top() as i32)
            .filter(|&level| level % label_step == 0 || level == range.top() as i32)
            .map(|level| {
                let text: WidgetText = if level > 0 {
                    format!("+{level}").into()
                } else {
                    format!("{level}").into()
                };
                let galley = text.into_galley_impl(
                    ui.ctx(),
                    ui.style(),
                    TextWrapping::no_max_width(),
                    TextStyle::Small.into(),
                    Align::Center,
                );
                // Offset pos so text appears wtih value in middle.
                // TODO: Understand this better so it can be properly aligned.
                // 0.66 * galley rect height is what makes it look like the centre but why?
                let offset = galley.rect.transform(to_screen.inverse()).size().y.abs() * 0.66;
                let text_pos = pos2(text_range.left() + left_padding, level as f32 + offset);
                Shape::Vec(vec![
                    TextShape::new(text_pos, galley, ui.visuals().text_color()).into(),
                    // Line to indicate level in line with text.
                    Shape::line(
                        vec![
                            pos2(line_range.left(), level as f32),
                            pos2(line_range.right() - right_padding, level as f32),
                        ],
                        Stroke::new(1.0, Color32::from_white_alpha(32)),
                    ),
                ])
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
        // Aesthetic padding around channel shapes top and bottom.
        let top_padding = 5.0;
        let bottom_padding = 10.0;
        let padding_transform = RectTransform::from_to(
            range,
            Rect::from_min_max(
                pos2(range.left(), range.top() - top_padding),
                pos2(range.right(), range.bottom() + bottom_padding),
            ),
        );
        // Get the level of audio channels.
        let [left_level, right_level] = player.level();
        let InnerResponse { inner: _, response } = Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(size, Sense::all());
            let to_screen = RectTransform::from_to(range, response.rect);
            let text_level_split: f32 = 0.5;
            // Split range for text increments and level bars.
            let (text_range, level_range) = range.split_left_right_at_fraction(text_level_split);
            let level_shapes =
                self.create_level_shape(ui, to_screen, level_range, left_level, right_level);
            let text_shapes = self.create_text_shape(ui, to_screen, text_range);
            // Add background shape.
            painter.add(Shape::rect_filled(
                response.rect,
                CornerRadius::same(0),
                Color32::from_white_alpha(4),
            ));
            painter.add(
                text_shapes
                    .transform(padding_transform)
                    .transform(to_screen),
            );
            painter.add(
                level_shapes
                    .transform(padding_transform)
                    .transform(to_screen),
            );
        });
        response
    }
}
