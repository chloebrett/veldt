use crate::components::NoteSequencerObject;
use crate::playback::AudioPlayer;
use crate::window_state::WindowKind;
use crate::{GetSet, LocalState, transform::Transform};
use egui::PointerButton;
use egui::{
    Color32, CornerRadius, CursorIcon, Frame, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2,
    Widget, emath::RectTransform, pos2, vec2,
};
use egui::{Event, LayerId, Modifiers, Order};
use shared::model::{PitchName, PlacementType, TrackId};
use shared::types::Beats;
use state::{Action, FloatField, GeneratorSelector, NoteSelector, Store, TypeField};

pub struct NoteSequencer<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
    range: Rect,
    size: Vec2,
    objects: Vec<NoteSequencerObject>,
    quantise_level: Beats,
    background_shapes: Vec<Shape>,
    track_id: TrackId,
    audio_player: Option<&'a mut AudioPlayer>,
}

impl<'a> NoteSequencer<'a> {
    pub fn new(
        store: &'a Store,
        local_state: &'a LocalState,
        range: Rect,
        track_id: TrackId,
        audio_player: Option<&'a mut AudioPlayer>,
    ) -> Self {
        let x_size = 4000.0 * 2f32.powf(local_state.note_roll_zoom.get());
        NoteSequencer {
            store,
            local_state,
            range,
            size: vec2(x_size, 600.0),
            objects: vec![],
            quantise_level: 0.125,
            background_shapes: vec![],
            track_id,
            audio_player,
        }
    }

