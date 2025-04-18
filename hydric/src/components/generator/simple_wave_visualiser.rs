use egui::{Color32, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};
use std::f32::consts::PI;
use shared::model::WaveType;

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
            size: Vec2::new(130.0, 80.0),
        }
    }

    // Show the wave visualizer in the UI
    pub fn show(&self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());
        
        if ui.is_rect_visible(rect) {
            self.paint(ui, rect);
        }
        
        response
    }

    fn paint(&self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter();
        let center_y = rect.center().y;
        let center_x = rect.center().x;
        
        // Calculate wave points
        let points = self.calculate_wave_points(rect);

        match self.wave_type {
            WaveType::Sine => {
                let half_length = points.len() / 2;
                let fill_points_above = points[0..half_length].to_vec();
                let fill_points_below = points[half_length..].to_vec();
                painter.add(Shape::convex_polygon(
                    fill_points_above,
                    self.fill_color,
                    Stroke::NONE,
                ));
                painter.add(Shape::convex_polygon(
                    fill_points_below,
                    self.fill_color,
                    Stroke::NONE,
                ));
            },
            
            WaveType::Square => {
                let half_length = points.len() / 2;
                // area in the high
                let mut fill_points_above = points[0..half_length].to_vec();
                fill_points_above.push(Pos2::new(center_x, center_y));
                // area in the low
                let mut fill_points_below = Vec::with_capacity(half_length + 1);
                fill_points_below.push(Pos2::new(center_x, center_y));
                fill_points_below.extend(points[half_length..].iter());
                painter.add(Shape::convex_polygon(
                    fill_points_above,
                    self.fill_color,
                    Stroke::NONE,
                ));
                painter.add(Shape::convex_polygon(
                    fill_points_below,
                    self.fill_color,
                    Stroke::NONE,
                ));
            },
            
            WaveType::Saw => {
                let half_length = points.len() / 2;
                let fill_points_above = points[0..half_length].to_vec();
                let fill_points_below = points[half_length..].to_vec();
                painter.add(Shape::convex_polygon(
                    fill_points_above,
                    self.fill_color,
                    Stroke::NONE,
                ));
                painter.add(Shape::convex_polygon(
                    fill_points_below,
                    self.fill_color,
                    Stroke::NONE,
                ));

            },
            
            WaveType::Triangle => {
                let half_length = points.len() / 2;
                // area in the high
                let mut fill_points_above = points[0..half_length].to_vec();
                fill_points_above.push(Pos2::new(center_x, center_y));
                // area in the low
                let mut fill_points_below = Vec::with_capacity(half_length + 1);
                fill_points_below.push(Pos2::new(center_x, center_y));
                fill_points_below.extend(points[half_length..].iter());
                painter.add(Shape::convex_polygon(
                    fill_points_above,
                    self.fill_color,
                    Stroke::NONE,
                ));
                painter.add(Shape::convex_polygon(
                    fill_points_below,
                    self.fill_color,
                    Stroke::NONE,
                ));
            }
        }
        
        // Draw line of the actual wave
        painter.add(Shape::line(
            points,
            Stroke::new(3.0, self.line_color),
        ));
    }


    fn calculate_wave_points(&self, rect: Rect) -> Vec<Pos2> {
        let width = rect.width();
        let height = rect.height();
        let center_y = rect.center().y;
        let amplitude = height * 0.4; // Use 40% of height as amplitude
        let mut points = Vec::with_capacity(1);
        
        match self.wave_type {
            WaveType::Sine => {
                let num_points = 200;
                points = Vec::with_capacity(num_points);
                for i in 0..num_points {
                    let x = rect.left() + (i as f32 / (num_points - 1) as f32) * width;
                    let phase = (i as f32 / (num_points - 1) as f32) * 2.0 * PI;
                    let y = center_y - amplitude * phase.sin();
                    points.push(Pos2::new(x, y));
                }
            },
            
            WaveType::Square => {
                let num_points = 6;
                points = Vec::with_capacity(num_points);

                points.push(Pos2::new(rect.left(), center_y)); // let middle
                points.push(Pos2::new(rect.left(), center_y - amplitude)); // left top

                let mid_x = rect.left() + width * 0.5;
                points.push(Pos2::new(mid_x, center_y - amplitude)); // Last point at high
                points.push(Pos2::new(mid_x, center_y + amplitude)); // First point at low
                

                points.push(Pos2::new(rect.right(), center_y + amplitude)); // right bottom
                points.push(Pos2::new(rect.right(), center_y)); // right middle
            },
            
            WaveType::Saw => {
                let num_points = 200;
                points = Vec::with_capacity(num_points);

                points.push(Pos2::new(rect.left(), center_y)); // left middle
                
                // Sloping down to the bottom right
                for i in 1..(num_points-2) {
                    let x = rect.left() + (i as f32 / (num_points - 1) as f32) * width;
                    let normalized = i as f32 / (num_points - 1) as f32;
                    let y = center_y - amplitude + (2.0 * amplitude * normalized);
                    points.push(Pos2::new(x, y));
                }
                points.push(Pos2::new(rect.right(), center_y)); // right middle
            },
            
            WaveType::Triangle => {
                let num_points = 4;
                points = Vec::with_capacity(num_points);
                points.push(Pos2::new(rect.left(), center_y)); // left middle
                
                // Linear up to the peak at 1/4 of the wave
                let quarter_x = rect.left() + width * 0.25;
                points.push(Pos2::new(quarter_x, center_y - amplitude));
                
                // Linear down to the low point at 3/4 of the wave
                let three_quarter_x = rect.left() + width * 0.75;
                points.push(Pos2::new(three_quarter_x, center_y + amplitude));
                
                // Linear up to the end point at the center again
                points.push(Pos2::new(rect.right(), center_y));
            }
        }
        
        points
    }
}