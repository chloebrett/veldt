use crate::state::{Action, Selector, Store};

use egui::{
    Frame, Pos2, Rect, Response, ScrollArea, Sense, Shape, Ui, Vec2, emath::RectTransform,
    epaint::RectShape, pos2, vec2,
};
use ordered_float::OrderedFloat;
use shared::{
    model::{Note, PitchName, PlacedNote, ScaleValue},
    types::PitchValue,
};

use super::{
    piano::{Piano, PianoObject},
    roll::{Roll, RollObject},
    shapes::NoteRollShape,
};

// TODO Integrate into Store and project.
struct ProjectConfig {
    pub max_note: PitchValue,
    pub min_note: PitchValue,
    pub offset: f32,
    pub bars: f32,
    pub bar_length: f32,
}

pub fn note_roll_display(store: &Store, ui: &mut Ui) {
    let track_index = 0;
    new_note_button(store, ui, track_index);
    ScrollArea::vertical()
        .min_scrolled_height(200.0)
        .show(ui, |ui| {
            draw_note_roll_canvas(store, ui, track_index);
        });
}

fn new_note_button(store: &Store, ui: &mut Ui, track_index: usize) {
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

fn draw_note_roll_canvas(store: &Store, ui: &mut Ui, track_index: usize) {
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
    let project_config = ProjectConfig {
        max_note,
        min_note: min_note - 1,
        offset,
        bars,
        bar_length,
    };
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
        let note_roll_canvas =
            NoteRollCanvas::new(store, project_config, size, piano_width, track_index);
        let shapes = note_roll_canvas.make_all_shapes();
        painter.extend(shapes.transform(to_screen));
        update_notes(
            ui,
            &response,
            to_screen,
            store,
            note_roll_canvas,
            track_index,
        );
        response
    });
}

fn note_to_pos(
    note: &PlacedNote,
    max_note: i32,
    min_note: i32,
    project_offset: f32,
    beats: f32,
) -> Pos2 {
    let offset: f32 = note.offset.into();
    let pitch_value: PitchValue = note.note.pitch_name.into();
    let x = (offset - project_offset).clamp(project_offset, beats);
    let y = (max_note - pitch_value).clamp(min_note, max_note) as f32;
    pos2(x, y)
}

fn make_note_rect(note: &PlacedNote, note_pos: Pos2) -> Rect {
    let note_size = vec2(note.note.beats, 1.0);
    Rect::from_min_size(note_pos, note_size)
}

fn update_notes(
    ui: &Ui,
    response: &Response,
    to_screen: RectTransform,
    store: &Store,
    note_roll_canvas: NoteRollCanvas,
    track_index: usize,
) {
    store.get().project.tracks[track_index]
        .notes
        .iter()
        .enumerate()
        .for_each(|(note_index, note)| {
            let note_rect = make_note_rect(
                note,
                note_to_pos(
                    note,
                    note_roll_canvas.project_config.max_note,
                    note_roll_canvas.project_config.min_note,
                    note_roll_canvas.project_config.offset,
                    note_roll_canvas.project_config.bar_length
                        * note_roll_canvas.project_config.bars,
                ),
            );
            let note_response = track_note_response(
                &ui,
                response,
                note_index,
                note_rect
                    .transform(note_roll_canvas.roll_transform)
                    .transform(to_screen),
            );
            let drag_pos = note_response.interact_pointer_pos();
            if let Some(pos) = drag_pos {
                let pos_as_note = pos
                    .transform(to_screen.inverse())
                    .transform(note_roll_canvas.roll_transform.inverse());
                let pos = note_roll_canvas.roll.clamp_pos(pos_as_note);
                let offset: OrderedFloat<f32> = pos.x.into();
                let pitch_name = PitchName::from(pos.y as i32);
                dispatch_note(store, note_index, offset, pitch_name, track_index);
            }
        });
}

fn track_note_response(
    ui: &Ui,
    response: &Response,
    note_index: usize,
    note_rect: Rect,
) -> Response {
    let shape_index = response.id.with(note_index);
    let note_response = ui.interact(note_rect, shape_index, Sense::drag());
    note_response
}

fn dispatch_note(
    store: &Store,
    note_index: usize,
    offset: OrderedFloat<f32>,
    pitch_name: PitchName,
    track_index: usize,
) {
    let curr_note = &store.get().project.tracks[track_index].notes[note_index];
    let sel = Selector::Note(track_index, note_index);
    if curr_note.offset != offset {
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
    project_config: ProjectConfig,
    piano_transform: RectTransform,
    roll_transform: RectTransform,
}

impl NoteRollCanvas {
    pub fn new(
        store: &Store,
        project_config: ProjectConfig,
        size: Vec2,
        piano_width: f32,
        track_index: usize,
    ) -> Self {
        let piano_transform = RectTransform::from_to(
            Rect::from_min_size(
                pos2(0.0, 0.0),
                vec2(
                    1.0,
                    (project_config.max_note - project_config.min_note) as f32,
                ),
            ),
            Rect::from_min_size(pos2(0.0, 0.0), vec2(piano_width, size.y)),
        );
        let roll_transform = RectTransform::from_to(
            Rect::from_min_size(
                pos2(0.0, 0.0),
                vec2(
                    project_config.bars * project_config.bar_length - project_config.offset,
                    (project_config.max_note - project_config.min_note) as f32,
                ),
            ),
            Rect::from_min_size(pos2(piano_width, 0.0), vec2(size.x - piano_width, size.y)),
        );
        NoteRollCanvas {
            piano: Piano::new(project_config.max_note, project_config.min_note),
            roll: Roll::new(
                store.get().project.tracks[track_index].notes.clone(),
                project_config.max_note,
                project_config.min_note,
                project_config.offset,
                project_config.bars,
                project_config.bar_length,
            ),
            project_config,
            piano_transform,
            roll_transform,
        }
    }

    pub fn make_all_shapes(&self) -> Vec<Shape> {
        // TODO Consider assigning render order values to shapes.
        let mut shapes = vec![];
        let roll_objects =
            self.transform_roll_objects(self.roll.make_all_objects(), self.roll_transform);
        shapes.extend(self.make_roll_shapes(roll_objects));
        let piano_objects =
            self.transform_piano_objects(self.piano.make_all_objects(), self.piano_transform);
        shapes.extend(self.make_piano_shapes(piano_objects));
        shapes
    }

    fn transform_roll_objects(
        &self,
        objects: Vec<RollObject>,
        roll_transform: RectTransform,
    ) -> Vec<RollObject> {
        objects
            .clone()
            .into_iter()
            .map(|object| object.transform(roll_transform))
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
