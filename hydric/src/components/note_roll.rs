use crate::state::{Action, Selector, Store};

use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Response, ScrollArea, Sense, Shape, Stroke, Ui, Vec2,
    emath::RectTransform, epaint::RectShape,
};
use shared::{
    model::{Note, PitchName, PlacedNote, ScaleValue},
    types::PitchValue,
};

struct RollConfig {
    to_screen: RectTransform,
    x_size: f32,
    y_size: f32,
    project_length: f32,
    max_pitch_value: PitchValue,
    project_offset: f32,
    note_height: f32,
}

impl RollConfig {
    pub fn new(
        to_screen: RectTransform,
        project_length: f32,
        project_offset: f32,
        max_pitch_value: PitchValue,
    ) -> Self {
        let y_size = to_screen.to().max.y - to_screen.to().min.y;
        let x_size = to_screen.to().max.x - to_screen.to().min.x;
        RollConfig {
            to_screen,
            x_size,
            y_size,
            project_length,
            max_pitch_value,
            project_offset,
            note_height: y_size / max_pitch_value as f32,
        }
    }
}

trait Renderable<T> {
    fn to_pos2(
        &self,
        project_length: f32,
        max_pitch_value: f32,
        canvas_x_size: f32,
        canvas_y_size: f32,
    ) -> Pos2;

    fn to_shape(
        &self,
        project_length: f32,
        max_pitch_value: f32,
        canvas_x_size: f32,
        canvas_y_size: f32,
    ) -> Shape;

    fn from_pos2(
        &self, // TODO Create new instance of PlacedNote
        pos2: Pos2,
        project_length: f32,
        max_pitch_value: f32,
        canvas_x_size: f32,
        canvas_y_size: f32,
    ) -> T;
}

impl Renderable<PlacedNote> for PlacedNote {
    fn to_pos2(
        &self,
        project_length: f32,
        max_pitch_value: f32,
        canvas_x_size: f32,
        canvas_y_size: f32,
    ) -> Pos2 {
        let scale = Vec2::new(
            canvas_x_size / project_length,
            canvas_y_size / max_pitch_value,
        );
        let x: f32 = self.offset.into();
        let pitch_value: PitchValue = self.note.pitch_name.into();
        let y = max_pitch_value - pitch_value as f32;
        (Vec2::new(x, y) * scale).to_pos2()
    }

    fn to_shape(
        &self,
        project_length: f32,
        max_pitch_value: f32,
        canvas_x_size: f32,
        canvas_y_size: f32,
    ) -> Shape {
        let note_pos = self.to_pos2(
            project_length,
            max_pitch_value,
            canvas_x_size,
            canvas_y_size,
        );
        let x_scale = canvas_x_size / project_length;
        let y_scale = canvas_y_size / max_pitch_value;
        let note_height = 1.0;
        let note_width = Vec2::new(self.note.beats * x_scale, note_height * y_scale);
        let note_rect = Rect::from_min_size(note_pos, note_width);
        Shape::rect_filled(note_rect, CornerRadius::same(1), Color32::WHITE)
    }

