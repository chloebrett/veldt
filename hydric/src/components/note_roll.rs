use crate::state::{Action, Selector, Store};
use ordered_float::OrderedFloat;
use shared::types::Beats;

use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Response, ScrollArea, Sense, Shape, Stroke,
    StrokeKind, Ui, Vec2, emath::RectTransform, epaint::RectShape, pos2, vec2,
};
use shared::{
    model::{Note, PitchName, PlacedNote, ScaleValue},
    types::PitchValue,
};
use web_sys::console;

enum RollObject {
    InteractiveNote { note_rect: Rect },
    BackgroundNote { note_rect: Rect },
    BarLine { line: [Pos2; 2], order: u32 },
}

struct InteractiveRoll {
    notes: Vec<PlacedNote>,
    max_note: PitchName,
    min_note: PitchName,
    offset: f32,
    n_bars: f32,
    metre: f32,
}

impl InteractiveRoll {
    pub fn make_all_objects(&self) -> Vec<RollObject> {
        let mut roll_objects = vec![];
        let background_notes = self.get_background_notes();
        roll_objects.extend(self.make_all_background_notes(background_notes));
        roll_objects.extend(self.make_all_interactive_notes());
        roll_objects.extend(self.make_all_bar_lines());
        roll_objects
    }

    pub fn get_background_notes(&self) -> Vec<PitchName> {
        let min_pitch_value: PitchValue = self.min_note.into();
        let max_pitch_value: PitchValue = self.max_note.into();
        (min_pitch_value..=max_pitch_value)
            .map(|pitch_value| pitch_value.into())
            .collect()
    }

    pub fn make_all_background_notes(&self, notes: Vec<PitchName>) -> Vec<RollObject> {
        notes
            .iter()
            .map(|&note| self.make_background_notes(note))
            .collect()
    }

    pub fn make_background_notes(&self, note: PitchName) -> RollObject {
        let pitch_value: PitchValue = note.into();
        let min_pitch_value: PitchValue = self.min_note.into();
        let note_pos = pos2(0.0, (pitch_value - min_pitch_value) as f32);
        let note_size = vec2(self.n_bars * self.metre, 1.0);
        RollObject::BackgroundNote {
            note_rect: Rect::from_min_size(note_pos, note_size),
        }
    }

    pub fn make_all_interactive_notes(&self) -> Vec<RollObject> {
        self.notes
            .iter()
            .map(|note| self.make_interactive_note(note.clone()))
            .collect()
    }

    pub fn make_interactive_note(&self, note: PlacedNote) -> RollObject {
        let pitch_value: PitchValue = note.note.pitch_name.into();
        let min_note_value: PitchValue = self.min_note.into();
        let offset: f32 = note.offset.into();
        let note_pos = pos2(offset - self.offset, (pitch_value - min_note_value) as f32);
        let note_size = vec2(note.note.beats, 1.0);
        RollObject::InteractiveNote {
            note_rect: Rect::from_min_size(note_pos, note_size),
        }
    }

    pub fn make_all_bar_lines(&self) -> Vec<RollObject> {
        let n_orders: u32 = 3;
        let mut bar_lines = vec![];
        for order in 0..n_orders {
            let mut order_barlines = vec![];
            let n_bar_lines = (self.n_bars * self.metre.powf(order as f32)) as i32;
            for value in 0..=n_bar_lines {
                let beat = value as f32 * self.metre.powf(1.0 - order as f32);
                order_barlines.push(self.make_bar_line(beat, order))
            }
            bar_lines.extend(order_barlines)
        }
        bar_lines
    }

    pub fn make_bar_line(&self, beat: f32, order: u32) -> RollObject {
        let min_pitch_value: PitchValue = self.min_note.into();
        let max_pitch_value: PitchValue = self.max_note.into();
        let line = [
            pos2(beat, min_pitch_value as f32),
            pos2(beat, max_pitch_value as f32),
        ];
        RollObject::BarLine { line, order }
    }
}

