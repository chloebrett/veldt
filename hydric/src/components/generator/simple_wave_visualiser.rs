use crate::transform::Transform;
use egui::emath::RectTransform;
use egui::{Color32, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, lerp, pos2};
use mesic::wave::make_basic_wave_with_frequency;
use shared::model::WaveType;

// (potential) TODO: further generalise this to just WaveVisualiser so the painting logic can be resued in other components like ENV and LFO visualisers in the subsynth (might require calculating wave points outside of this component)
// ideas for things to make adjustable:
// make the 'x-axis' positioning flexible so the semi transparent painting of the area under the plotted line can change where it starts
// num points for larger visualisation components
pub struct SimpleWaveVisualiser {
    wave_type: WaveType,
    line_color: Color32,
    fill_color: Color32,
    size: Vec2,
    frequency: f32,
}

impl SimpleWaveVisualiser {
    pub fn new(
        wave_type: WaveType,
        line_color: Color32,
        fill_color: Color32,
        frequency: f32,
        size: Vec2,
    ) -> Self {
        Self {
            wave_type,
            line_color,
            fill_color,
            frequency,
            size,
        }
    }

    // Show the wave visualiser in the UI.
    pub fn show(&self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::empty());

        if ui.is_rect_visible(rect) {
            self.paint(ui, rect);
        }

        response
    }

    fn paint(&self, ui: &mut Ui, screen_rect: Rect) {
        let num_points = screen_rect.width() as usize;
        let points = self.wave_points(num_points);

        // Map from wave space where x is in [0.0, 1.0] and y is in [-1.0, 1.0]
        // into screen space. Invert y axis (positive wave values should be above centre line).
        let wave_rect = Rect::from_min_max(pos2(0.0, 1.0), pos2(1.0, -1.0));
        let transform = RectTransform::from_to(wave_rect, screen_rect);

        let fill: Vec<_> = self
            .wave_fill_segments(&points)
            .into_iter()
            .map(|segment| {
                Shape::convex_polygon(segment.transform(transform), self.fill_color, Stroke::NONE)
            })
            .collect();

        let stroke = Shape::line(
            points.transform(transform),
            Stroke::new(3.0, self.line_color),
        );

        let painter = ui.painter();
        painter.extend(fill);
        painter.add(stroke);
    }

    fn wave_points(&self, num_points: usize) -> Vec<Pos2> {
        (0..num_points)
            .map(|i| {
                // This value should go from 0.0 to < 1.0 across the points because make_wave will then use (wave_input_x % 1.0) * TAU which means any int passed to it becomes 0.
                let x = i as f32 / num_points as f32;
                let y = make_basic_wave_with_frequency(x, self.wave_type, self.frequency);
                pos2(x, y)
            })
            .collect()
    }

    fn wave_fill_segments(&self, points: &[Pos2]) -> Vec<Vec<Pos2>> {
        let mut segments: Vec<Vec<Pos2>> = vec![];

        // vector for each segment which will be used for painting in the area between the plotted line and the x-axis.
        let mut curr_points: Vec<Pos2> = vec![];

        // Iterate through generated points to build each polygon segment and find when the plotted point crosses the x-axis.
        for point in points {
            if curr_points.is_empty() {
                // Always add the zero point as a base.
                // Also add the current point.
                curr_points.extend([Pos2::ZERO, *point]);
                continue;
            }

            let prev = *curr_points.last().unwrap();

            // if current point and prev point are the same then just skip the current point
            if prev == *point {
                continue;
            }

            // If the current point and last in segment are on the same side of the axis
            if prev.y.signum() == point.y.signum() && point.y != 0.0 {
                curr_points.push(*point); // push point as usual if line hasn't crossed x-axis
                continue;
            }

            // Line crossed x-axis so calculate the intersection point on the axis.
            let diff = *point - prev;

            // Calculate the x-coordinate of the intersection point
            let t = (prev.y / diff.y).clamp(0.0, 1.0);
            let intersect = pos2(lerp(prev.x..=point.x, t), 0.0);

            // Add the intersection point to the current segment - this is the point that lies on the axis and marks the end of the current segment along the wave curve.
            if intersect != prev {
                curr_points.push(intersect);
            }

            // Manually close the polygon as long as it's not the first segment
            if !segments.is_empty() {
                let starting_point = curr_points.first().unwrap();
                curr_points.push(*starting_point);
            }

            // Save this segment.
            segments.push(curr_points);

            // Start a new segment with the intersection point and the current point (point).
            // Intersection point is the start of the new segment.
            curr_points = vec![intersect];

            // Add point to the new segment but only if it's different from the intersection point to avoid double ups incase p_current is already exactly on the x-axis
            if *point != intersect {
                curr_points.push(*point);
            }
        }

        // Manually add the last point to the last segment and add it to the rest of the segments but onlf if there are more than two points in the last unpushed segment.
        if curr_points.len() > 2 {
            let prev = curr_points.last().unwrap();
            let last_point = pos2(prev.x, 0.0);
            if *prev != last_point {
                curr_points.push(last_point);
            }
            let starting_point = curr_points.first().unwrap();
            curr_points.push(*starting_point);
            segments.push(curr_points);
        }

        segments
    }
}
