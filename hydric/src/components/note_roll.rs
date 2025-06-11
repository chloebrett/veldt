use super::{Piano, PianoOrientation};
use crate::{GetSet, LocalState, transform::Transform};
use crate::{
    playback::AudioPlayer, transform::Yx, view::View, widget::StateWindow, window_state::WindowKind,
};
use egui::PointerButton;
use egui::{
    Color32, CornerRadius, CursorIcon, Frame, Pos2, Rect, Response, ScrollArea, Sense, Shape,
    Stroke, StrokeKind, Ui, Vec2, Widget, emath::RectTransform, pos2, vec2,
};
use mesic::create_scale_values;
use shared::types::Beats;
use shared::{
    model::{Note, PitchName, PlacedNote, PlacementType, Scale, ScaleValue, TrackId},
    types::PitchValue,
};
use state::{
    Action, FloatField, GeneratorSelector, IndexField, MultiIndexField, NoteSelector, Store,
    TrackSelector, TypeField,
};
use std::collections::HashSet;

pub struct NoteRoll<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
    audio_player: &'a mut AudioPlayer,
    min_note: PitchValue,
    max_note: PitchValue,
    offset: f32,
    bar_length: f32,
}

impl<'a> NoteRoll<'a> {
    pub fn new(
        store: &'a Store,
        local_state: &'a LocalState,
        audio_player: &'a mut AudioPlayer,
    ) -> Self {
        Self {
            store,
            local_state,
            audio_player,
            min_note: PitchName {
                scale_value: ScaleValue::A,
                octave: 1,
            }
            .into(),
            max_note: PitchName {
                scale_value: ScaleValue::C,
                octave: 8,
            }
            .into(),
            offset: 0.0,
            bar_length: 4.0,
        }
    }

    fn make_white_note_pattern(&self, max_note: i32) -> impl Fn(i32) -> bool + use<> {
        // Return a pattern for Background Rects to display white notes.
        // Account for max note changing.
        let c_value: PitchValue = ScaleValue::C.into();
        let max_scale_value: PitchValue = PitchName::from(max_note).scale_value.into();
        let c_delta = c_value - max_scale_value;
        move |y| {
            let notes = create_scale_values(Scale::Major, ScaleValue::C);
            // Return true for notes in C Major (White notes)
            // Determine if `y` is a white note by checking if the `ScaleValue` of the
            // note is in the C Major scale where `0 => C`, `1 => CSharp` etc.
            // Calculate `y` modulo `12` to account for higher values of y (`ScaleValues` are
            // between 0 and 11).
            // `y` will start at 0 no matter what the `max_note` is. Account for this by
            // adding the difference between the C `ScaleValue` and the `max_note` scale so that
            // `y` will start at the correct `ScaleValue`.
            // Use the negative of `y + c_delta` as `y` starts from the top of the piano
            // and moves down and so moves backwards through the scale.
            let scale_value = ScaleValue::from(((0 - (y + c_delta)) as i32).rem_euclid(12) as i32);
            notes.contains(&scale_value)
        }
    }
}

