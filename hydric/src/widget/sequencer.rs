use crate::{GetSet, LocalState, transform::Transform};
use egui::{
    Color32, CornerRadius, CursorIcon, Frame, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2,
    Widget, emath::RectTransform, pos2, vec2,
};
use shared::types::Beats;
use state::{Action, SelectorTrait, Store};

pub struct Sequencer<'a, T: SequencerObject<T>> {
    store: &'a Store,
    local_state: &'a LocalState,
    range: Rect,
    size: Vec2,
    objects: Vec<T>,
    sense: Sense,
    quantise_level: Beats,
    // A closure to modify object in Store. Takes object index and `Action` to dispatch change.
    background_shapes: Vec<Shape>,
    parent_index: Option<usize>,
    select: bool,
}

impl<'a, T: SequencerObject<T>> Sequencer<'a, T> {
    pub fn new(store: &'a Store, local_state: &'a LocalState, range: Rect) -> Self {
        Sequencer {
            store,
            local_state,
            range,
            size: vec2(400.0, 600.0),
            objects: vec![],
            sense: Sense::drag(),
            quantise_level: 0.125,
            background_shapes: vec![],
            parent_index: None,
            select: false,
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

    pub fn parent_index(mut self, index: usize) -> Self {
        self.parent_index = Some(index);
        self
    }

    pub fn select(mut self, select: bool) -> Self {
        self.select = select;
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
    pub fn horizontal_rects<F: Fn(i32) -> bool>(mut self, pattern: F, colour: Color32) -> Self {
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

    fn interact(&self, ui: &mut Ui, response: &Response) {
        let edit_object = |index: usize, action: Action| {
            self.store
                .dispatch(&T::selector(index, self.parent_index), action)
        };
        let on_release = || self.store.dispatchr(Action::Release);

        let make_movable_rect = |object: &T| object.to_rect(self.range);
        let make_resize_rect = |object: &T| {
            let rect = object.to_rect(self.range);
            // Width of window where shape can be grabbed to resize.
            let x_size = 0.3;
            Rect::from_min_size(
                rect.right_top() - vec2(x_size * 0.5, 0.0),
                vec2(x_size * 0.5, rect.size().y),
            )
        };
        let to_sequencer = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, self.range.size()),
            response.rect,
        );
        for (index, object) in self.objects.iter().enumerate() {
            let movable_id = response.id.with(format!("movable_{}", index));
            let movable_resp = ui.interact(
                make_movable_rect(object).transform(to_sequencer),
                movable_id,
                Sense::drag(),
            );
            let resize_id = response.id.with(format!("resize_{}", index));
            let resize_resp = ui.interact(
                make_resize_rect(object).transform(to_sequencer),
                resize_id,
                Sense::drag(),
            );
            if self.select {
                if movable_resp.interact(Sense::click()).clicked() {
                    T::set_selected(self.local_state, Some(index));
                }
            } else if movable_resp.interact(Sense::click()).double_clicked() {
                object.set_active(self.local_state, index);
            } else if movable_resp.interact(Sense::click()).clicked_by(egui::PointerButton::Secondary) {
                object.set_active(self.local_state, index);
            }
            if resize_resp.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::ResizeColumn);
            }
            let release = self.move_object(index, movable_resp, to_sequencer, &edit_object)
                || self.resize_object(index, resize_resp, to_sequencer, &edit_object);
            if release {
                // TODO: fix release dispatch for move actions.
                // Compaction doesn't work properly because we have separate x and y actions.
                on_release();
                self.local_state.drag_cursor_delta.set(None);
            }
        }
    }

    fn quantise(&self, value: Beats) -> Beats {
        (value / self.quantise_level).round() * self.quantise_level
    }

    fn move_object(
        &self,
        index: usize,
        response: Response,
        to_sequencer: RectTransform,
        edit_object: &impl Fn(usize, Action),
    ) -> bool {
        let object = self.objects.get(index).expect("Should have got object.");
        let drag_pos = response.interact_pointer_pos();
        let drag_delta = response.drag_delta();
        if let Some(drag_pos) = drag_pos {
            // Keep track of the delta between object and cursor position at drag start.
            if response.interact(Sense::drag()).drag_started() {
                self.local_state
                    .drag_cursor_delta
                    .set(Some(drag_pos - response.rect.min.to_vec2()));
            }
            let click_delta: Pos2 = self
                .local_state
                .drag_cursor_delta
                .get()
                .unwrap_or(Pos2::ZERO);
            let scaled_pos = pos2(drag_pos.x - click_delta.x, drag_pos.y)
                .transform(to_sequencer.inverse())
                .clamp(
                    pos2(0.0, 0.0),
                    // Clamp to `y` range - 1 so that object cannot be dragged beyond bottom of sequencer.
                    vec2(f32::INFINITY, self.range.size().y - 1.0).to_pos2(),
                );
            if drag_delta.y != 0.0 {
                if let Some(action) = object.y_action(scaled_pos.y, self.range) {
                    edit_object(index, action);
                };
            }
            if drag_delta.x != 0.0 {
                if let Some(action) = object.x_action(self.quantise(scaled_pos.x), self.range) {
                    edit_object(index, action);
                };
            }
        }
        // Return true when interaction completed.
        response.lost_focus() || response.drag_stopped()
    }