struct Piano {
    max_note: PitchName,
    min_note: PitchName,
}

impl Piano {
    pub fn make_all_objects(&self) -> Vec<PianoKey> {
        let notes = self.get_piano_notes();
        self.make_all_piano_keys(notes)
    }

    fn get_piano_notes(&self) -> Vec<PitchName> {
        let min_pitch_value: PitchValue = self.min_note.into();
        let max_pitch_value: PitchValue = self.max_note.into();
        (min_pitch_value..=max_pitch_value)
            .map(|pitch_value| pitch_value.into())
            .collect()
    }

    fn make_all_piano_keys(&self, notes: Vec<PitchName>) -> Vec<PianoKey> {
        notes
            .iter()
            .map(|&note| self.make_piano_key(note))
            .collect()
    }

    fn make_piano_key(&self, note: PitchName) -> PianoKey {
        // White notes are arranged so that the edge of B and C and the edge of
        // E and F lines align with the edge of the equivalent background.
        // This helps visual align background notes with the piano keys.
        // As a result white notes C, D, and E are slightly larger, spread out
        // over 5 background notes and F, G, A, and B slight smaller spread out
        // over 7.
        // y_offset indicates where the white note shape should start in
        // relation to the background note and note_height corresponding height
        // of that note.
        let (y_offset, note_height) = match note.scale_value {
            ScaleValue::C => (-2.0 / 3.0, 5.0 / 3.0),
            ScaleValue::D => (-1.0 / 3.0, 5.0 / 3.0),
            ScaleValue::E => (0.0, 5.0 / 3.0),
            ScaleValue::F => (-3.0 / 4.0, 7.0 / 4.0),
            ScaleValue::G => (-1.0 / 2.0, 7.0 / 4.0),
            ScaleValue::A => (-1.0 / 4.0, 7.0 / 4.0),
            ScaleValue::B => (0.0, 7.0 / 4.0),
            // return black key note as regular size and offset
            _ => return self.make_black_key(note),
        };
        self.make_white_key(note, y_offset, note_height)
    }

    fn make_white_key(&self, note: PitchName, y_offset: f32, note_height: f32) -> PianoKey {
        let pitch_value: PitchValue = note.into();
        let min_pitch_value: PitchValue = self.min_note.into();
        let note_pos = pos2(0.0, (pitch_value - min_pitch_value) as f32 - y_offset);
        let note_size = vec2(1.0, note_height);
        PianoKey::WHITE {
            note_rect: Rect::from_min_size(note_pos, note_size),
        }
    }

    fn make_black_key(&self, note: PitchName) -> PianoKey {
        let black_note_length = 0.6;
        let pitch_value: PitchValue = note.into();
        let min_pitch_value: PitchValue = self.min_note.into();
        let note_pos = pos2(0.0, (pitch_value - min_pitch_value) as f32);
        let note_size = vec2(black_note_length, 1.0);
        PianoKey::BLACK {
            note_rect: Rect::from_min_size(note_pos, note_size),
        }
    }
}

pub fn note_roll_display(store: &Store, ui: &mut Ui) {
    new_note_button(store, ui);
    ScrollArea::vertical()
        .min_scrolled_height(200.0)
        .show(ui, |ui| {
            note_roll_canvas(store, ui);
        });
}

fn new_note_button(store: &Store, ui: &mut Ui) {
    let track_index = 0;
    if ui.button("New note").clicked() {
        store.dispatch(
            &Selector::Track(track_index),
            Action::AddNote(PlacedNote {
                note: Note {
                    pitch_name: PitchName {
                        scale_value: store.get().key,
                        octave: 4,
                    },
                    beats: 1.0,
                },
                offset: 0.0.into(),
            }),
        );
    }
}

