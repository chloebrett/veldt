use crate::transform::Transform;
use egui::emath::RectTransform;
use egui::{Color32, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, lerp, pos2};
use mesic::wave::make_wave;
use shared::model::{AntiAliasingMode, WaveType};

// (potential) TODO: further generalise this to just WaveVisualiser so the painting logic can be resued in other components like ENV and LFO visualisers in the subsynth (might require calculating wave points outside of this component)
// ideas for things to make adjustable:
// make the 'x-axis' positioning flexible so the semi transparent painting of the area under the plotted line can change where it starts
// num points for larger visualisation components
pub struct SimpleWaveVisualiser {
    wave_type: WaveType,
    line_color: Color32,
    fill_color: Color32,
    size: Vec2,
}

impl SimpleWaveVisualiser {
    pub fn new(wave_type: WaveType, line_color: Color32, fill_color: Color32) -> Self {
        Self {
            wave_type,
            line_color,
            fill_color,
            size: Vec2::new(130.0, 74.0),
        }
    }

    // Show the wave visualiser in the UI.
    pub fn show(&self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());

        if ui.is_rect_visible(rect) {
            self.paint(ui, rect);
        }

        response
    }

    fn paint_segment(
        ui: &mut Ui,
        segment_wave_points: &[Pos2],
        fill_color: Color32,
        transform: RectTransform,
    ) {
        let painter = ui.painter();

        if segment_wave_points.len() < 2 {
            // Need at least two points on the wave curve to form a segment worth painting.
            return;
        }
        let mut polygon_points: Vec<Pos2> = segment_wave_points.to_vec();

        // Get the x-coordinates for the base of the polygon on the axis.
        let end_x = segment_wave_points.last().unwrap().x;

        // Add the points on the axis to close the polygon.
        polygon_points.push(pos2(end_x, 0.0));

        painter.add(Shape::convex_polygon(
            polygon_points.transform(transform),
            fill_color,
            Stroke::NONE,
        ));
    }

    fn paint(&self, ui: &mut Ui, rect: Rect) {
        let num_points = rect.width() as usize;

        // Map from wave space where x is in [0.0, 1.0] and y is in [-1.0, 1.0]
        // into screen space. Invert y axis (positive wave values should be above centre line).
        let transform =
            RectTransform::from_to(Rect::from_min_max(pos2(0.0, 1.0), pos2(1.0, -1.0)), rect);

        let points: Vec<_> = (0..num_points)
            .map(|i| {
                // This value should go from 0.0 to < 1.0 across the points because make_wave will then use (wave_input_x % 1.0) * TAU which means any int passed to it becomes 0.
                let x = i as f32 / num_points as f32;
                let y = make_wave(x, self.wave_type, 1.0, AntiAliasingMode::Off); // using arbitrary wave_freq since anti aliasing is off
                pos2(x, y)
            })
            .collect();

        // vector for each segment which will be used for painting in the area between the plotted line and the x-axis.
        let mut curr_points: Vec<Pos2> = vec![];

        // Iterate through generated points to build each polygon segment and find when the plotted point crosses the x-axis.
        for point in &points {
            if curr_points.is_empty() {
                // Always add the zero point as a base.
                // Also add the current point.
                curr_points.extend([Pos2::ZERO, *point]);
                continue;
            }

            let prev = *curr_points.last().unwrap();

            // If the current point and last in segment are on the same side of the axis
            if prev.y.signum() == point.y.signum() {
                curr_points.push(*point); // push point as usual if line hasn't crossed x-axis
                continue;
            }

            // Line crossed x-axis so calculate the intersection point on the axis.
            let diff = *point - prev;
            if diff.y.abs() <= 1e-6 {
                // If the y_diff.abs() is small then there is no point making a new segment yet because that means the interval is nearly horizontal.
                // i.e. crossing won't be visible anyway
                curr_points.push(*point);
                continue;
            }

            // Calculate the x-coordinate of the intersection point
            let t = (prev.y / diff.y).clamp(0.0, 1.0);
            let intersect = pos2(lerp(prev.x..=point.x, t), 0.0);

            // Add the intersection point to the current segment - this is the point that lies on the axis and marks the end of the current segment along the wave curve.
            curr_points.push(intersect);

            // Paint the completed segment's polygon.
            Self::paint_segment(ui, &curr_points, self.fill_color, transform);

            // Start a new segment with the intersection point and the current point (point).
            // Intersection point is the start of the new segment.
            curr_points = vec![intersect];

            // Add point to the new segment but only if it's different from the intersection point to avoid double ups incase p_current is already exactly on the x-axis
            if *point != intersect {
                curr_points.push(*point);
            }
        }

        // Paint the last segment if it exists
        if !curr_points.is_empty() {
            curr_points.push(pos2(1.0, 0.0)); // always add this base point (it's the end of the graph)
            Self::paint_segment(ui, &curr_points, self.fill_color, transform);
        }

        // Draw the line of the actual wave after filling so it appears on top
        ui.painter().add(Shape::line(
            points.transform(transform),
            Stroke::new(3.0, self.line_color),
        ));
    }
}
