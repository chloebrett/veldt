use crate::transform::Transform;
use egui::{
    Color32, CornerRadius, CursorIcon, Frame, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2,
    Widget, emath::RectTransform, pos2, vec2,
};
use state::Action;

pub struct Sequencer<T: SequencerObject<T>, F: Fn(usize, Action), G: Fn(), H: Fn(&mut Ui, usize)> {
    range: Rect,
    size: Vec2,
    objects: Vec<T>,
    sense: Sense,
    dispatch: F, // A closure to modify object in Store. Takes object index and `Action` to dispatch chage.
    on_release: G,
    on_click: H,
    background_shapes: Vec<Shape>,
}

impl<T: SequencerObject<T>, F: Fn(usize, Action), G: Fn(), H: Fn(&mut Ui, usize)>
    Sequencer<T, F, G, H>
{
    pub fn new(range: Rect, dispatch: F, on_release: G, on_click: H) -> Self {
        Sequencer {
            range,
            size: vec2(400.0, 600.0),
            objects: vec![],
            sense: Sense::drag(),
            dispatch,
            on_release,
            on_click,
            background_shapes: vec![],
        }
    }

    #[inline]
    pub fn objects(mut self, objects: Vec<T>) -> Self {
        self.objects = objects;
        self
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
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
    pub fn horizontal_rects<J: Fn(i32) -> bool>(mut self, pattern: J, colour: Color32) -> Self {
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

    fn resize_object(
        &self,
        object: &T,
        object_index: usize,
        ui: &mut Ui,
        response: &Response,
        sequencer_transform: &RectTransform,
    ) {
        // TODO Handle this ID making better.
        // IDs must be different between resize and move fuctions.
        let id = response.id.with(format!("resize {}", object_index));
        let rect = object.to_rect(self.range);
        let x_size = 0.3;
        let resize_rect = Rect::from_min_size(
            rect.right_top() - vec2(x_size / 2.0, 0.0),
            vec2(x_size / 2.0, rect.size().y),
        );
        let rect_response =
            ui.interact(resize_rect.transform(*sequencer_transform), id, self.sense);
        if rect_response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::ResizeColumn);
        }
        let drag_pos = rect_response.interact_pointer_pos();
        if let Some(drag_pos) = drag_pos {
            let scaled_pos = drag_pos.transform(sequencer_transform.inverse()).clamp(
                pos2(rect.left(), 0.0),
                // Ensure entire rect stays on sequencer.
                self.range.size().to_pos2(),
            );
            if let Some(action) = object.resize_action(scaled_pos.x, self.range) {
                // Only dispatch on_click if object can be resized. Otherwise on_release will be
                // called with no actions in the stack.
                // TODO improve this implementation.
                if rect_response.drag_stopped() || rect_response.lost_focus() {
                    (self.on_release)();
                } else {
                    (self.dispatch)(object_index, action)
                }
            };
        }
    }

    fn interact_object(
        &self,
        object: &T,
        object_index: usize,
        ui: &mut Ui,
        response: &Response,
        sequencer_transform: &RectTransform,
    ) {
        let id = response.id.with(format!("update {}", object_index));
        let rect = object.to_rect(self.range);
        let rect_response = ui.interact(rect.transform(*sequencer_transform), id, self.sense);
        if rect_response.interact(Sense::click()).double_clicked() {
            (self.on_click)(ui, object_index)
        }
        let drag_pos = rect_response.interact_pointer_pos();
        let drag_delta = rect_response.drag_delta();
        let next_rect_pos = object.to_pos(self.range).transform(*sequencer_transform) + drag_delta;
        let mut action_dispatched = false;
        if let Some(drag_pos) = drag_pos {
            // 'y' moves in increments and should update to to wherever the mouse is while dragging.
            // 'x' moves continiously and so move based on the drag detla.
            let scaled_pos = pos2(next_rect_pos.x, drag_pos.y)
                .transform(sequencer_transform.inverse())
                .clamp(
                    pos2(0.0, 0.0),
                    // Ensure entire rect stays on sequencer.
                    (self.range.size() - vec2(rect.size().x, 1.0)).to_pos2(),
                );
            if drag_delta.y != 0.0 {
                if let Some(action) = object.y_action(scaled_pos.y, self.range) {
                    (self.dispatch)(object_index, action);
                    action_dispatched = true;
                };
            }
            if drag_delta.x != 0.0 {
                if let Some(action) = object.x_action(scaled_pos.x, self.range) {
                    (self.dispatch)(object_index, action);
                    action_dispatched = true;
                };
            }
        }
        if action_dispatched && (rect_response.drag_stopped() || rect_response.lost_focus()) {
            (self.on_release)();
        }
    }
}

impl<T: SequencerObject<T>, F: Fn(usize, Action), G: Fn(), H: Fn(&mut Ui, usize)> Widget
    for Sequencer<T, F, G, H>
{
    fn ui(self, ui: &mut Ui) -> Response {
        let Sequencer {
            range,
            size,
            ref objects,
            sense,
            dispatch: _,
            on_release: _,
            on_click: _,
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
                    self.interact_object(object, index, ui, &response, &sequencer_transform);
                    self.resize_object(object, index, ui, &response, &sequencer_transform);
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

    fn x_action(&self, x: f32, range: Rect) -> Option<Action>;

    fn y_action(&self, y: f32, range: Rect) -> Option<Action>;

    fn resize_action(&self, x: f32, range: Rect) -> Option<Action>;
}
