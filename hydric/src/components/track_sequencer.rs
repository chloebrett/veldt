use crate::LocalState;
use crate::components::PlacedTrack;
use crate::playback::AudioPlayer;
use crate::{GetSet, transform::Transform};
use egui::PointerButton;
use egui::{
    Color32, CornerRadius, CursorIcon, Frame, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2,
    Widget, emath::RectTransform,
};
use egui::{pos2, vec2};
use mesic::{beats_to_samples, samples_to_beats};
use shared::model::PlacementId;
use shared::model::{Colour, Placement, PlacementType, TrackPlacement};
use shared::types::Beats;
use state::Action;
use state::PlacementSelector;
use state::Store;
use state::TypeField;
use state::{FloatField, UintField};
use std::collections::HashMap;

pub struct TrackSequencer<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
    range: Rect,
    size: Vec2,
    objects: HashMap<PlacementId, PlacedTrack<'a>>,
    quantise_level: Beats,
    background_shapes: Vec<Shape>,
    select: bool,
    audio_player: &'a mut AudioPlayer,
}

impl<'a> TrackSequencer<'a> {
    pub fn new(
        store: &'a Store,
        local_state: &'a LocalState,
        range: Rect,
        audio_player: &'a mut AudioPlayer,
    ) -> Self {
        TrackSequencer {
            store,
            local_state,
            range,
            size: vec2(400.0, 600.0),
            objects: HashMap::new(),
            quantise_level: 0.125,
            background_shapes: vec![],
            select: false,
            audio_player,
        }
    }

    #[inline]
    pub fn objects(mut self, objects: HashMap<PlacementId, PlacedTrack<'a>>) -> Self {
        self.objects = objects;
        self
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn select(mut self, select: bool) -> Self {
        self.select = select;
        self
    }

    #[inline]
    pub fn vertical_bars(mut self, increment: f32, colour: Color32) -> Self {
        let steps = (self.range.size().x / increment).ceil() as i32;
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

    // Interactions with the placed tracks
    fn interact(&self, ui: &mut Ui, response: &Response) {
        let on_release = || self.store.dispatchr(Action::Release);

        let make_movable_rect = |object: &PlacedTrack| object.to_rect(self.range);
        let make_resize_rect = |object: &PlacedTrack| {
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
        for (id, object) in &self.objects {
            let movable_id = response.id.with(format!("movable_{:?}", id));
            let movable_resp = ui.interact(
                make_movable_rect(object).transform(to_sequencer),
                movable_id,
                Sense::drag(),
            );
            let resize_id = response.id.with(format!("resize_{:?}", id));
            let resize_resp = ui.interact(
                make_resize_rect(object).transform(to_sequencer),
                resize_id,
                Sense::drag(),
            );
            if self.select {
                if movable_resp.interact(Sense::click()).clicked() {
                    PlacedTrack::set_selected(self.local_state, Some(*id));
                }
            } else if movable_resp.interact(Sense::click()).secondary_clicked() {
                object.delete_self(self.store, self.local_state, *id);
            } else if movable_resp.interact(Sense::click()).double_clicked() {
                object.set_active(self.local_state, *id);
            }
            if resize_resp.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::ResizeColumn);
            }
            let edit_object = |action: Action| self.store.dispatch(&PlacementSelector(*id), action);
            let release = self.move_object(movable_resp, to_sequencer, &edit_object)
                || self.resize_object(*id, resize_resp, to_sequencer, &edit_object);
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
            // Keep track of the delta between object and cursor position at drag start.
            let drag_delta = response.drag_delta();
            let drag_pos = response.interact_pointer_pos().unwrap();
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
                edit_object(Action::SetUint(
                    UintField::VisualPlacement,
                    self.quantise(scaled_pos.y) as u32,
                ))
            }
            if drag_delta.x != 0.0 {
                edit_object(Action::SetFloat(
                    FloatField::Offset,
                    self.quantise(scaled_pos.x) - self.range.left(),
                ))
            }
        }
        // Return true when interaction completed.
        response.lost_focus() || response.drag_stopped()
    }

