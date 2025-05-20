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
            min_level: -100.0,
            max_level: 10.0,
            size: vec2(50.0, 200.0),
        }
    }

    /// Convert a level from dB to a [0, 1] using a piecewise function.
    /// Spreads out dB closer to 0 and condenses smaller dB.
    /// Function is continuous but uses different gradients over different ranges.
    /// 0 is top of the level, 1 is the bottom.
    // TODO: Can this be better generalised?
    fn convert_level(&self, level: f32) -> f32 {
        if level < -100.0 {
            // Smaller signals clipped at bottom of level.
            1.0
        } else if level < -50.0 {
            // 1.0 to 0.875
            (300.0 - level) / 400.0
        } else if level < -30.0 {
            // 0.875 to 0.75
            (90.0 - level) / 160.0
        } else if level < -10.0 {
            // 0.75 to 0.5
            (30.0 - level) / 80.0
        } else if level < 10.0 {
            // 0.5 to 0.0
            (10.0 - level) / 40.0
        } else {
            // Larger signals clipped at top of level.
            0.0
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
        let marker_height = 0.01;
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
            left_range.center_bottom() + vec2(padding * 0.25, 0.05),
            "L".into(),
        );
        let right_label = self.create_channel_label(
            ui,
            to_screen,
            right_range.center_bottom() + vec2(0.0 - padding * 0.25, 0.05),
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
        let text_markers = [self.max_level as i32, 5, 0, -5, -10, -20, -30, -50, -100];
        let shapes: Vec<_> = text_markers
            .iter()
            .map(|&marker| {
                let level = self.convert_level(marker as f32) as f32;
                let text: WidgetText = if marker > 0 {
                    format!("+{marker}").into()
                } else if marker == -100 {
                    "-∞".into()
                } else {
                    format!("{marker}").into()
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
                let text_pos = pos2(text_range.left() + left_padding, level as f32 - offset);
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
        let range = Rect::from_min_max(
            // Convert max and min level to set y-range to [0, 1].
            pos2(0.0, self.convert_level(max_level)),
            pos2(1.0, self.convert_level(min_level)),
        );
        // Aesthetic padding around channel shapes top and bottom.
        let top_padding = 0.05;
        let bottom_padding = 0.1;
        let padding_transform = RectTransform::from_to(
            range,
            Rect::from_min_max(
                pos2(range.left(), range.top() + top_padding),
                pos2(range.right(), range.bottom() - bottom_padding),
            ),
        );
        // Get the level of audio channels.
        let [left_level, right_level] = player.level();
        // Convert levels to [0, 1] range.
        let left_level = self.convert_level(left_level);
        let right_level = self.convert_level(right_level);
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
