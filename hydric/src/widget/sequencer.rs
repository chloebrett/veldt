use crate::transform::Transform;
use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Widget,
    emath::RectTransform, pos2, vec2,
};
use shared::{
    model::{PitchName, PlacedNote},
    types::PitchValue,
};
use state::Action;

pub struct Sequencer<T: SequencerObject<T>, F: Fn(usize, Action)> {
    range: Rect,
    size: Vec2,
    objects: Vec<T>,
    sense: Sense,
    dispatch: F,
    background_shapes: Vec<Shape>,
}

impl<T: SequencerObject<T>, F: Fn(usize, Action)> Sequencer<T, F> {
    pub fn new(range: Rect, dispatch: F) -> Self {
        Sequencer {
            range,
            size: vec2(400.0, 600.0),
            objects: vec![],
            sense: Sense::drag(),
            dispatch,
            background_shapes: vec![],
        }
    }

    #[inline]
    pub fn objects(mut self, objects: Vec<T>) -> Self {
        self.objects = objects;
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
    pub fn horizontal_rects<G: Fn(i32) -> bool>(mut self, pattern: G, colour: Color32) -> Self {
        // Add horizontal rectangles across background of Sequencer.
        // Indicate where to paint rectangles with `pattern` a closure that takes `i32` the y coordinate as the
        // input and returns `true` if a rectangle should be rendered there.
        // Example
        // To alternate rectangles in background:
        //     pattern: |y| (y % 2 == 0)
        let shapes: Vec<Shape> = (0..self.range.size().y as i32)
            .filter(|&y| pattern(y))
            .map(|y| {
                let rect = Rect::from_min_size(
                    pos2(self.range.left(), y as f32),
                    vec2(self.range.size().x, 1.0),
                );
                Shape::rect_filled(rect, CornerRadius::ZERO, colour)
            })
            .collect();
        self.background_shapes.extend(shapes);
        self
    }

    fn update_objects(
        &self,
        object: &T,
        object_index: usize,
        ui: &mut Ui,
        response: &Response,
        sequencer_transform: &RectTransform,
    ) {
        let id = response.id.with(object_index);
        let rect = object.to_rect(self.range);
        let rect_response = ui.interact(rect.transform(*sequencer_transform), id, self.sense);
        let drag_pos = rect_response.interact_pointer_pos();
        let drag_delta = rect_response.drag_delta();
        if let Some(pos) = drag_pos {
            let scaled_pos = pos.transform(sequencer_transform.inverse()).clamp(
                pos2(0.0, 0.0),
                (self.range.size() - vec2(0.0, 1.0)).to_pos2(),
            );
            if drag_delta != Vec2::ZERO {
                if drag_delta.x != 0.0 {
                    (self.dispatch)(object_index, object.x_action(scaled_pos.x, self.range))
                };
                if drag_delta.y != 0.0 {
                    (self.dispatch)(object_index, object.y_action(scaled_pos.y, self.range))
                }
            }
        }
    }
}

impl<T: SequencerObject<T>, F: Fn(usize, Action)> Widget for Sequencer<T, F> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Sequencer {
            range,
            size,
            ref objects,
            sense,
            dispatch: _,
            ref background_shapes,
        } = self;
        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(size, sense);
            let sequencer_transform = RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, range.size()),
                response.rect,
            );
            let shapes: Vec<Shape> = objects
                .iter()
                .enumerate()
                .map(|(index, object)| {
                    self.update_objects(object, index, ui, &response, &sequencer_transform);
                    Shape::rect_filled(object.to_rect(range), CornerRadius::same(1), Color32::WHITE)
                })
                .collect();
            painter.extend(background_shapes.clone().transform(sequencer_transform));
            painter.extend(shapes.transform(sequencer_transform))
        });
        let (_rect, response) = ui.allocate_at_least(Vec2::ZERO, sense);
        response
    }
}

pub trait SequencerObject<T> {
    fn to_pos(&self, range: Rect) -> Pos2;

    fn to_rect(&self, range: Rect) -> Rect;

    fn x_action(&self, x: f32, range: Rect) -> Action;

    fn y_action(&self, y: f32, range: Rect) -> Action;
}

impl SequencerObject<PlacedNote> for PlacedNote {
    fn to_pos(&self, range: Rect) -> Pos2 {
        let offset: f32 = self.offset.into();
        let x = offset - range.left();
        let pitch_value: PitchValue = self.note.pitch_name.into();
        let y = range.bottom() as i32 - pitch_value;
        pos2(x, y as f32)
    }

    fn to_rect(&self, range: Rect) -> Rect {
        let pos = self.to_pos(range);
        let note_size = vec2(self.note.beats, 1.0);
        Rect::from_min_size(pos, note_size)
    }

    fn x_action(&self, x: f32, range: Rect) -> Action {
        Action::SetNoteOffset(x - range.left())
    }

    fn y_action(&self, y: f32, range: Rect) -> Action {
        Action::SetNotePitchName(PitchName::from((range.bottom() - y) as i32))
    }
}