fn note_roll_canvas(store: &Store, ui: &mut Ui) {
    let project_length = 16.0;
    let project_offset = 0.0;
    let quantise_ratio = 16.0;
    let track_index = 0;
    let max_pitch_value: PitchValue = PitchName {
        scale_value: ScaleValue::C,
        octave: 8,
    }
    .into();
    let min_pitch_value: PitchValue = PitchName {
        scale_value: ScaleValue::A,
        octave: 1,
    }
    .into();
    let piano_size = 50.0;
    // Major / Minor / Quarter
    let interval_durations: Vec<Beats> = vec![4.0, 1.0, 0.25];
    let canvas_height = 600.0;

    Frame::canvas(ui.style()).show(ui, |ui| {
        let (response, painter) =
            ui.allocate_painter(vec2(ui.available_width(), canvas_height), Sense::hover());

        let to_screen = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let note_roll = NoteRoll {
            size: response.rect.size(),
            project_length,
            project_offset,
            max_pitch_value: max_pitch_value as f32,
            min_pitch_value: min_pitch_value as f32 - 1.0,
            quantise_ratio,
            track_index,
            piano_size,
        };
        // Major / Minor / Quarter
        let interval_strokes: Vec<Stroke> = vec![6, 3, 1]
            .into_iter()
            .map(|alpha| Stroke::new(1.0, Color32::from_white_alpha(alpha)))
            .collect();

        let mut shapes: Vec<Shape> = interval_durations
            .into_iter()
            .zip(interval_strokes)
            .flat_map(|(duration, stroke)| note_roll.create_beat_lines(duration, stroke))
            .collect();

        shapes = transform_for_piano(shapes, &note_roll);
        shapes.extend(note_roll.create_pitch_value_shapes());
        shapes.extend(note_roll.create_note_shapes(&to_screen, store, ui, &response));
        let piano_shapes = note_roll.create_piano_keys();
        shapes.push(note_roll.make_piano_board());
        painter.extend(shapes.transform(to_screen));
        painter.extend(piano_shapes.transform(to_screen));

        response
    });
}

struct NoteOnRoll<'a> {
    note: PlacedNote,
    note_roll: &'a NoteRoll,
}

impl NoteOnRoll<'_> {
    pub fn get_pos(&self) -> Pos2 {
        let offset: f32 = self.note.offset.into();
        let pitch_value: i32 = self.note.note.pitch_name.into();
        self.note_roll.scale(pos2(
            offset - self.note_roll.project_offset,
            self.note_roll.max_pitch_value - pitch_value as f32,
        ))
    }

    pub fn make_new_note(&self, delta: Vec2) -> PlacedNote {
        // TODO Pass in the cursor position to improve the pitch_value responsiveness.
        let transformed_delta = self
            .note_roll
            .inverse_scale(self.note_roll.inverse_transform_for_piano(delta.to_pos2()));
        let offset_delta: OrderedFloat<f32> = transformed_delta.x.into();
        let pitch_value_delta: PitchValue = -transformed_delta.y.round() as i32;
        let curr_pitch_value: PitchValue = self.note.note.pitch_name.into();
        PlacedNote {
            note: Note {
                pitch_name: PitchName::from(curr_pitch_value + pitch_value_delta),
                beats: self.note.note.beats,
            },
            offset: self.note.offset + offset_delta,
        }
    }

    pub fn make_interactive_note_rect(&self) -> Rect {
        let note_height = 1.0;
        let note_size = self
            .note_roll
            .scale(pos2(self.note.note.beats, note_height));
        let top_left = self.get_pos();
        let bottom_right = top_left + note_size.to_vec2();
        Rect::from_min_max(
            self.note_roll.transform_for_piano(top_left),
            self.note_roll.transform_for_piano(bottom_right),
        )
    }

    pub fn create_background_note(&self) -> Rect {
        let note_height = 1.0;
        let note_size = self
            .note_roll
            .scale(pos2(self.note_roll.project_length, note_height));
        let top_left = self.get_pos();
        let bottom_right = top_left + note_size.to_vec2();
        Rect::from_min_max(
            self.note_roll.transform_for_piano(top_left),
            self.note_roll.transform_for_piano(bottom_right),
        )
    }

    pub fn make_piano_note(&mut self) -> PianoKey {
        // White notes are arranged so that the edge of B and C and the edge of
        // E and F lines align with the edge of the equivalent background.
        // This helps visual align background notes with the piano keys.
        // As a result white notes C, D, and E are slightly larger, spread out
        // over 5 background notes and F, G, A, and B slight smaller spread out
        // over 7.
        // y_offset indicates where the white note shape should start in
        // relation to the background note and note_height corresponding height
        // of that note.
        let mut white_key = true;
        let (y_offset, note_height) = match self.note.note.pitch_name.scale_value {
            ScaleValue::C => (-2.0 / 3.0, 5.0 / 3.0),
            ScaleValue::D => (-1.0 / 3.0, 5.0 / 3.0),
            ScaleValue::E => (0.0, 5.0 / 3.0),
            ScaleValue::F => (-3.0 / 4.0, 7.0 / 4.0),
            ScaleValue::G => (-1.0 / 2.0, 7.0 / 4.0),
            ScaleValue::A => (-1.0 / 4.0, 7.0 / 4.0),
            ScaleValue::B => (0.0, 7.0 / 4.0),
            // return black key note as regular size and offset
            _ => {
                white_key = false;
                let black_note_length_ratio = 0.6;
                self.note.note.beats = self.note.note.beats * black_note_length_ratio;
                (0.0, 1.0)
            }
        };
        let note_pos = self.get_pos();
        let y_translation = self
            .note_roll
            .scale(vec2(0.0, y_offset).to_pos2())
            .to_vec2();
        let note_size = self
            .note_roll
            .scale(pos2(self.note.note.beats, note_height))
            .to_vec2();
        let note_rect = Rect::from_min_size(note_pos + y_translation, note_size);
        if white_key {
            PianoKey::WHITE { note_rect }
        } else {
            PianoKey::BLACK { note_rect }
        }
    }
}

