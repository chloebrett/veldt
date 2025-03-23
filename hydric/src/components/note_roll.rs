use crate::state::{Action, Selector, Store};

use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Response, ScrollArea, Sense, Shape, Stroke, Ui, Vec2,
    emath::RectTransform, epaint::RectShape, pos2, vec2,
};
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
    Frame::canvas(ui.style()).show(ui, |ui| {
        let (response, painter) =
            ui.allocate_painter(vec2(ui.available_width(), 600.0), Sense::hover());
        let to_screen = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );
        let mut note_roll = NoteRoll::new(
            to_screen,
            project_length,
            project_offset,
            max_pitch_value as f32,
            quantise_ratio,
            track_index,
        );
        let major_beat = 4.0;
        let minor_beat = 1.0;
        let quarter_beat = 0.25;
        let major_stroke = Stroke::new(1.0, Color32::from_white_alpha(6));
        let minor_stroke = Stroke::new(1.0, Color32::from_white_alpha(3));
        let quarter_stroke = Stroke::new(1.0, Color32::from_white_alpha(1));
        let mut shapes = vec![];
        shapes.extend(note_roll.create_beat_lines(major_beat, major_stroke));
        shapes.extend(note_roll.create_beat_lines(minor_beat, minor_stroke));
        shapes.extend(note_roll.create_beat_lines(quarter_beat, quarter_stroke));
        shapes.extend(note_roll.create_pitch_value_shapes());
        shapes.extend(note_roll.create_note_shapes(store, ui, &response));
        painter.extend(shapes.transform(to_screen));
        response
    });
}

struct NoteRoll {
    to_screen: RectTransform,
    size: Vec2,
    project_length: f32,
    max_pitch_value: f32,
    project_offset: f32,
    quantise_ratio: f32,
    track_index: usize,
}

impl NoteRoll {
    pub fn new(
        to_screen: RectTransform,
        project_length: f32,
        project_offset: f32,
        max_pitch_value: f32,
        quantise_ratio: f32,
        track_index: usize,
    ) -> Self {
        let y_size = to_screen.to().max.y - to_screen.to().min.y;
        let x_size = to_screen.to().max.x - to_screen.to().min.x;
        let size = vec2(x_size, y_size);
        NoteRoll {
            to_screen,
            size,
            project_length,
            max_pitch_value,
            project_offset,
            quantise_ratio,
            track_index,
        }
    }

    pub fn create_note_shapes(
        &mut self,
        store: &Store,
        ui: &Ui,
        response: &Response,
    ) -> Vec<Shape> {
        store.get().project.tracks[self.track_index]
            .notes
            .clone()
            .iter_mut()
            .enumerate()
            .map(|(note_index, note)| {
                let note_shape = note_to_shape(note, self);
                let note_value: PitchValue = note.note.pitch_name.scale_value.into();
                console::log_1(
                    &format!("{:?} {:?}", note.note.pitch_name.scale_value, note_value).into(),
                );
                let next_note = self.get_next_note(
                    ui,
                    note,
                    note_shape.visual_bounding_rect(),
                    response,
                    note_index,
                );
                let quantised_note = self.quantise_note(next_note);
                self.dispatch_note(store, note, quantised_note, note_index);
                note_shape
            })
            .collect()
    }

    fn get_next_note(
        &self,
        ui: &Ui,
        note: &mut PlacedNote,
        note_rect: Rect,
        response: &Response,
        note_index: usize,
    ) -> PlacedNote {
        let note_pos = note_to_pos2(note, self);
        let note_id = response.id.with(note_index);
        let note_response =
            ui.interact(note_rect.transform(self.to_screen), note_id, Sense::drag());
        let note_delta = note_response.drag_delta();
        let next_note_pos = note_pos + note_delta;
        note_from_pos2(note, next_note_pos, self)
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
        note: &mut PlacedNote,
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

    pub fn create_pitch_value_shapes(&mut self) -> Vec<Shape> {
        (0..(self.max_pitch_value as i32))
            .filter(|pitch_value| pitch_value % 2 == 0)
            .map(|pitch_value| {
                let background_note = PlacedNote {
                    note: Note {
                        pitch_name: pitch_value.into(),
                        beats: self.project_length,
                    },
                    offset: 0.0.into(),
                };
                let note_shape = note_to_shape(&background_note, self);
                let background_rect = note_shape.visual_bounding_rect();
                Shape::rect_filled(
                    background_rect,
                    CornerRadius::same(0),
                    Color32::from_white_alpha(2),
                )
            })
            .collect()
    }

    fn create_beat_lines(&mut self, beat_increment: f32, stroke: Stroke) -> Vec<Shape> {
        ((self.project_offset as i32)..=(self.project_length / beat_increment) as i32)
            .map(|beat| {
                let x = (beat as f32) * beat_increment / self.project_length;
                let top = (pos2(x, 0.0).to_vec2() * self.size).to_pos2();
                let bottom = (pos2(x, 1.0).to_vec2() * self.size).to_pos2();
                Shape::line_segment([top, bottom], stroke)
            })
            .collect()
    }
}

fn note_to_pos2(note: &PlacedNote, note_roll: &NoteRoll) -> Pos2 {
    let scale = note_roll.size / vec2(note_roll.project_length, note_roll.max_pitch_value);
    let x: f32 = note.offset.into();
    let pitch_value: PitchValue = note.note.pitch_name.into();
    let y = note_roll.max_pitch_value - pitch_value as f32;
    (vec2(x, y) * scale).to_pos2()
}

fn note_to_shape(note: &PlacedNote, note_roll: &NoteRoll) -> Shape {
    let note_pos = note_to_pos2(note, note_roll);
    let scale = note_roll.size / vec2(note_roll.project_length, note_roll.max_pitch_value);
    let note_height = 1.0;
    let note_width = vec2(note.note.beats, note_height) * scale;
    let note_rect = Rect::from_min_size(note_pos, note_width);
    Shape::rect_filled(note_rect, CornerRadius::same(1), Color32::WHITE)
}

fn note_from_pos2(note: &PlacedNote, pos2: Pos2, note_roll: &NoteRoll) -> PlacedNote {
    let inverse_scale = vec2(note_roll.project_length, note_roll.max_pitch_value) / note_roll.size;
    let note_values = pos2.to_vec2() * inverse_scale;
    let offset = note_values.x.clamp(0.0, note_roll.project_length);
    let pitch_value: PitchValue = ((note_roll.max_pitch_value - note_values.y).round() as i32)
        .clamp(0, note_roll.max_pitch_value as i32);
    PlacedNote {
        note: Note {
            pitch_name: pitch_value.into(),
            beats: note.note.beats,
        },
        offset: offset.into(),
    }
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
            _ => panic!("Shape note implemented."),
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