    fn from_pos2(
        &self,
        pos2: Pos2,
        project_length: f32,
        max_pitch_value: f32,
        canvas_x_size: f32,
        canvas_y_size: f32,
    ) -> PlacedNote {
        let inverse_scale = Vec2 {
            x: project_length / canvas_x_size,
            y: max_pitch_value / canvas_y_size,
        };
        let note_values = pos2.to_vec2() * inverse_scale;
        let offset = note_values.x.clamp(0.0, project_length);
        let pitch_value: PitchValue =
            ((max_pitch_value - note_values.y).round() as i32).clamp(0, max_pitch_value as i32);
        PlacedNote {
            note: Note {
                pitch_name: pitch_value.into(),
                beats: self.note.beats,
            },
            offset: offset.into(),
        }
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

pub fn note_roll(store: &Store, ui: &mut Ui) {
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
    ScrollArea::vertical()
        .min_scrolled_height(200.0)
        .show(ui, |ui| {
            note_roll_canvas(store, ui);
        });
}

fn note_roll_canvas(store: &Store, ui: &mut Ui) {
    let project_length = 16.0;
    let project_offset = 0.0;
    let quantise_ratio = 16.0;
    let max_pitch_value: PitchValue = PitchName {
        scale_value: ScaleValue::GSharp,
        octave: 8,
    }
    .into();
    Frame::canvas(ui.style()).show(ui, |ui| {
        let (response, painter) =
            ui.allocate_painter(Vec2::new(ui.available_width(), 600.0), Sense::hover());
        let to_screen = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );
        let roll_config =
            RollConfig::new(to_screen, project_length, project_offset, max_pitch_value);
        let note_shapes = create_note_shapes(store, ui, &response, &roll_config, quantise_ratio);
        let pitch_value_shapes = create_pitch_value_shapes(&roll_config);
        let major_beat = 4.0;
        let minor_beat = 1.0;
        let quarter_beat = 0.25;
        let major_stroke = Stroke::new(1.0, Color32::from_white_alpha(6));
        let minor_stroke = Stroke::new(1.0, Color32::from_white_alpha(3));
        let quarter_stroke = Stroke::new(1.0, Color32::from_white_alpha(1));
        let major_line_shapes = create_beat_lines(major_beat, major_stroke, &roll_config);
        let minor_line_shapes = create_beat_lines(minor_beat, minor_stroke, &roll_config);
        let quarter_line_shapes = create_beat_lines(quarter_beat, quarter_stroke, &roll_config);
        painter.extend(major_line_shapes.transform(to_screen));
        painter.extend(minor_line_shapes.transform(to_screen));
        painter.extend(quarter_line_shapes.transform(to_screen));
        painter.extend(pitch_value_shapes.transform(to_screen));
        painter.extend(note_shapes.transform(to_screen));
        response
    });
}

fn create_note_shapes(
    store: &Store,
    ui: &Ui,
    response: &Response,
    roll_config: &RollConfig,
    quantise_ratio: f32,
) -> Vec<Shape> {
    let note_shapes: Vec<Shape> = store.get().project.tracks[0]
        .notes
        .clone()
        .iter_mut()
        .enumerate()
        .map(|(note_idx, note)| {
            let note_shape = create_note_shape(note, roll_config);
            let next_note = get_next_note(
                ui,
                note,
                note_shape.visual_bounding_rect(),
                response,
                note_idx,
                roll_config,
            );
            let quantised_note = quantise_note(next_note, quantise_ratio);
            dispatch_note(store, note, quantised_note, note_idx);
            note_shape
        })
        .collect();
    note_shapes
}

fn create_note_shape(note: &mut PlacedNote, roll_config: &RollConfig) -> Shape {
    note.to_shape(
        roll_config.project_length,
        roll_config.max_pitch_value as f32,
        roll_config.x_size,
        roll_config.y_size,
    )
}

fn get_next_note(
    ui: &Ui,
    note: &mut PlacedNote,
    note_rect: Rect,
    response: &Response,
    note_idx: usize,
    roll_config: &RollConfig,
) -> PlacedNote {
    let note_pos2 = note.to_pos2(
        roll_config.project_length,
        roll_config.max_pitch_value as f32,
        roll_config.x_size,
        roll_config.y_size,
    );
    let note_id = response.id.with(note_idx);
    let note_response = ui.interact(
        note_rect.transform(roll_config.to_screen),
        note_id,
        Sense::drag(),
    );
    let note_delta = note_response.drag_delta();
    let next_note_pos2 = note_pos2 + note_delta;
    note.from_pos2(
        next_note_pos2,
        roll_config.project_length,
        roll_config.max_pitch_value as f32,
        roll_config.x_size,
        roll_config.y_size,
    )
}

fn quantise_note(note: PlacedNote, quantise_ratio: f32) -> PlacedNote {
    let prev_offset: f32 = note.offset.into();
    let next_offset = (prev_offset * quantise_ratio).round() / quantise_ratio;
    PlacedNote {
        note: note.note,
        offset: next_offset.into(),
    }
}

fn dispatch_note(store: &Store, note: &mut PlacedNote, next_note: PlacedNote, note_idx: usize) {
    let track_index = 0;
    let sel = Selector::Note(track_index, note_idx);
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

fn create_pitch_value_shapes(roll_config: &RollConfig) -> Vec<Shape> {
    let inv_max_pitch_value = 1.0 / roll_config.max_pitch_value as f32;
    (0..roll_config.max_pitch_value)
        .filter(|pitch_value| pitch_value % 2 == 0)
        .map(|pitch_value| {
            let pitch_ratio = 1.0 - pitch_value as f32 * inv_max_pitch_value;
            let y1 = roll_config.y_size * pitch_ratio - roll_config.note_height;
            let top_left = Pos2::new(0.0, y1);
            let y2 = y1 + roll_config.note_height;
            let bottom_left = Pos2::new(roll_config.x_size, y2);
            let background_rect = Rect::from_min_max(top_left, bottom_left);

            Shape::rect_filled(
                background_rect,
                CornerRadius::same(0),
                Color32::from_white_alpha(2),
            )
        })
        .collect()
}

fn create_beat_lines(beat_increment: f32, stroke: Stroke, roll_config: &RollConfig) -> Vec<Shape> {
    ((roll_config.project_offset as i32)..=(roll_config.project_length / beat_increment) as i32)
        .map(|beat| {
            let x =
                roll_config.x_size * (beat as f32) * beat_increment / roll_config.project_length;
            let top = Pos2::new(x, 0.0);
            let bottom = Pos2::new(x, roll_config.y_size);
            Shape::line_segment([top, bottom], stroke)
        })
        .collect()
}