enum PianoKey {
    WHITE { note_rect: Rect },
    BLACK { note_rect: Rect },
}

enum NoteRollShape {
    InteractiveNote { note_rect: Rect },
    BackgroundNote { note_rect: Rect },
    PianoNote { key: PianoKey },
}

impl NoteRollShape {
    pub fn make_shape(self) -> Shape {
        match self {
            Self::InteractiveNote { note_rect } => {
                Shape::rect_filled(note_rect, CornerRadius::same(1), Color32::WHITE)
            }
            Self::BackgroundNote { note_rect } => Shape::rect_filled(
                note_rect,
                CornerRadius::same(0),
                Color32::from_white_alpha(2),
            ),
            Self::PianoNote { key } => match key {
                PianoKey::WHITE { note_rect } => Shape::rect_stroke(
                    note_rect,
                    CornerRadius::same(0),
                    Stroke::new(0.5, Color32::from_black_alpha(128)),
                    StrokeKind::Inside,
                ),
                PianoKey::BLACK { note_rect } => {
                    let corner_radius = 2;
                    Shape::rect_filled(
                        note_rect,
                        CornerRadius {
                            nw: 0,
                            ne: corner_radius,
                            sw: 0,
                            se: corner_radius,
                        },
                        Color32::BLACK,
                    )
                }
            },
        }
    }
}

struct NoteRoll {
    size: Vec2,
    project_length: f32,
    max_pitch_value: f32,
    min_pitch_value: f32,
    project_offset: f32,
    quantise_ratio: f32,
    track_index: usize,
    piano_size: f32,
}

impl NoteRoll {
    fn scale(&self, pos: Pos2) -> Pos2 {
        let scale = self.size
            / vec2(
                self.project_length - self.project_offset,
                self.max_pitch_value - self.min_pitch_value,
            );
        (pos.to_vec2() * scale).to_pos2()
    }

    fn inverse_scale(&self, pos: Pos2) -> Pos2 {
        let scale = self.scale(pos2(1.0, 1.0)).to_vec2();
        (pos.to_vec2() / scale).to_pos2()
    }

