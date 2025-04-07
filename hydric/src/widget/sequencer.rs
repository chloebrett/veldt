use crate::transform::Transform;
use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Widget,
    emath::RectTransform, pos2, vec2,
};
use state::Selector;

pub struct Sequencer<F: Fn(&Selector, f32), G: Fn(&Selector, f32)> {
    range: Rect,
    size: Vec2,
    rects: Vec<Rect>,
    sense: Sense,
    dispatch_x: F,
    dispatch_y: G,
    background_shapes: Vec<Shape>,
}

impl<F: Fn(&Selector, f32), G: Fn(&Selector, f32)> Sequencer<F, G> {
    pub fn new(range: Rect, dispatch_x: F, dispatch_y: G) -> Self {
        Sequencer {
            range,
            size: vec2(400.0, 600.0),
            rects: vec![],
            sense: Sense::drag(),
            dispatch_x,
            dispatch_y,
            background_shapes: vec![],
        }
    }

    #[inline]
    pub fn rects(mut self, rects: Vec<Rect>) -> Self {
        self.rects = rects;
        self
    }

    #[inline]
    pub fn vertical_bars(mut self, increment: f32, colour: Color32) -> Self {
        let steps = (self.range.size().x / increment) as i32;
        let shapes: Vec<Shape> = (0..=steps)
            .map(|step| {
                let x = (step as f32) * increment;
                Shape::line_segment(
                    [pos2(x, 0.0), pos2(x, self.range.size().y)],
                    Stroke::new(1.0, colour),
                )
            })
            .collect();
        self.background_shapes.extend(shapes);
        self
    }

    #[inline]
    pub fn horizontal_rects(mut self, increment: f32, colour: Color32) -> Self {
        let steps = (self.range.size().y / increment) as i32;
        let shapes: Vec<Shape> = (0..=steps)
            .map(|step| {
                let y = (step as f32) * increment;
                let rect =
                    Rect::from_min_size(pos2(self.range.left(), y), vec2(self.range.size().x, 1.0));
                Shape::rect_filled(rect, CornerRadius::ZERO, colour)
            })
            .collect();
        self.background_shapes.extend(shapes);
        self
    }

    fn update_rect(
        &self,
        rect: Rect,
        rect_index: usize,
        ui: &mut Ui,
        response: &Response,
        sequencer_transform: &RectTransform,
    ) {
        let track_index = 0;
        let id = response.id.with(rect_index);
        let rect_response = ui.interact(rect.transform(*sequencer_transform), id, self.sense);
        let drag_pos = rect_response.interact_pointer_pos();
        let drag_delta = rect_response.drag_delta();
        if let Some(pos) = drag_pos {
            let scaled_pos = pos
                .transform(sequencer_transform.inverse())
                .clamp(pos2(0.0, 0.0), self.range.size().to_pos2());
            if drag_delta != Vec2::ZERO {
                let sel = Selector::Note(track_index, rect_index);
                if drag_delta.x != 0.0 {
                    (self.dispatch_x)(&sel, scaled_pos.x)
                };
                if drag_delta.y != 0.0 {
                    (self.dispatch_y)(&sel, scaled_pos.y)
                }
            }
        }
    }
}

impl<F: Fn(&Selector, f32), G: Fn(&Selector, f32)> Widget for Sequencer<F, G> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Sequencer {
            range,
            size,
            ref rects,
            sense,
            dispatch_x: _,
            dispatch_y: _,
            ref background_shapes,
        } = self;
        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(size, sense);
            let sequencer_transform = RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, range.size()),
                response.rect,
            );
            let shapes: Vec<Shape> = rects
                .into_iter()
                .enumerate()
                .map(|(index, rect)| {
                    self.update_rect(*rect, index, ui, &response, &sequencer_transform);
                    Shape::rect_filled(*rect, CornerRadius::same(1), Color32::WHITE)
                })
                .collect();
            painter.extend(background_shapes.clone().transform(sequencer_transform));
            painter.extend(shapes.transform(sequencer_transform))
        });
        let (_rect, response) = ui.allocate_at_least(Vec2::ZERO, sense);
        response
    }
}
