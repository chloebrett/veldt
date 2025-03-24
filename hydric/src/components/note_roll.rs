use std::collections::{HashMap, HashSet};

use crate::state::{Action, Selector, Store};
use mesic::create_scale_values;
use shared::{model::Scale, types::Beats};

use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Response, ScrollArea, Sense, Shape, Stroke,
    StrokeKind, Ui, Vec2, emath::RectTransform, epaint::RectShape, pos2, vec2,
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

        let mut piano_shapes = note_roll.create_white_keys();
        piano_shapes.extend(note_roll.create_black_keys());
        painter.extend(piano_shapes.transform(to_screen));
        shapes.extend(note_roll.create_pitch_value_shapes());
        shapes.extend(note_roll.create_note_shapes(&to_screen, store, ui, &response));
        shapes = note_roll.transform_for_piano(shapes);

        painter.extend(shapes.transform(to_screen));

        response
    });
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
                let note_shape = note_to_shape(note, self);
                let next_note = self.get_next_note(
                    ui,
                    to_screen,
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
        to_screen: &RectTransform,
        note: &PlacedNote,
        note_rect: Rect,
        response: &Response,
        note_index: usize,
    ) -> PlacedNote {
        let note_pos = note_to_pos2(note, self);
        let note_id = response.id.with(note_index);
        let note_response = ui.interact(note_rect.transform(*to_screen), note_id, Sense::drag());
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

    fn transform_for_piano(&self, shapes: Vec<Shape>) -> Vec<Shape> {
        shapes
            .iter()
            .map(|shape| {
                let rescale = 1.0 / self.size.x * (self.size.x - self.piano_size);
                match shape {
                    Shape::Rect(rect_shape) => {
                        let min_x = rect_shape.rect.left() * rescale + self.piano_size;
                        let max_x = rect_shape.rect.right() * rescale + self.piano_size;
                        let new_rect = Rect::from_min_max(
                            pos2(min_x, rect_shape.rect.top()),
                            pos2(max_x, rect_shape.rect.bottom()),
                        );

                        Shape::Rect(RectShape {
                            rect: new_rect,
                            ..rect_shape.clone()
                        })
                    }
                    Shape::LineSegment { points, stroke } => Shape::LineSegment {
                        points: [
                            pos2(points[0].x * rescale + self.piano_size, points[0].y),
                            pos2(points[1].x * rescale + self.piano_size, points[1].y),
                        ],
                        stroke: stroke.clone(),
                    },
                    _ => panic!("{}", format!("Shape {:?} not implemented.", shape)),
                }
            })
            .collect()
    }

    fn create_white_keys(&self) -> Vec<Shape> {
        let piano_size_beats = self.piano_size / self.size.x * self.project_length;
        let piano_board = Rect::from_min_max(pos2(0.0,0.0), pos2(self.piano_size, self.size.y));
        let mut shapes = vec![Shape::rect_filled(piano_board, CornerRadius::same(0), Color32::WHITE)];
        let white_values = HashSet::<ScaleValue>::from_iter(create_scale_values(Scale::Major, ScaleValue::C));
        ((self.min_pitch_value as i32)..=(self.max_pitch_value as i32) as i32)
            .filter(|&pitch_value| {
                let pitch_name = PitchName::from(pitch_value);
                white_values.contains(&pitch_name.scale_value)
            })
            .for_each(|pitch_value| {
                let pitch_name = PitchName::from(pitch_value);
                let note = PlacedNote {
                    note: Note {
                        pitch_name,
                        beats: piano_size_beats,
                    },
                    offset: 0.0.into(),
                };
                shapes.push(white_key_to_shape(&note, self))
            });
        shapes
    }

    fn create_black_keys(&self) -> Vec<Shape> {
        let piano_size_beats = self.piano_size / self.size.x * self.project_length;
        let white_values = create_scale_values(Scale::Major, ScaleValue::C);
        let mut white_value_map = HashMap::<ScaleValue, i32>::new();
        white_values
            .iter()
            .enumerate()
            .for_each(|(value_index, scale_value)| {
                white_value_map.insert(*scale_value, value_index as i32);
            });
        ((self.min_pitch_value as i32)..=(self.max_pitch_value as i32) as i32)
            .filter(|&pitch_value| {
                let pitch_name = PitchName::from(pitch_value);
                !white_value_map.contains_key(&pitch_name.scale_value)
            })
            .map(|pitch_value| {
                let pitch_name = PitchName::from(pitch_value);
                let note = PlacedNote {
                    note: Note {
                        pitch_name,
                        beats: piano_size_beats/2.0,
                    },
                    offset: 0.0.into(),
                };
                black_note_to_shape(&note, self)
            })
            .collect()
    }
}