    fn make_piano_board(&self) -> Shape {
        let piano_board = Rect::from_min_max(pos2(0.0, 0.0), pos2(self.piano_size, self.size.y));
        Shape::rect_filled(piano_board, CornerRadius::same(0), Color32::WHITE)
    }

    fn transform_for_piano(&self, pos: Pos2) -> Pos2 {
        let from_rect = Rect::from_min_size(pos2(0.0, 0.0), self.size);
        let to_rect = Rect::from_min_size(
            pos2(self.piano_size, 0.0),
            self.size - vec2(self.piano_size, 0.0),
        );
        let rect_transform = RectTransform::from_to(from_rect, to_rect);
        rect_transform * pos
    }

    fn inverse_transform_for_piano(&self, pos: Pos2) -> Pos2 {
        let from_rect = Rect::from_min_size(pos2(0.0, 0.0), self.size - vec2(self.piano_size, 0.0));
        let to_rect = Rect::from_min_size(pos2(0.0, 0.0), self.size);
        let rect_transform = RectTransform::from_to(from_rect, to_rect);
        rect_transform * pos
    }

    pub fn create_note_shapes(
        &self,
        to_screen: &RectTransform,
        store: &Store,
        ui: &Ui,
        response: &Response,
    ) -> Vec<Shape> {
        store.get().project.tracks[self.track_index]
            .notes
            .iter()
            .enumerate()
            .map(|(note_index, note)| {
                let note_object = NoteOnRoll {
                    note: note.clone(),
                    note_roll: &self,
                };
                let next_note =
                    self.get_next_note(ui, to_screen, note_object, response, note_index);
                let quantised_note = self.quantise_note(next_note);
                self.dispatch_note(store, note, quantised_note.clone(), note_index);
                let new_note_object = NoteOnRoll {
                    note: quantised_note,
                    note_roll: &self,
                };
                NoteRollShape::InteractiveNote {
                    note_rect: new_note_object.make_interactive_note_rect(),
                }
                .make_shape()
            })
            .collect()
    }

    fn get_next_note(
        &self,
        ui: &Ui,
        to_screen: &RectTransform,
        note_object: NoteOnRoll,
        response: &Response,
        note_index: usize,
    ) -> PlacedNote {
        let note_rect = note_object.make_interactive_note_rect();
        let note_id = response.id.with(note_index);
        let note_response =
            ui.interact(to_screen.transform_rect(note_rect), note_id, Sense::drag());
        let note_delta = note_response.drag_delta();
        note_object.make_new_note(note_delta)
    }

    fn quantise_note(&self, note: PlacedNote) -> PlacedNote {
        let prev_offset: f32 = note.offset.into();
        let next_offset = (prev_offset * self.quantise_ratio).round() / self.quantise_ratio;
        PlacedNote {
            note: note.note,
            offset: next_offset.into(),
        }
    }

    fn dispatch_note(
        &self,
        store: &Store,
        note: &PlacedNote,
        next_note: PlacedNote,
        note_index: usize,
    ) {
        let track_index = 0;
        let sel = Selector::Note(track_index, note_index);
        if next_note.offset != note.offset {
            store.dispatch(&sel, Action::SetNoteOffset(next_note.offset.into()));
        };
        if next_note.note.pitch_name.octave != note.note.pitch_name.octave {
            store.dispatch(
                &sel,
                Action::SetNoteOctave(next_note.note.pitch_name.octave),
            );
        }
        if next_note.note.pitch_name.scale_value != note.note.pitch_name.scale_value {
            store.dispatch(
                &sel,
                Action::SetNoteScaleValue(next_note.note.pitch_name.scale_value),
            );
        }
    }

    pub fn create_pitch_value_shapes(&self) -> Vec<Shape> {
        ((self.min_pitch_value as i32)..=(self.max_pitch_value as i32))
            .filter(|pitch_value| pitch_value % 2 == 0)
            .map(|pitch_value| {
                let background_note = PlacedNote {
                    note: Note {
                        pitch_name: pitch_value.into(),
                        beats: self.project_length,
                    },
                    offset: 0.0.into(),
                };
                let note_rect = NoteOnRoll {
                    note: background_note,
                    note_roll: self,
                }
                .create_background_note();
                NoteRollShape::BackgroundNote { note_rect }.make_shape()
            })
            .collect()
    }