    fn resize_object(
        &self,
        index: usize,
        response: Response,
        to_sequencer: RectTransform,
        edit_object: &impl Fn(usize, Action),
    ) -> bool {
        let object = &self.objects[index];
        let drag_pos = response.interact_pointer_pos();
        if let Some(drag_pos) = drag_pos {
            let scaled_pos = drag_pos.transform(to_sequencer.inverse()).clamp(
                pos2(object.to_rect(self.range).left(), 0.0),
                self.range.size().to_pos2(),
            );
            if let Some(action) = object.resize_action(self.quantise(scaled_pos.x), self.range) {
                edit_object(index, action);
            };
        }
        // Return true when interaction completed.
        response.lost_focus() || response.drag_stopped()
    }

    fn object_shapes(&self) -> Shape {
        Shape::Vec(
            self.objects
                .iter()
                .map(|object| object.shape(self.range))
                .collect(),
        )
    }
}

impl<T: SequencerObject<T>> Widget for Sequencer<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut res: Option<Response> = None;

        Frame::canvas(ui.style()).show(ui, |ui| {
            let Self {
                store,
                range,
                size,
                sense,
                select,
                ..
            } = self;
            let (response, painter) = ui.allocate_painter(size, sense);
            let to_screen = RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, range.size()),
                response.rect,
            );

            // If user double clicks outside of an object remove all objects from selection.
            if select {
                if response.interact(Sense::click()).double_clicked() {
                    T::set_selected(self.local_state, None)
                }
                if ui.input(|input| {
                    input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace)
                }) {
                    T::delete_selected(store, self.local_state, self.parent_index);
                    T::set_selected(self.local_state, None);
                }
            } else if response.interact(Sense::click()).clicked() {
                let pos = response.interact_pointer_pos().unwrap();
                let object = T::from_pos(pos.transform(to_screen.inverse()), range);
                object.add_new(store, self.parent_index);
            }

            self.interact(ui, &response);

            painter.extend(self.background_shapes.clone().transform(to_screen));
            painter.add(self.object_shapes().transform(to_screen));

            if let Some(object) = T::get_active(self.store, self.local_state) {
                painter.add(object.active_shape(range).transform(to_screen));
            }

            let objects = T::get_selected(ui, self.store, self.local_state);
            if !objects.is_empty() {
                painter.extend(
                    objects
                        .into_iter()
                        .map(|object| object.selected_shape(range).transform(to_screen)),
                );
            }

            res = Some(response.clone());
        });

        res.unwrap()
    }
}

pub trait SequencerObject<T> {
    fn to_pos(&self, range: Rect) -> Pos2;

    fn to_rect(&self, range: Rect) -> Rect;

    fn from_pos(pos: Pos2, rect: Rect) -> T;

    fn x_action(&self, x: f32, range: Rect) -> Option<Action>;

    fn y_action(&self, y: f32, range: Rect) -> Option<Action>;

    fn resize_action(&self, x: f32, range: Rect) -> Option<Action>;

    fn shape(&self, range: Rect) -> Shape;

    fn get_active(store: &Store, local_state: &LocalState) -> Option<T>;

    fn active_shape(&self, range: Rect) -> Shape;

    fn get_selected(ui: &Ui, store: &Store, local_state: &LocalState) -> Vec<T>;

    fn selected_shape(&self, range: Rect) -> Shape;

    fn selector(index: usize, parent_index: Option<usize>) -> impl SelectorTrait;

    fn set_active(&self, local_state: &LocalState, index: usize);

    fn set_selected(local_state: &LocalState, index: Option<usize>);

    fn add_new(&self, store: &Store, parent_index: Option<usize>);

    fn delete_selected(store: &Store, local_state: &LocalState, parent_index: Option<usize>);
}
