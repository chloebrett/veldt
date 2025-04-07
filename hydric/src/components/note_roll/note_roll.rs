use egui::{emath::RectTransform, vec2, Color32, CornerRadius, Frame, Pos2, Rect, Response, Sense, Shape, Ui, Vec2, Widget};
use state::{Action, Selector};
use crate::transform::Transform;

pub struct Sequencer<F: FnOnce(Action, Selector)> {
    range: Rect,
    size: Option<Vec2>,
    rects: Option<Vec<Rect>>,
    sense: Option<Sense>,
    dispatcher: F,
}

impl<F> Sequencer<F> where F: FnOnce(Action, Selector) {
    pub fn new(range: Rect, dispatcher: F) -> Self {
        Sequencer { 
            range, 
            size: None,
            rects: None,
            sense: None,
            dispatcher,
        }
    }

    pub fn rects(mut self, rects: Vec<Rect>) -> Self {
        self.rects = Some(rects);
        self
    }

}

impl<F> Widget for Sequencer<F> where F: FnOnce(Action, Selector) {
    fn ui(self, ui: &mut Ui) -> Response {
        let Sequencer { range, size, rects, sense, dispatcher } = self;
        let size = size.unwrap_or(vec2(ui.available_width(), 400.0));
        let rects = rects.unwrap_or(vec![]);
        let sense = sense.unwrap_or(Sense::drag());
        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(size, sense);
            let sequencer_transform = RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, range.size()),
                response.rect
            );
            let shapes: Vec<Shape> = rects.iter().map(|&rect| {
                Shape::rect_filled(rect, CornerRadius::same(1), Color32::WHITE)
            }).collect();
            painter.extend(shapes.transform(sequencer_transform))
        });
        let (rect, response) = ui.allocate_at_least(size, sense);
        return response
    }

}