    fn create_beat_lines(&self, beat_increment: f32, stroke: Stroke) -> Vec<Shape> {
        ((self.project_offset as i32)..=(self.project_length / beat_increment) as i32)
            .map(|beat| {
                let x = (beat as f32) * beat_increment / self.project_length;
                let top = (pos2(x, 0.0).to_vec2() * self.size).to_pos2();
                let bottom = (pos2(x, 1.0).to_vec2() * self.size).to_pos2();
                Shape::line_segment([top, bottom], stroke)
            })
            .collect()
    }

    fn create_piano_keys(&self) -> Vec<Shape> {
        let piano_size_beats = self.piano_size / self.size.x * self.project_length;
        ((self.min_pitch_value as i32)..=(self.max_pitch_value as i32))
            .map(|pitch_value| {
                let pitch_name = PitchName::from(pitch_value);
                let note = PlacedNote {
                    note: Note {
                        pitch_name,
                        beats: piano_size_beats,
                    },
                    offset: 0.0.into(),
                };
                let key = NoteOnRoll {
                    note,
                    note_roll: &self,
                }
                .make_piano_note();
                NoteRollShape::PianoNote { key }.make_shape()
            })
            .collect()
    }
}

fn shrink_rect_left(rect: Rect, x: f32, note_roll: &NoteRoll) -> Rect {
    let from_rect = Rect::from_min_size(pos2(0.0, 0.0), note_roll.size);
    let to_rect = Rect::from_min_size(pos2(x, 0.0), vec2(note_roll.size.x - x, note_roll.size.y));
    let rect_transform = RectTransform::from_to(from_rect, to_rect);
    rect_transform.transform_rect(rect)
}

fn shrink_line_left(points: [Pos2; 2], x: f32, note_roll: &NoteRoll) -> [Pos2; 2] {
    let rescale = 1.0 / note_roll.size.x * (note_roll.size.x - x);
    [
        pos2(points[0].x * rescale + x, points[0].y),
        pos2(points[1].x * rescale + x, points[1].y),
    ]
}

fn transform_for_piano(shapes: Vec<Shape>, note_roll: &NoteRoll) -> Vec<Shape> {
    shapes
        .iter()
        .map(|shape| match shape {
            Shape::Rect(rect_shape) => Shape::Rect(RectShape {
                rect: shrink_rect_left(rect_shape.rect, note_roll.piano_size, note_roll),
                ..rect_shape.clone()
            }),
            Shape::LineSegment { points, stroke } => Shape::LineSegment {
                points: shrink_line_left(*points, note_roll.piano_size, note_roll),
                stroke: *stroke,
            },
            _ => panic!("{}", format!("Shape {:?} not implemented.", shape)),
        })
        .collect()
}

trait Transformable<T> {
    fn transform(self, rect: RectTransform) -> T;
}

impl Transformable<Rect> for Rect {
    fn transform(self, rect: RectTransform) -> Rect {
        rect.transform_rect(self)
    }
}

impl Transformable<Shape> for Shape {
    fn transform(self, rect: RectTransform) -> Shape {
        match self {
            Shape::LineSegment { points, stroke } => Shape::LineSegment {
                points: [rect * points[0], rect * points[1]],
                stroke,
            },
            Shape::Rect(rect_shape) => Shape::Rect(RectShape {
                rect: rect.transform_rect(rect_shape.rect),
                ..rect_shape
            }),
            _ => panic!("Shape not implemented."),
        }
    }
}

impl Transformable<Vec<Shape>> for Vec<Shape> {
    fn transform(self, rect: RectTransform) -> Vec<Shape> {
        self.iter()
            .map(|shape| shape.clone().transform(rect))
            .collect()
    }
}
