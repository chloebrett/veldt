use crate::{GetSet, LocalState, transform::Transform};
use crate::{view::View, widget::StateWindow, window_state::WindowKind};
use egui::{Galley, PointerButton};
use egui::epaint::{RectShape, TextShape};
use egui::{
    Color32, CornerRadius, CursorIcon, Frame, Pos2, Rect, Response, ScrollArea, Sense, Shape,
    Stroke, StrokeKind, Ui, Vec2, Widget, emath::RectTransform, pos2, vec2,
};
use mesic::samples_to_beats;
use ordered_float::OrderedFloat;
use shared::{
    model::{
        PlacedNote, Placement, PlacementId, PlacementType, SamplePlacement, Track, TrackPlacement, SampleId
    },
    types::Beats,
};
use state::{
    Action, FloatField, MultiTypeField, PlacementSelector, SampleSelector, Store, TrackSelector,
    TypeField, UintField,
};
use std::cmp::max;
use std::collections::{HashMap, HashSet};

pub struct TrackRoll<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
}

impl<'a> TrackRoll<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState) -> Self {
        Self { store, local_state }
    }
}

const PITCH_RANGE: f32 = 4131.0;

impl View for TrackRoll<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let store = self.store;
        let project = &store.get().project;

        // TODO: rename PlacedTrack to something encompassing both tracks and samples.
        // They can be a single object, but just contain a tag/enum specifying which one they are.
        let placed_tracks: HashMap<PlacementId, PlacedTrack> = project
            .placements
            .clone()
            .into_iter()
            .map(|(id, placement)| {
                (
                    id,
                    match placement.kind {
                        PlacementType::Track(TrackPlacement { track_id, .. }) => PlacedTrack {
                            unclipped_duration: project.tracks[&track_id].unclipped_duration(),
                            placement: placement.clone(),
                            store: self.store,
                        },
                        PlacementType::Sample(SamplePlacement { sample_id }) => {
                            let duration = store
                                .try_select(&SampleSelector(sample_id))
                                .map(|sample| {
                                    samples_to_beats(
                                        max(sample.left.len(), sample.right.len()),
                                        store.get().project.bpm,
                                    )
                                })
                                .unwrap_or(1.0);
                            PlacedTrack {
                                unclipped_duration: duration.into(),
                                placement: placement.clone(),
                                store: self.store,
                            }
                        }
                    },
                )
            })
            .collect();

        let min_rows = 4;
        let max_visual_placement = max(
            project
                .placements
                .values()
                .map(|it| it.visual_placement)
                .max()
                .unwrap_or(0),
            min_rows,
        );

        let mut select = self.local_state.track_roll_select_enabled.get();
        if !select {
            self.local_state.selected_placements.set(HashSet::default());
        }
        StateWindow::show_from_window_state_resizable(
            ui,
            &self.local_state.window_state,
            WindowKind::TrackRoll,
            "Track Roll",
            |ui| {
                ui.horizontal(|ui| {
                    if ui.button("New track").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::Track(Track::default())));
                    }
                    if ui.button("New track placement").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::Placement(
                            Placement::default(),
                        )));
                    }
                    if ui.button("New sample placement").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::Placement(Placement {
                            kind: PlacementType::Sample(SamplePlacement::default()),
                            offset: 0.0.into(),
                            clipped_duration: None,
                            visual_placement: 0,
                            colour: [67, 206, 222],
                        })));
                    }
                    ui.checkbox(&mut select, "Select")
                });
                ui.separator();
                let window_size = ui.available_size();
                const PADDING_AROUND_TRACK_SQUENCER: f32 = 6.0;
                const MINIMUM_SIZE: f32 = 600.0;
                const INCREMENT_SIZE: f32 = 37.5;
                let range = Rect::from_min_max(
                    Pos2::ZERO,
                    pos2(
                        ((window_size.x - PADDING_AROUND_TRACK_SQUENCER).max(MINIMUM_SIZE)
                            / INCREMENT_SIZE)
                            .ceil(),
                        max_visual_placement as f32,
                    ),
                );
                ScrollArea::vertical()
                    .min_scrolled_height(400.0)
                    .show(ui, |ui| {
                        ui.add(
                            TrackSequencer::new(store, self.local_state, range)
                                .objects(placed_tracks)
                                .size(vec2(
                                    (window_size.x - 6.0).max(600.0),
                                    100.0 * max_visual_placement as f32,
                                ))
                                .select(select)
                                .vertical_bars(4.0, Color32::from_white_alpha(6))
                                .vertical_bars(1.0, Color32::from_white_alpha(3))
                                .horizontal_rects(
                                    |index| index % 2 == 1,
                                    Color32::from_white_alpha(1),
                            )

                        );
                    });
            },
        );
        self.local_state.track_roll_select_enabled.set(select);
    }
}

