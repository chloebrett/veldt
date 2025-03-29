use crate::state::{Action, Selector, Store};

use egui::{
    emath::RectTransform, epaint::RectShape, pos2, vec2, Color32, CornerRadius, Frame, Pos2, Rect, Response, ScrollArea, Sense, Shape, Stroke, StrokeKind, Ui, Vec2
};
use ordered_float::OrderedFloat;
use shared::{
    model::{Note, PitchName, PlacedNote, ScaleValue},
    types::PitchValue,
};
use web_sys::console;

pub fn note_roll_display(store: &Store, ui: &mut Ui) {
    new_note_button(store, ui);
    ScrollArea::vertical()
        .min_scrolled_height(200.0)
        .show(ui, |ui| {
            draw_note_roll_canvas(store, ui);
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

fn draw_note_roll_canvas(store: &Store, ui: &mut Ui) {
    let offset = 0.0;
    let bar_length = 4.0;
    let bars = 4.0;
    let max_note: PitchValue = PitchName {
        scale_value: ScaleValue::C,
        octave: 8,
    }
    .into();
    let min_note: PitchValue = PitchName {
        scale_value: ScaleValue::A,
        octave: 1,
    }
    .into();
    let piano_width = 50.0;
    let canvas_height = 600.0;

    Frame::canvas(ui.style()).show(ui, |ui| {
        let (response, painter) =
            ui.allocate_painter(vec2(ui.available_width(), canvas_height), Sense::hover());

        let to_screen = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );
        let size = response.rect.size();
        let note_roll_canvas = NoteRollCanvas::new(
            store,
            max_note,
            min_note - 1, // Needed to fit all notes on canvas.
            // TODO  Fix properly.
            offset,
            bar_length,
            bars,
            size,
            piano_width,
        );
        let shapes = note_roll_canvas.make_all_shapes();
        painter.extend(shapes.transform(to_screen));
        update_notes(ui, &response, to_screen, store, note_roll_canvas);
        response
    });
}


fn update_notes(ui: &Ui, response: &Response, to_screen: RectTransform, store: &Store, note_roll_canvas: NoteRollCanvas) {
    let track_index = 0;
    store.get().project.tracks[track_index].notes.iter()
        .enumerate()
        .for_each(|(note_index, note)| {
            let note_rect = note_roll_canvas.roll.make_interactive_rect(note.clone());
            let note_response = track_note_response(&ui, response, note_index, note_rect.transform(note_roll_canvas.roll_transform).transform(to_screen));
            let drag_pos = note_response.interact_pointer_pos();
            if let Some(pos) = drag_pos { 
                let pos_as_note = pos.transform(to_screen.inverse()).transform(note_roll_canvas.roll_transform.inverse());
                let pos = note_roll_canvas.roll.clamp_pos(pos_as_note);
                let offset: OrderedFloat<f32> = pos.x.into();
                let pitch_name = PitchName::from(pos.y as i32);
                dispatch_note(store, note_index, offset, pitch_name);
            }
        });
}


fn track_note_response(ui: &Ui, response: &Response, note_index: usize, note_rect: Rect) -> Response {
    let shape_index = response.id.with(note_index);
    let note_response = ui.interact(note_rect, shape_index, Sense::drag());
    note_response 
}

fn dispatch_note(store: &Store, note_index: usize, offset: OrderedFloat<f32>, pitch_name: PitchName) {
    let track_index = 0;
    let curr_note = &store.get().project.tracks[track_index].notes[note_index];
    let sel = Selector::Note(track_index, note_index);
    if curr_note.offset !=  offset {
        store.dispatch(&sel, Action::SetNoteOffset(offset.into()))
    };
    if curr_note.note.pitch_name.octave != pitch_name.octave {
        store.dispatch(&sel, Action::SetNoteOctave(pitch_name.octave))
    };
    if curr_note.note.pitch_name.scale_value != pitch_name.scale_value {
        store.dispatch(&sel, Action::SetNoteScaleValue(pitch_name.scale_value));
    }
}

struct NoteRollCanvas {
    piano: Piano,
    roll: Roll,
    piano_transform: RectTransform,
    roll_transform: RectTransform,
}

impl NoteRollCanvas {
    pub fn new(
        store: &Store,
        max_note: PitchValue,
        min_note: PitchValue,
        offset: f32,
        bar_length: f32,
        bars: f32,
        size: Vec2,
        piano_width: f32,
    ) -> Self {
        let track_index = 0;
        let piano_transform = RectTransform::from_to(
            Rect::from_min_size(pos2(0.0, 0.0), vec2(1.0, (max_note - min_note) as f32)),
            Rect::from_min_size(pos2(0.0, 0.0), vec2(piano_width, size.y)),
        );
        let roll_transform = RectTransform::from_to(
            Rect::from_min_size(
                pos2(0.0, 0.0),
                vec2(bars * bar_length - offset, (max_note - min_note) as f32),
            ),
            Rect::from_min_size(pos2(piano_width, 0.0), vec2(size.x - piano_width, size.y)),
        );
        NoteRollCanvas {
            piano: Piano { max_note, min_note },
            roll: Roll {
                notes: store.get().project.tracks[track_index].notes.clone(),
                max_note,
                min_note,
                offset,
                bars,
                bar_length,
            },
            piano_transform,
            roll_transform,
        }
    }

    pub fn make_all_shapes(&self) -> Vec<Shape> {
        // TODO Consider assigning render order values to shapes.
        let mut shapes = vec![];
        let roll_objects =
            self.transform_roll_objects(self.make_roll_object(), self.roll_transform);
        shapes.extend(self.make_roll_shapes(roll_objects));
        let piano_objects =
            self.transform_piano_objects(self.make_piano_object(), self.piano_transform);
        shapes.extend(self.make_piano_shapes(piano_objects));
        shapes
    }

    fn make_roll_object(&self) -> Vec<RollObject> {
        self.roll.make_all_objects()
    }

    fn transform_roll_objects(
        &self,
        objects: Vec<RollObject>,
        roll_transform: RectTransform,
    ) -> Vec<RollObject> {
        objects
            .iter()
            .map(|object| object.clone().transform(roll_transform))
            .collect()
    }

    fn make_roll_shapes(&self, objects: Vec<RollObject>) -> Vec<Shape> {
        objects
            .iter()
            .map(|object| match object {
                RollObject::InteractiveNote { note_rect } => NoteRollShape::InteractiveNote {
                    note_rect: *note_rect,
                }
                .make_shape(),
                RollObject::BackgroundNote { note_rect } => NoteRollShape::BackgroundNote {
                    note_rect: *note_rect,
                }
                .make_shape(),
                RollObject::BarLine { line, order } => NoteRollShape::BarLine {
                    line: *line,
                    order: *order,
                }
                .make_shape(),
            })
            .collect()
    }

    fn make_piano_object(&self) -> Vec<PianoObject> {
        self.piano.make_all_objects()
    }

    fn transform_piano_objects(
        &self,
        objects: Vec<PianoObject>,
        piano_transform: RectTransform,
    ) -> Vec<PianoObject> {
        objects
            .iter()
            .map(|object| object.clone().transform(piano_transform))
            .collect()
    }

    fn make_piano_shapes(&self, objects: Vec<PianoObject>) -> Vec<Shape> {
        objects
            .iter()
            .map(|object| match object {
                PianoObject::WhiteKey { note_rect } => NoteRollShape::WhiteKey {
                    note_rect: *note_rect,
                }
                .make_shape(),
                PianoObject::BlackKey { note_rect } => NoteRollShape::BlackKey {
                    note_rect: *note_rect,
                }
                .make_shape(),
                PianoObject::Board { rect } => {
                    NoteRollShape::PianoBoard { rect: *rect }.make_shape()
                }
            })
            .collect()
    }
}

#[derive(Clone)]
enum RollObject {
    InteractiveNote { note_rect: Rect },
    BackgroundNote { note_rect: Rect },
    BarLine { line: [Pos2; 2], order: u32 },
}

struct Roll {
    notes: Vec<PlacedNote>,
    max_note: PitchValue,
    min_note: PitchValue,
    offset: f32,
    bars: f32,
    bar_length: f32,
}

impl Roll {
    pub fn make_all_objects(&self) -> Vec<RollObject> {
        let mut roll_objects = vec![];
        roll_objects.extend(self.make_all_background_notes(self.get_background_notes()));
        roll_objects.extend(self.make_all_bar_lines());
        roll_objects.extend(self.make_all_interactive_notes());
        roll_objects
    }

    fn clamp_pos(&self, pos: Pos2) -> Pos2 {
        let x = pos.x.clamp(self.offset, self.bars * self.bar_length);
        let cursor_offset = 1.0; // Default Y value of cursor position felt strange.
        let y = ((self.max_note as f32 - pos.y + cursor_offset) as i32).clamp(self.min_note, self.max_note) as f32;
        pos2(x, y)
    }

    fn get_background_notes(&self) -> Vec<PitchName> {
        (self.min_note..=self.max_note)
            .filter(|pitch_value| pitch_value % 2 == 0)
            .map(|pitch_value| pitch_value.into())
            .collect()
    }

    fn make_all_background_notes(&self, notes: Vec<PitchName>) -> Vec<RollObject> {
        notes
            .iter()
            .map(|&note| self.make_background_note(note))
            .collect()
    }

    fn make_background_note(&self, note: PitchName) -> RollObject {
        let pitch_value: PitchValue = note.into();
        let note_pos = pos2(0.0, (self.max_note - pitch_value) as f32);
        let note_size = vec2(self.bars * self.bar_length, 1.0);
        RollObject::BackgroundNote {
            note_rect: Rect::from_min_size(note_pos, note_size),
        }
    }

    fn make_all_interactive_notes(&self) -> Vec<RollObject> {
        self.notes
            .iter()
            .map(|note| self.make_interactive_note(note.clone()))
            .collect()
    }

    fn make_interactive_note(&self, note: PlacedNote) -> RollObject {
        RollObject::InteractiveNote {
            note_rect: self.make_interactive_rect(note)
        }
    }

    pub fn make_interactive_rect(&self, note: PlacedNote) -> Rect {
        let pitch_value: PitchValue = note.note.pitch_name.into();
        let offset: f32 = note.offset.into();
        let note_pos = pos2(offset - self.offset, (self.max_note - pitch_value) as f32);
        let note_size = vec2(note.note.beats, 1.0);
        Rect::from_min_size(note_pos, note_size)

    }

    fn make_all_bar_lines(&self) -> Vec<RollObject> {
        let max_order: u32 = 3;
        let mut bar_lines = vec![];
        for order in 0..max_order {
            let mut order_barlines = vec![];
            let n_bar_lines = (self.bars * self.bar_length.powf(order as f32)) as i32;
            for value in 0..=n_bar_lines {
                let beat = value as f32 * self.bar_length.powf(1.0 - order as f32);
                order_barlines.push(self.make_bar_line(beat, order))
            }
            bar_lines.extend(order_barlines)
        }
        bar_lines
    }

    fn make_bar_line(&self, beat: f32, order: u32) -> RollObject {
        let line = [pos2(beat, 0.0_f32), pos2(beat, self.max_note as f32)];
        RollObject::BarLine { line, order }
    }
}

#[derive(Clone)]
enum PianoObject {
    WhiteKey { note_rect: Rect },
    BlackKey { note_rect: Rect },
    Board { rect: Rect },
}

struct Piano {
    max_note: PitchValue,
    min_note: PitchValue,
}

impl Piano {
    pub fn make_all_objects(&self) -> Vec<PianoObject> {
        let mut objects = vec![self.make_piano_board()];
        objects.extend(self.make_all_piano_keys(self.get_piano_notes()));
        objects
    }

    fn make_piano_board(&self) -> PianoObject {
        PianoObject::Board {
            rect: Rect::from_min_size(
                pos2(0.0, 0.0),
                vec2(1.0, (self.max_note - self.min_note) as f32),
            ),
        }
    }

    fn get_piano_notes(&self) -> Vec<PitchName> {
        (self.min_note..=self.max_note)
            .map(|pitch_value| pitch_value.into())
            .collect()
    }

    fn make_all_piano_keys(&self, notes: Vec<PitchName>) -> Vec<PianoObject> {
        notes
            .iter()
            .map(|&note| self.make_piano_key(note))
            .collect()
    }

    fn make_piano_key(&self, note: PitchName) -> PianoObject {
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

    fn make_white_key(&self, note: PitchName, y_offset: f32, note_height: f32) -> PianoObject {
        let pitch_value: PitchValue = note.into();
        let note_pos = pos2(0.0, (self.max_note - pitch_value) as f32 + y_offset);
        let note_size = vec2(1.0, note_height);
        PianoObject::WhiteKey {
            note_rect: Rect::from_min_size(note_pos, note_size),
        }
    }

    fn make_black_key(&self, note: PitchName) -> PianoObject {
        let black_note_length = 0.6;
        let pitch_value: PitchValue = note.into();
        let note_pos = pos2(0.0, (self.max_note - pitch_value) as f32);
        let note_size = vec2(black_note_length, 1.0);
        PianoObject::BlackKey {
            note_rect: Rect::from_min_size(note_pos, note_size),
        }
    }
}

enum NoteRollShape {
    InteractiveNote { note_rect: Rect },
    BackgroundNote { note_rect: Rect },
    WhiteKey { note_rect: Rect },
    BlackKey { note_rect: Rect },
    PianoBoard { rect: Rect },
    BarLine { line: [Pos2; 2], order: u32 },
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
            Self::WhiteKey { note_rect } => Shape::rect_stroke(
                note_rect,
                CornerRadius::same(0),
                Stroke::new(0.5, Color32::from_black_alpha(128)),
                StrokeKind::Inside,
            ),
            Self::BlackKey { note_rect } => {
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
            Self::PianoBoard { rect } => {
                Shape::rect_filled(rect, CornerRadius::same(0), Color32::WHITE)
            }
            NoteRollShape::BarLine { line, order } => Shape::line_segment(
                line,
                Stroke {
                    width: 1.0,
                    color: Color32::from_white_alpha(2.0f32.powf((3 - order) as f32) as u8),
                },
            ),
        }
    }
}

trait Transformable<T> {
    fn transform(self, rect: RectTransform) -> T;
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

impl Transformable<[Pos2; 2]> for [Pos2; 2] {
    fn transform(self, rect: RectTransform) -> [Pos2; 2] {
        [rect * self[0], rect * self[1]]
    }
}

impl Transformable<Rect> for Rect {
    fn transform(self, rect: RectTransform) -> Rect {
        rect.transform_rect(self)
    }
}

impl Transformable<Pos2> for Pos2 {
    fn transform(self, rect: RectTransform) -> Pos2 {
        rect * self
    }
}

impl Transformable<PianoObject> for PianoObject {
    fn transform(self, rect: RectTransform) -> PianoObject {
        match self {
            PianoObject::WhiteKey { note_rect } => PianoObject::WhiteKey {
                note_rect: note_rect.transform(rect),
            },
            PianoObject::BlackKey { note_rect } => PianoObject::BlackKey {
                note_rect: note_rect.transform(rect),
            },
            PianoObject::Board { rect: piano_rect } => PianoObject::Board {
                rect: piano_rect.transform(rect),
            },
        }
    }
}

impl Transformable<RollObject> for RollObject {
    fn transform(self, rect: RectTransform) -> RollObject {
        match self {
            RollObject::InteractiveNote { note_rect } => RollObject::InteractiveNote {
                note_rect: note_rect.transform(rect),
            },
            RollObject::BackgroundNote { note_rect } => RollObject::BackgroundNote {
                note_rect: note_rect.transform(rect),
            },
            RollObject::BarLine { line, order } => RollObject::BarLine {
                line: line.transform(rect),
                order,
            },
        }
    }
}