    #[inline]
    pub fn objects(mut self, objects: Vec<NoteSequencerObject>) -> Self {
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
        let on_release = || self.store.dispatchr(Action::Release);

        let make_movable_rect = |object: &NoteSequencerObject| object.to_rect(self.range);
        let make_resize_rect = |object: &NoteSequencerObject| {
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
            let movable_id = response.id.with(format!("movable_{index}"));
            let movable_resp = ui.interact(
                make_movable_rect(object).transform(to_sequencer),
                movable_id,
                Sense::drag(),
            );
            let resize_id = response.id.with(format!("resize_{index}"));
            let resize_resp = ui.interact(
                make_resize_rect(object).transform(to_sequencer),
                resize_id,
                Sense::drag(),
            );
            if ui.input(|input| input.modifiers.shift_only())
                && movable_resp.interact(Sense::click()).clicked()
            {
                // Add note to selected notes
                NoteSequencerObject::set_selected(self.local_state, Some(index));
            } else if movable_resp.interact(Sense::click()).double_clicked() {
                // Make active note.
                NoteSequencerObject::set_selected(self.local_state, None);
                object.set_active(self.local_state, index);
            } else if movable_resp.interact(Sense::click()).secondary_clicked() {
                // Delete note
                // self.send_note_off(object);
                NoteSequencerObject::set_selected(self.local_state, None);
                object.delete_self(self.store, self.local_state, self.track_id, index);
            }
            // Change cursor icon if cursor over shape.
            if movable_resp.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::Move);
            } else if resize_resp.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::ResizeColumn);
            }
            let edit_object = |action: Action| {
                self.store
                    .dispatch(&NoteSelector(self.track_id, index), action)
            };
            let release = self.move_object(movable_resp, to_sequencer, &edit_object)
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
        response: Response,
        to_sequencer: RectTransform,
        edit_object: &impl Fn(Action),
    ) -> bool {
        if response.dragged_by(PointerButton::Primary) {
            // Disable the note and send note off while its being dragged.
            // This deletes the initial note sound.
            edit_object(Action::SetChild(TypeField::NoteOn(false)));

            // Keep track of the delta between object and cursor position at drag start.
            let drag_pos = response.interact_pointer_pos().unwrap();
            let drag_delta = response.drag_delta();
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
                    vec2(self.range.right(), self.range.size().y - 1.0).to_pos2(),
                );
            if drag_delta.y != 0.0 {
                edit_object(Action::SetChild(TypeField::PitchName(PitchName::from(
                    (self.range.bottom() - scaled_pos.y) as i32,
                ))))
            }
            if drag_delta.x != 0.0 {
                edit_object(Action::SetFloat(
                    FloatField::Offset,
                    self.quantise(scaled_pos.x) - self.range.left(),
                ))
            }
        }
        // Enable note after interaction complete.
        if response.drag_stopped() || response.lost_focus() {
            edit_object(Action::SetChild(TypeField::NoteOn(true)));
        }
        // Return true when interaction completed.
        response.lost_focus() || response.drag_stopped()
    }

    fn resize_object(
        &self,
        index: usize,
        response: Response,
        to_sequencer: RectTransform,
        edit_object: &impl Fn(Action),
    ) -> bool {
        let object = &self.objects[index];
        if let Some(drag_pos) = response.interact_pointer_pos() {
            let scaled_pos = drag_pos.transform(to_sequencer.inverse()).clamp(
                pos2(object.to_rect(self.range).left(), 0.0),
                self.range.size().to_pos2(),
            );
            if let Some(action) = object.resize_action(self.quantise(scaled_pos.x)) {
                edit_object(action);
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

    fn send_note_off(&mut self, object: &NoteSequencerObject) {
        // Find the generator for this track
        let gen_sel = self
            .store
            .get()
            .project
            .placements
            .values()
            .filter_map(|placement| match &placement.kind {
                PlacementType::Track(it) if it.track_id == self.track_id => Some(it.generator_id),
                _ => None,
            })
            .next()
            .map(GeneratorSelector);

        // Send note off event before deleting
        if let Some(generator) = gen_sel {
            let pitch_name = object.0.note.pitch_name;
            if let Some(player) = self.audio_player.as_mut() {
                player.send_note_off(generator, pitch_name);
            }
        }
    }
}

impl Widget for NoteSequencer<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut res: Option<Response> = None;

        Frame::canvas(ui.style()).show(ui, |ui| {
            let Self {
                store,
                range,
                size,
                local_state,
                ..
            } = self;
            let (response, painter) = ui.allocate_painter(size, Sense::drag());
            let to_screen = RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, range.size()),
                response.rect,
            );
            let window_id = local_state.window_state.get_id(WindowKind::NoteRoll);
            // Only allow zoom when NoteRoll is the top layer window
            // or if pointer is on the NoteRoll.
            if Some(LayerId {
                id: window_id,
                order: Order::Middle,
            }) == ui.ctx().top_layer_id()
                || response.hover_pos().is_some()
            {
                ui.input(|input| {
                    for event in &input.events {
                        if let Event::MouseWheel {
                            modifiers: Modifiers { ctrl: true, .. },
                            delta,
                            ..
                        } = event
                        {
                            local_state
                                .note_roll_zoom
                                .update(|zoom| (zoom + delta.y * 0.05).clamp(-2.0, 5.0));
                            break;
                        }
                    }
                });
            }

            // If user inputs delete or backspace, delete any selected notes.
            if ui.input(|input| {
                input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace)
            }) {
                NoteSequencerObject::delete_selected(store, self.local_state, self.track_id);
                NoteSequencerObject::set_selected(self.local_state, None);
            } else if response.interact(Sense::click()).clicked()
                && !ui.input(|input| input.modifiers.shift)
            {
                // Remove any selected notes.
                NoteSequencerObject::set_selected(self.local_state, None);
                // Create a new note where note roll was clicked.
                let pos = response.interact_pointer_pos().unwrap();
                let object =
                    NoteSequencerObject::from_pos(pos.transform(to_screen.inverse()), range);
                object.add_new(store, self.track_id);
            }

            self.interact(ui, &response);

            painter.extend(self.background_shapes.clone().transform(to_screen));
            painter.add(self.object_shapes().transform(to_screen));

            if let Some(object) = NoteSequencerObject::get_active(self.store, self.local_state) {
                painter.add(object.active_shape(range).transform(to_screen));
            }

            let objects = NoteSequencerObject::get_selected(self.store, self.local_state);
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