struct PlacedTrack<'a> {
    placement: Placement,
    unclipped_duration: OrderedFloat<f32>,
    store: &'a Store,
}

impl<'a> PlacedTrack<'a> {
    fn to_pos(&self, range: Rect) -> Pos2 {
        let y = self.placement.visual_placement as f32;
        let x = *self.placement.offset - range.left();
        pos2(x, y)
    }

    fn to_rect(&self, range: Rect) -> Rect {
        let track_pos = self.to_pos(range);
        // If not clipped duration render length based on notes.
        let length: f32 = *self
            .placement
            .clipped_duration
            .unwrap_or(self.unclipped_duration);
        // Min `track_size.x` of 0.4 to ensure part of the object is still visible to interact with.
        let track_size = vec2(length.max(0.4), 1.0);
        Rect::from_min_size(track_pos, track_size)
    }

    fn resize_action(&self, x: f32) -> Action {
        let clipped_duration = x - *self.placement.offset;
        let max_note_length = *self.unclipped_duration;
        let clipped_duration = if clipped_duration < max_note_length {
            Some(clipped_duration as Beats)
        } else {
            None
        };
        Action::SetChild(TypeField::ClippedDuration(clipped_duration))
    }

    fn shape(&self, range: Rect) -> Shape {
        let rgb_values = self.placement.colour;
        let background_colour = Color32::from_rgba_unmultiplied(
            rgb_values[0] as u8,
            rgb_values[1] as u8,
            rgb_values[2] as u8,
            20,
        );
        Shape::Vec(vec![
            // track background shape
            if *self.unclipped_duration == 0.0 {
                Shape::rect_filled(
                    self.to_rect(range),
                    CornerRadius::same(1),
                    background_colour,
                )
            } else {
                Shape::rect_filled(
                    self.to_rect(range),
                    CornerRadius::same(1),
                    background_colour,
                )
            },
            match &self.placement.kind {
                PlacementType::Track(track_placement) => {
                    let track_id = track_placement.track_id;
                    if let Some(track) = self.store.get().project.tracks.get(&track_id) {
                        let notes = &track.notes;
                        self.map_notes_to_shapes(range, notes)
                    } else {
                        Shape::rect_filled(
                            self.to_rect(range),
                            CornerRadius::same(1),
                            background_colour,
                        )
                    }
                }
                PlacementType::Sample(sample_placement) => {
                    let sample_id = sample_placement.sample_id;
                    self.sample_shape(range, sample_id)
                }
,
            },
            self.create_header_shape(range), // header label is created separately via create_label as textshape galley requires access to Ui
        ])
    }

    fn sample_shape(&self, range: Rect, sample_id: SampleId) -> Shape{
        if let Some(sample) = self.store.get().project.samples.get(&sample_id) {
            
        }
        Shape::Noop
    }

