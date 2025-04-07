use crate::transform::Transform;
use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Widget,
    emath::RectTransform, pos2, vec2,
};
use state::Selector;

pub struct Sequencer<F: Fn(&Selector, Pos2)> {
    range: Rect,
    size: Option<Vec2>,
    rects: Option<Vec<Rect>>,
    sense: Option<Sense>,
    dispatch: F,
    background_shapes: Option<Vec<Shape>>,
}

impl<F: Fn(&Selector, Pos2)> Sequencer<F> {
    pub fn new(range: Rect, dispatch: F) -> Self {
        Sequencer {
            range,
            size: None,
            rects: None,
            sense: None,
            dispatch,
            background_shapes: None,
        }
    }

    #[inline]
    pub fn rects(mut self, rects: Vec<Rect>) -> Self {
        self.rects = Some(rects);
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
        let mut background_shapes = self.background_shapes.unwrap_or_default();
        background_shapes.extend(shapes);
        self.background_shapes = Some(background_shapes);
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
        let mut background_shapes = self.background_shapes.unwrap_or_default();
        background_shapes.extend(shapes);
        self.background_shapes = Some(background_shapes);
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
        let rect_response = ui.interact(
            rect.transform(*sequencer_transform),
            id,
            self.sense.unwrap_or(Sense::drag()),
        );
        let drag_pos = rect_response.interact_pointer_pos();
        let drag_delta = rect_response.drag_delta();
        if let Some(pos) = drag_pos {
            let scaled_pos = pos
                .transform(sequencer_transform.inverse())
                .clamp(pos2(0.0, 0.0), self.range.size().to_pos2());
            if drag_delta != Vec2::ZERO {
                let sel = Selector::Note(track_index, rect_index);
                (self.dispatch)(&sel, scaled_pos)
            }
        }
    }
}

impl<F: Fn(&Selector, Pos2)> Widget for Sequencer<F> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Sequencer {
            range,
            size,
            ref rects,
            sense,
            dispatch: _,
            ref background_shapes,
        } = self;
        let size = size.unwrap_or(vec2(400.0, 600.0));
        let rects = rects.clone().unwrap_or_default();
        let sense = sense.unwrap_or(Sense::drag());
        let background_shapes = background_shapes.clone().unwrap_or(vec![]);
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
                    self.update_rect(rect, index, ui, &response, &sequencer_transform);
                    Shape::rect_filled(rect, CornerRadius::same(1), Color32::WHITE)
                })
                .collect();
            painter.extend(background_shapes.transform(sequencer_transform));
            painter.extend(shapes.transform(sequencer_transform))
        });
        let (_rect, response) = ui.allocate_at_least(Vec2::ZERO, sense);
        response
    }
}