impl View for NoteRoll<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            store,
            min_note,
            max_note,
            offset,
            bar_length,
            local_state,
            ..
        } = *self;
        let Some(track_sel) = self.local_state.active_track.get() else {
            return;
        };

        let default_note = PlacedNote {
            note: Note {
                pitch_name: PitchName {
                    scale_value: store.get().key,
                    octave: 4,
                },
                beats: 1.0,
            },
            offset: offset.into(),
        };
        let notes = store.select(&track_sel).notes.clone();
        let white_note_pattern = self.make_white_note_pattern(max_note);
        let unclipped_duration = store.select(&track_sel).unclipped_duration();
        let range = Rect::from_min_max(
            pos2(offset, min_note as f32 - 1.0),
            // NoteRoll is at least 1 bar long
            // Extends when notes are dragged or set beyond 1 bar.
            // Add 0.5 to X as a small buffer after max note.
            pos2(
                f32::max(bar_length, *unclipped_duration) + 0.5,
                max_note as f32,
            ),
        );
        let mut select = local_state.note_roll_select_enabled.get();
        if !select {
            local_state.selected_notes.set(HashSet::default());
        }
        let title = format!("Track {}", *track_sel.0);
        StateWindow::show_from_window_state(
            ui,
            &local_state.window_state,
            WindowKind::NoteRoll,
            &title,
            |ui| {
                ui.horizontal(|ui| {
                    if ui.button("New note").clicked() {
                        store.dispatch(
                            &track_sel,
                            Action::AddChild(TypeField::PlacedNote(default_note)),
                        );
                    }
                    ui.checkbox(&mut select, "Select")
                });
                ui.separator();
                ScrollArea::vertical()
                    .min_scrolled_height(200.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // TODO: use the correct generator for the track placement that
                            // actually opened this UI - not just the first track placement we can
                            // find that matches this track.
                            let gen_sel = store
                                .get()
                                .project
                                .placements
                                .values()
                                .filter_map(|placement| match &placement.kind {
                                    PlacementType::Track(it) if it.track_id == track_sel.0 => {
                                        Some(it.generator_id)
                                    }
                                    _ => None,
                                })
                                .next()
                                .map(GeneratorSelector);

                            Piano::new(
                                max_note,
                                min_note - 1,
                                PianoOrientation::Vertical,
                                Vec2::new(600.0, 50.0),
                                Some(self.audio_player),
                                gen_sel,
                            )
                            .ui(ui);
                            ui.add(
                                NoteSequencer::new(store, local_state, range, track_sel.0)
                                    .objects(notes.into_iter().map(NoteSequencerObject).collect())
                                    .select(select)
                                    .horizontal_rects(
                                        white_note_pattern,
                                        Color32::from_white_alpha(4),
                                    )
                                    .vertical_bars(bar_length, Color32::from_white_alpha(6))
                                    .vertical_bars(1.0, Color32::from_white_alpha(3))
                                    .vertical_bars(1.0 / bar_length, Color32::from_white_alpha(1)),
                            );
                        });
                    });
            },
        );
        local_state.note_roll_select_enabled.set(select);
    }
}

pub struct NoteSequencerObject(pub PlacedNote);

impl NoteSequencerObject {
    pub fn to_pos(&self, range: Rect) -> Pos2 {
        let offset: f32 = self.0.offset.into();
        let y = offset - range.top();
        let pitch_value: PitchValue = self.0.note.pitch_name.into();
        let x = range.right() as i32 - pitch_value;
        pos2(x as f32, y)
    }

    fn to_rect(&self, range: Rect) -> Rect {
        let pos = self.to_pos(range.yx()).yx();
        let note_size = vec2(self.0.note.beats, 1.0);
        Rect::from_min_size(pos, note_size)
    }

    fn resize_action(&self, x: f32) -> Option<Action> {
        let beats = x - *self.0.offset;
        Some(Action::SetFloat(FloatField::Duration, beats))
    }

    fn shape(&self, range: Rect) -> egui::Shape {
        Shape::rect_filled(self.to_rect(range), CornerRadius::same(1), Color32::WHITE)
    }

    fn get_active(store: &Store, local_state: &LocalState) -> Option<Self> {
        let track_sel = local_state.active_track.get()?;
        local_state.active_note.get().map(|note_index| {
            let sel: NoteSelector = track_sel.downcast_note(note_index);
            let note: &PlacedNote = store.select(&sel);
            NoteSequencerObject(note.clone())
        })
    }

    fn active_shape(&self, range: Rect) -> Shape {
        Shape::Vec(vec![
            self.shape(range),
            Shape::rect_stroke(
                self.to_rect(range),
                CornerRadius::same(0),
                Stroke {
                    width: 1.0,
                    color: Color32::BLUE,
                },
                StrokeKind::Inside,
            ),
        ])
    }

    fn get_selected(store: &Store, local_state: &LocalState) -> Vec<Self> {
        let Some(track_sel) = local_state.active_track.get() else {
            return vec![];
        };
        local_state
            .selected_notes
            .get()
            .into_iter()
            .map(|note_index| {
                let sel: NoteSelector = track_sel.downcast_note(note_index);
                let note: &PlacedNote = store.select(&sel);
                NoteSequencerObject(note.clone())
            })
            .collect()
    }

    fn selected_shape(&self, range: Rect) -> Shape {
        Shape::Vec(vec![
            self.shape(range),
            Shape::rect_stroke(
                self.to_rect(range),
                CornerRadius::same(0),
                Stroke {
                    width: 1.0,
                    color: Color32::RED,
                },
                StrokeKind::Inside,
            ),
        ])
    }