fn note_to_pos2(note: &PlacedNote, note_roll: &NoteRoll) -> Pos2 {
    let scale = note_roll.size
        / vec2(
            note_roll.project_length,
            note_roll.max_pitch_value - note_roll.min_pitch_value,
        );
    let x: f32 = note.offset.into();
    let pitch_value: PitchValue = note.note.pitch_name.into();
    let y = note_roll.max_pitch_value - pitch_value as f32;
    (vec2(x, y) * scale).to_pos2()
}

fn note_to_shape(note: &PlacedNote, note_roll: &NoteRoll) -> Shape {
    let note_pos = note_to_pos2(note, note_roll);
    let scale = note_roll.size
        / vec2(
            note_roll.project_length,
            note_roll.max_pitch_value - note_roll.min_pitch_value,
        );
    let note_height = 1.0;
    let note_width = vec2(note.note.beats, note_height) * scale;
    let note_rect = Rect::from_min_size(note_pos, note_width);
    Shape::rect_filled(note_rect, CornerRadius::same(1), Color32::WHITE)
}

fn note_from_pos2(note: &PlacedNote, pos2: Pos2, note_roll: &NoteRoll) -> PlacedNote {
    let inverse_scale = vec2(
        note_roll.project_length,
        note_roll.max_pitch_value - note_roll.min_pitch_value,
    ) / note_roll.size;
    let note_values = pos2.to_vec2() * inverse_scale;
    let offset = note_values.x.clamp(0.0, note_roll.project_length);
    let pitch_value: PitchValue = ((note_roll.max_pitch_value - note_values.y).round() as i32)
        .clamp(
            note_roll.min_pitch_value as i32,
            note_roll.max_pitch_value as i32,
        );
    PlacedNote {
        note: Note {
            pitch_name: pitch_value.into(),
            beats: note.note.beats,
        },
        offset: offset.into(),
    }
}

fn white_key_to_shape(
    note: &PlacedNote,
    note_roll: &NoteRoll,
) -> Shape {
    // Offsets for whitenote layout.
    let (y1, y2) = match note.note.pitch_name.scale_value {
        ScaleValue::C => (-2.0/3.0, 5.0/3.0),
        ScaleValue::D => (-1.0/3.0, 5.0/3.0),
        ScaleValue::E => (0.0, 5.0/3.0),
        ScaleValue::F => (-3.0/4.0, 7.0/4.0),
        ScaleValue::G => (-1.0/2.0, 7.0/4.0),
        ScaleValue::A => (-1.0/4.0, 7.0/4.0),
        ScaleValue::B => (0.0, 7.0/4.0),
        _ => panic!("ScaleValue is not a white note.")
    };
    let scale = note_roll.size / vec2(note_roll.project_length, note_roll.max_pitch_value - note_roll.min_pitch_value);
    let note_pos = note_to_pos2(note, note_roll);
    let note_width = vec2(note.note.beats, y2) * scale;
    let white_note_pos = (note_pos.to_vec2() + vec2(0.0, y1) * scale).to_pos2();
    let note_rect = Rect::from_min_size(white_note_pos, note_width);
    Shape::rect_stroke(
        note_rect,
        CornerRadius::same(1),
        Stroke::new(1.0, Color32::from_black_alpha(128)),
        StrokeKind::Inside,
    )
}

fn black_note_to_shape(note: &PlacedNote, note_roll: &NoteRoll) -> Shape {
    let note_pos = note_to_pos2(note, note_roll);
    let scale = note_roll.size
        / vec2(
            note_roll.project_length,
            note_roll.max_pitch_value - note_roll.min_pitch_value,
        );
    let note_height = 1.0;
    let note_width = vec2(note.note.beats, note_height) * scale;
    let note_rect = Rect::from_min_size(note_pos, note_width);
    Shape::rect_filled(note_rect, CornerRadius::same(1), Color32::BLACK)
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