    fn map_notes_to_shapes(&self, range: Rect, notes: &Vec<PlacedNote>) -> Shape {
        let rgb_values = self.placement.colour;
        let note_positions: Vec<Pos2> = notes
            .into_iter()
            .map(|note| {
                let x_pos = f32::from(note.offset);
                let y_pos: f32 = note.note.pitch_name.into();
                Pos2::new(x_pos, PITCH_RANGE - y_pos)
            })
            .collect();

        let note_shapes: Vec<Shape> = notes
            .into_iter()
            .enumerate()
            .map(|(i, note)| {
                let mut note_rect = Rect::from_pos(note_positions[i]);
                note_rect.set_width(note.note.beats);
                note_rect.set_height(250.0);
                let note_shape: Shape = RectShape::new(
                    note_rect,
                    1.5,
                    Color32::from_rgb_additive(
                        rgb_values[0],
                        rgb_values[1],
                        rgb_values[2],
                    ),
                    Stroke::NONE,
                    StrokeKind::Inside,
                )
                .into();
                note_shape
            })
            .collect();

        let track_transform = RectTransform::from_to(
            Rect::from_min_max(Pos2::ZERO, Pos2::new(*self.unclipped_duration, PITCH_RANGE)),
            self.to_rect(range).shrink2(Vec2::new(0.0, 0.07)),
        );
        let transformed_shapes = Shape::Vec(note_shapes.transform(track_transform));

        transformed_shapes
    }

    fn create_header_shape(&self, range: Rect) -> Shape {
        let rgb_values = self.placement.colour;
        let header_brackground_colour = Color32::from_rgb_additive(
            rgb_values[0],
            rgb_values[1],
            rgb_values[2],
        );

        let mut header_rect = Rect::from_pos(Pos2::ZERO);
        header_rect.set_width(*self.unclipped_duration);
        header_rect.set_height(500.0);

        let placement_transform = RectTransform::from_to(
            Rect::from_min_max(Pos2::ZERO, Pos2::new(*self.unclipped_duration, PITCH_RANGE)),
            self.to_rect(range),
        );

        let header_shape = Shape::rect_filled(header_rect, 0.2, header_brackground_colour)
            .transform(placement_transform);

        header_shape
    }

    fn create_label(&self, range: Rect, ui: &mut Ui) -> Shape {
        let font_id = egui::FontId::proportional(12.0);
        let galley = ui.fonts(|fonts| fonts.layout_no_wrap(("Please I beg").to_string(), font_id.clone(), Color32::BLACK));
        let transform = RectTransform::from_to(
            Rect::from_min_max(Pos2::ZERO, Pos2::new(*self.unclipped_duration, PITCH_RANGE)),
            self.to_rect(range),
        );
        let position = Pos2::ZERO.transform(transform);
        let text_shape = Shape::Text(TextShape {
            pos: position,
            galley,
            underline: Stroke::NONE,
            override_text_color: Some(Color32::BLACK),
            angle: 0.0,
            fallback_color: Color32::BLACK,
            opacity_factor: 1.0,
        });
        text_shape
    }