    fn set_active(&self, local_state: &LocalState, index: usize) {
        local_state.window_state.set_visible(WindowKind::Note, true);
        local_state.active_note.set(Some(index));
    }

    fn set_selected(local_state: &LocalState, index: Option<usize>) {
        let Some(index) = index else {
            local_state.selected_notes.set(HashSet::default());
            return;
        };

        let mut notes = local_state.selected_notes.get();

        if notes.contains(&index) {
            notes.remove(&index);
        } else {
            notes.insert(index);
        }

        local_state.selected_notes.set(notes);
    }

    fn add_new(&self, store: &Store, track_id: TrackId) {
        store.dispatch(
            &TrackSelector(track_id),
            Action::AddChild(TypeField::PlacedNote(self.0.clone())),
        );
    }

    fn from_pos(pos: Pos2, range: Rect) -> Self {
        let offset = pos.x + range.left();
        let pitch_value: PitchValue = (range.bottom() - pos.y) as i32;
        NoteSequencerObject(PlacedNote {
            note: Note {
                pitch_name: pitch_value.into(),
                beats: 1.0,
            },
            offset: offset.into(),
        })
    }

    fn delete_selected(store: &Store, local_state: &LocalState, track_id: TrackId) {
        // De-activate active note.
        // TODO: Only set active note to none if that note is deleted. 
        // This is currently necessary as the local state stores the active by index.
        // If the active note is greater than the deleted note it either changes the active note or
        // is an index out of range and panics.
        local_state.active_note.update(|_| None);
        store.dispatch(
            &TrackSelector(track_id),
            Action::DeleteChildren(MultiIndexField::PlacedNote(
                local_state.selected_notes.get().into_iter().collect(),
            )),
        );
    }

    fn delete_self(
        &self,
        store: &Store,
        local_state: &LocalState,
        track_id: TrackId,
        note_index: usize,
    ) {
        // De-activate active note.
        // TODO: Fix this. The local state stores the active by index.
        // If the active note is greater than the deleted note it either changes the active note or
        // is an index out of range and panics.
        local_state.active_note.update(|_| None);
        store.dispatch(
            &TrackSelector(track_id),
            Action::DeleteChild(IndexField::PlacedNote(note_index)),
        )
    }
}

pub struct NoteSequencer<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
    range: Rect,
    size: Vec2,
    objects: Vec<NoteSequencerObject>,
    quantise_level: Beats,
    background_shapes: Vec<Shape>,
    track_id: TrackId,
    select: bool,
}

impl<'a> NoteSequencer<'a> {
    pub fn new(
        store: &'a Store,
        local_state: &'a LocalState,
        range: Rect,
        track_id: TrackId,
    ) -> Self {
        NoteSequencer {
            store,
            local_state,
            range,
            size: vec2(400.0, 600.0),
            objects: vec![],
            quantise_level: 0.125,
            background_shapes: vec![],
            track_id,
            select: false,
        }
    }

    #[inline]
    pub fn objects(mut self, objects: Vec<NoteSequencerObject>) -> Self {
        self.objects = objects;
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
            if self.select {
                if movable_resp.interact(Sense::click()).clicked() {
                    NoteSequencerObject::set_selected(self.local_state, Some(index));
                }
            } else if movable_resp.interact(Sense::click()).double_clicked() {
                object.set_active(self.local_state, index);
            } else if movable_resp.interact(Sense::click()).secondary_clicked() {
                object.delete_self(self.store, self.local_state, self.track_id, index);
            };
            if resize_resp.hovered() {
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
                    // Clamp to `y` range - 1 so that object cannot be dragged beyond bottom of
                    // sequencer.
                    vec2(f32::INFINITY, self.range.size().y - 1.0).to_pos2(),
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
}

impl Widget for NoteSequencer<'_> {
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
                    NoteSequencerObject::set_selected(self.local_state, None)
                }
                if ui.input(|input| {
                    input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace)
                }) {
                    NoteSequencerObject::delete_selected(store, self.local_state, self.track_id);
                    NoteSequencerObject::set_selected(self.local_state, None);
                }
            } else if response.interact(Sense::click()).clicked() {
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