    fn resize_object(
        &self,
        id: PlacementId,
        response: Response,
        to_sequencer: RectTransform,
        edit_object: &impl Fn(Action),
    ) -> bool {
        let object = &self.objects[&id];
        let drag_pos = response.interact_pointer_pos();
        if let Some(drag_pos) = drag_pos {
            let scaled_pos = drag_pos.transform(to_sequencer.inverse()).clamp(
                pos2(object.to_rect(self.range).left(), 0.0),
                self.range.size().to_pos2(),
            );
            edit_object(object.resize_action(self.quantise(scaled_pos.x)));
        }
        // Return true when interaction completed.
        response.lost_focus() || response.drag_stopped()
    }

    fn object_shapes(&self) -> Shape {
        Shape::Vec(
            self.objects
                .values()
                .map(|object| object.shape(self.range))
                .collect(),
        )
    }

    fn object_labels(&self, ui: &mut Ui) -> Shape {
        Shape::Vec(
            self.objects
                .values()
                .map(|object| object.create_label(self.range, ui))
                .collect(),
        )
    }
}

impl Widget for TrackSequencer<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut res: Option<Response> = None;
        let store = self.store;
        let project = &store.get().project;

        Frame::canvas(ui.style()).show(ui, |ui| {
            let Self {
                store,
                range,
                size,
                select,
                ..
            } = self;
            let (response, painter) = ui.allocate_painter(size, Sense::empty());
            let to_screen = RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, range.size()),
                response.rect,
            );

            // If user double clicks outside of an object remove all objects from selection.
            // This is for clicks directly on the track roll
            if select {
                if response.interact(Sense::click()).double_clicked() {
                    PlacedTrack::set_selected(self.local_state, None);
                }
                if ui.input(|input| {
                    input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace)
                }) {
                    PlacedTrack::delete_selected(store, self.local_state);
                    PlacedTrack::set_selected(self.local_state, None);
                }
            } else if response.interact(Sense::click()).clicked() {
                let pos = response
                    .interact_pointer_pos()
                    .unwrap()
                    .transform(to_screen.inverse());

                let offset = range.left() + pos.x;

                // Add a new track of No.1 to the track roll
                let placement = Placement {
                    kind: PlacementType::Track(TrackPlacement {
                        track_id: 0.into(),
                        generator_id: 0.into(),
                    }),
                    offset: offset.into(),
                    clipped_duration: None,
                    visual_placement: pos.y as u32,
                    colour: Colour::from_8bit(67, 206, 222),
                };
                store.dispatchr(Action::AddChild(TypeField::Placement(placement)));
            }

            // Interactions with the track rectangles
            self.interact(ui, &response);

            painter.extend(self.background_shapes.clone().transform(to_screen));
            painter.add(self.object_shapes().transform(to_screen));
            painter.add(self.object_labels(ui).transform(to_screen));

            if let Some(object) = PlacedTrack::get_active(self.store, self.local_state) {
                painter.add(object.active_shape(range).transform(to_screen));
            }

            let objects = PlacedTrack::get_selected(self.store, self.local_state);
            if !objects.is_empty() {
                painter.extend(
                    objects
                        .into_iter()
                        .map(|object| object.selected_shape(range).transform(to_screen)),
                );
            }

            // Playhead visualisation and interaction
            let playhead_samples = self.audio_player.effective_pos();
            let playhead_beats = samples_to_beats(playhead_samples, project.bpm);
            let playhead_x = playhead_beats - range.left();

            let playhead_shape = Shape::line_segment(
                [pos2(playhead_x, 0.0), pos2(playhead_x, range.size().y)],
                Stroke::new(2.0, Color32::RED),
            );
            let playhead_dragger = Rect::from_min_max(
                pos2(playhead_x - 0.3, 0.0),
                pos2(playhead_x + 0.3, range.size().y),
            );
            let playhead_id = ui.id().with("playhead_dragger");
            let playhead_response = ui.interact(
                playhead_dragger.transform(to_screen),
                playhead_id,
                Sense::drag(),
            );

            painter.add(playhead_shape.transform(to_screen));

            if playhead_response.dragged() {
                if let Some(drag_pos) = playhead_response.interact_pointer_pos() {
                    let local_pos = drag_pos.transform(to_screen.inverse());
                    let new_playhead_x = local_pos.x.clamp(0.0, range.size().x);
                    let new_playhead_beats = range.left() + new_playhead_x;
                    let new_samples = beats_to_samples(new_playhead_beats, project.bpm);
                    self.audio_player.seek(new_samples as usize);
                }
            }
            res = Some(response.clone());
        });

        res.unwrap()
    }
}
