use crate::transform::Transform;
use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Response, Sense, Shape, Ui, Vec2, Widget,
    emath::{OrderedFloat, RectTransform},
    pos2, vec2,
};
use shared::{logger, model::PitchName, types::PitchValue};
use state::{Action, Selector};

pub struct Sequencer<F: Fn(&Selector, Action)> {
    range: Rect,
    size: Option<Vec2>,
    rects: Option<Vec<Rect>>,
    sense: Option<Sense>,
    dispatch: F,
}

impl<F: Fn(&Selector, Action)> Sequencer<F> {
    pub fn new(range: Rect, dispatch: F) -> Self {
        Sequencer {
            range,
            size: None,
            rects: None,
            sense: None,
            dispatch,
        }
    }

    #[inline]
    pub fn add_rects(mut self, rects: Vec<Rect>) -> Self {
        self.rects = Some(rects);
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
        let rect_response = ui.interact(rect, id, self.sense.unwrap_or(Sense::drag()));
        let drag_pos = rect_response.interact_pointer_pos();
        let drag_delta = rect_response.drag_delta();
        logger::log(&format!("{:?}", rect_response));
        if let Some(pos) = drag_pos {
            let scaled_pos = pos
                .transform(sequencer_transform.inverse())
                .clamp(pos2(0.0, 0.0), self.range.size().to_pos2());
            let offset = scaled_pos.x;
            let pitch_value: PitchValue = (scaled_pos.y as i32).into();
            let pitch_name = PitchName::from(self.range.bottom() as i32 - pitch_value);
            if drag_delta != Vec2::ZERO {
                let sel = Selector::Note(track_index, rect_index);
                if drag_delta.x != 0.0 {
                    (self.dispatch)(&sel, Action::SetNoteOffset(offset))
                }
                if drag_delta.y != 0.0 {
                    (self.dispatch)(&sel, Action::SetNoteOctave(pitch_name.octave));
                    (self.dispatch)(&sel, Action::SetNoteScaleValue(pitch_name.scale_value))
                }
            }
        }
    }
}

impl<F: Fn(&Selector, Action)> Widget for Sequencer<F> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Sequencer {
            range,
            size,
            ref rects,
            sense,
            ref dispatch,
        } = self;
        let size = size.unwrap_or(vec2(400.0, 400.0));
        let rects = rects.clone().unwrap_or(vec![]);
        let sense = sense.unwrap_or(Sense::drag());
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
                    self.update_rect(
                        rect.transform(sequencer_transform),
                        index,
                        ui,
                        &response,
                        &sequencer_transform,
                    );
                    Shape::rect_filled(rect, CornerRadius::same(1), Color32::WHITE)
                })
                .collect();
            painter.extend(shapes.transform(sequencer_transform))
        });
        let (rect, response) = ui.allocate_at_least(size, sense);
        return response;
    }
}