    fn get_active(store: &'a Store, local_state: &LocalState) -> Option<PlacedTrack<'a>> {
        let id = local_state.active_placement.get()?;
        let selector = PlacementSelector(id);
        let placement = store.select(&selector);
        let track_placement: Option<&TrackPlacement> = placement.try_into().ok();
        Some(PlacedTrack {
            // TODO: sample duration
            unclipped_duration: track_placement
                .map(|it| {
                    store
                        .select(&TrackSelector(it.track_id))
                        .unclipped_duration()
                })
                .unwrap_or(1.0.into()),
            placement: placement.clone(),
            store: store,
        })
    }

    fn active_shape(&self, range: Rect) -> Shape {
        let rgb_values = self.placement.colour;
        Shape::Vec(vec![
            self.shape(range),
            Shape::rect_stroke(
                self.to_rect(range),
                CornerRadius::same(0),
                Stroke {
                    width: 1.5,
                    color: Color32::from_rgb_additive(
                        rgb_values[0],
                        rgb_values[1],
                        rgb_values[2],
                    ),
                },
                StrokeKind::Inside,
            ),
        ])
    }

    fn get_selected(store: &'a Store, local_state: &LocalState) -> Vec<PlacedTrack<'a>> {
        local_state
            .selected_placements
            .get()
            .into_iter()
            .map(|index| {
                let placement_sel = PlacementSelector(index);
                let placement = store.select(&placement_sel);
                let track_placement: &TrackPlacement = placement.try_into().unwrap();
                PlacedTrack {
                    unclipped_duration: store
                        .select(&TrackSelector(track_placement.track_id))
                        .unclipped_duration(),
                    placement: placement.clone(),
                    store: store,
                }
            })
            .collect()
    }

    fn selected_shape(&self, range: Rect) -> Shape {
        let rgb_values = self.placement.colour;
        Shape::Vec(vec![
            self.shape(range),
            Shape::rect_stroke(
                self.to_rect(range),
                CornerRadius::same(0),
                Stroke {
                    width: 1.5,
                    color: Color32::from_rgb_additive(
                        rgb_values[0] as u8,
                        rgb_values[1] as u8,
                        rgb_values[2] as u8,
                    ),
                },
                StrokeKind::Inside,
            ),
        ])
    }

    fn set_active(&self, local_state: &LocalState, id: PlacementId) {
        let track_placement: Option<&TrackPlacement> = (&self.placement).try_into().ok();

        local_state
            .window_state
            .set_visible(WindowKind::NoteRoll, true);
        local_state
            .window_state
            .set_visible(WindowKind::Placement, true);
        local_state
            .active_track
            .set(track_placement.map(|it| TrackSelector(it.track_id)));
        local_state.active_placement.set(Some(id));
    }

    fn set_selected(local_state: &LocalState, id: Option<PlacementId>) {
        // TODO: weird API. Deleting all selections if this is none? Shouldn't we just pass a set
        // of IDs?
        let Some(id) = id else {
            local_state.selected_placements.set(HashSet::default());
            return;
        };

        local_state.selected_placements.update(|mut it| {
            if it.contains(&id) {
                it.remove(&id);
            } else {
                it.insert(id);
            }
            it
        });
    }

    fn delete_selected(store: &Store, local_state: &LocalState) {
        local_state
            .active_placement
            .update(|placement| match placement {
                // If the active placement is selected, "de-activate" it.
                Some(id) if local_state.selected_placements.get().contains(&id) => None,
                _ => placement,
            });
        store.dispatchr(Action::DeleteChildrenById(MultiTypeField::PlacementId(
            local_state.selected_placements.get().into_iter().collect(),
        )));
    }

    fn delete_self(&self, store: &Store, local_state: &LocalState, placement_id: PlacementId) {
        local_state.active_placement.update(|placement| {
            if placement == Some(placement_id) {
                None
            } else {
                placement
            }
        });
        store.dispatchr(Action::DeleteChildById(TypeField::PlacementId(
            placement_id,
        )));
    }
}

struct TrackSequencer<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
    range: Rect,
    size: Vec2,
    objects: HashMap<PlacementId, PlacedTrack<'a>>,
    quantise_level: Beats,
    background_shapes: Vec<Shape>,
    select: bool,
}

impl<'a> TrackSequencer<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState, range: Rect) -> Self {
        TrackSequencer {
            store,
            local_state,
            range,
            size: vec2(400.0, 600.0),
            objects: HashMap::new(),
            quantise_level: 0.125,
            background_shapes: vec![],
            select: false,
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

        Frame::canvas(ui.style()).show(ui, |ui| {
            let Self {
                store,
                range,
                size,
                select,
                ..
            } = self;
            let (response, painter) = ui.allocate_painter(size, Sense::drag());
            let to_screen = RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, range.size()),
                response.rect,
            );

            // If user double clicks outside of an object remove all objects from selection.
            if select {
                if response.interact(Sense::click()).double_clicked() {
                    PlacedTrack::set_selected(self.local_state, None)
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
                let placement = Placement {
                    kind: PlacementType::Track(TrackPlacement {
                        track_id: 0.into(),
                        generator_id: 0.into(),
                    }),
                    offset: offset.into(),
                    clipped_duration: None,
                    visual_placement: pos.y as u32,
                    colour: [67, 206, 222],
                };
                store.dispatchr(Action::AddChild(TypeField::Placement(placement)));
            }

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

            res = Some(response.clone());
        });

        res.unwrap()
    }
}
