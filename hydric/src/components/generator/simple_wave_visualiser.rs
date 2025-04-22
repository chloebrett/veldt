use egui::{Color32, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};
use shared::model::{WaveType, AntiAliasingMode};
use mesic::wave::make_wave;

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

    // Show the wave visualiser in the UI
    pub fn show(&self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());

        if ui.is_rect_visible(rect) {
            self.paint(ui, rect);
        }

        response
    }

    fn paint(&self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter();
        let axis_y = rect.center().y;

        // Calculate wave points
        let num_points = 80;
        let mut points = Vec::with_capacity(num_points);
        for i in 0..80{
            let mapped_x_rect = rect.left() + (i as f32 / (num_points - 1) as f32) * rect.width();

            // this value should go from 0.0 to < 1.0 across the points because make_wave will then use (wave_input_x % 1.0) * TAU which means any int passed to it becomes 0
            let wave_input_x = i as f32 / num_points as f32;
            let y = make_wave(wave_input_x, self.wave_type, 1.0, AntiAliasingMode::Off); // using arbitrary wave_freq since anti aliasing is off

            // Map the wave's y output (-1.0 to 1.0) to the rectangle's y-range
            let mapped_y_rect = axis_y - y * (rect.height() / 2.0);
            points.push(Pos2::new(mapped_x_rect, mapped_y_rect));
            
        }
        
        // vector for each segment which will be used for painting in the area between the plotted line and the x-axis
        let mut current_segment_points: Vec<Pos2> = vec![];

        // paint_segment will paint in each segment
        let paint_segment = |segment_wave_points: &[Pos2], fill_color: Color32, axis_y: f32| {
            if segment_wave_points.len() < 2 {
                // Need at least two points on the wave curve to form a segment worth painting
                return;
            }
            let mut polygon_points: Vec<Pos2> = segment_wave_points.to_vec();

            // Get the x-coordinates for the base of the polygon on the axis
            let start_x = segment_wave_points[0].x;
            let end_x = segment_wave_points.last().unwrap().x;

            // add the points on the axis to close the polygon
            polygon_points.push(Pos2::new(end_x, axis_y));
            polygon_points.push(Pos2::new(start_x, axis_y));

            painter.add(Shape::convex_polygon(
                polygon_points,
                fill_color,
                Stroke::NONE,
            ));
        };
        // iterate through generated points to build each polygon segment and find when the plotted point crosses the x-axis
        for i in 0..num_points {
            let p_current = points[i];
            // Determine if the current point is above or below the axis
            let current_is_above = p_current.y <= axis_y; // y increases downwards!!

            if current_segment_points.is_empty() {
                 current_segment_points.push(p_current); // adding first point
            } else {
                let p_last_in_segment = *current_segment_points.last().unwrap();
                let last_is_above = p_last_in_segment.y <= axis_y;

                if current_is_above == last_is_above {
                    current_segment_points.push(p_current); // push point as usual if line hasn't crossed x-axis
                } else {
                    // line corrsed x-axis so calculate the intersection point on the axis
                    let y_diff = p_current.y - p_last_in_segment.y;
                    if y_diff.abs() > 1e-6 { // only create a new segment if y-diff interval is meaninfgully large
                       // Calculate the x-coordinate of the intersection point
                       let t = (axis_y - p_last_in_segment.y) / y_diff;
                       let t = t.clamp(0.0, 1.0);
                       let intersect_x = p_last_in_segment.x + (p_current.x - p_last_in_segment.x) * t;
                       let p_int = Pos2::new(intersect_x, axis_y);

                       // add the intersection point to the current segment - this is the point that lies on the axis and marks the end of the current segment along the wave curve
                       current_segment_points.push(p_int);

                       // Paint the completed segment's polygon
                       paint_segment(&current_segment_points, self.fill_color, axis_y);

                       // Start a new segment with the intersection point and the current point (p_current)
                       current_segment_points.clear(); // Clear points from the just-painted segment
                       current_segment_points.push(p_int); // Intersection point is the start of the new segment

                       // add p_current to the new segment but only if it's different from the intersection point to avoid double ups incase p_current is already exactly on the x-axis
                       if p_current != p_int {
                          current_segment_points.push(p_current);
                       }

                    } else {
                         // if the y_diff.abs() is small then there is no point making a new segment yet because that means the interval is nearly horizontal
                         // i.e. crossing won't be visible anyway
                         current_segment_points.push(p_current);
                    }
                }
            }
        }

        // Paint the last segment if it exists
        if !current_segment_points.is_empty() {
             paint_segment(&current_segment_points, self.fill_color, axis_y);
        }

         // Draw the line of the actual wave after filling so it appears on top
         painter.add(Shape::line(points, Stroke::new(3.0, self.line_color)));
    }

}
