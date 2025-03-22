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

trait Transformable<T> {
    fn transform(self: Self, rect: RectTransform) -> T;
}

impl Transformable<Rect> for Rect {
    fn transform(self: Self, rect: RectTransform) -> Rect {
        rect.transform_rect(self)
    }
}

impl Transformable<Shape> for Shape {
    fn transform(self: Self, rect: RectTransform) -> Shape {
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
    fn transform(self: Self, rect: RectTransform) -> Vec<Shape> {
        self.iter()
            .map(|shape| shape.clone().transform(rect))
            .collect()
    }
}

pub fn note_roll(store: &Store, ui: &mut Ui) {
    ScrollArea::vertical()
        .min_scrolled_height(200.0)
        .show(ui, |ui| {
            note_roll_canvas(store, ui);
        });
}

fn note_roll_canvas(store: &Store, ui: &mut Ui) {
    let project_length = 16.0;
    let project_offset = 0.0;
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
        let note_shapes = create_note_shapes(store, ui, &response, &roll_config);
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
        painter.extend(note_shapes);
        response
    });
}

struct Inverses {
    inv_x_size: f32,
    inv_y_size: f32,
    inv_project_length: f32,
    inv_max_pitch_value: f32,
}

fn create_note_shapes(
    store: &Store,
    ui: &Ui,
    response: &Response,
    roll_config: &RollConfig,
) -> Vec<Shape> {
    let inverses = Inverses {
        inv_x_size: 1.0 / roll_config.x_size,
        inv_y_size: 1.0 / roll_config.y_size,
        inv_project_length: 1.0 / roll_config.project_length,
        inv_max_pitch_value: 1.0 / roll_config.max_pitch_value as f32,
    };
    let note_shapes: Vec<Shape> = store.get().project.tracks[0]
        .notes
        .clone()
        .iter_mut()
        .enumerate()
        .map(|(note_idx, note)| {
            let note_rect = create_note_rect(note, &roll_config, &inverses);
            let (offset_delta, pitch_delta) = get_note_delta(ui, note_rect, &response, note_idx, &roll_config, &inverses);
            let next_note = transform_note(note, offset_delta, pitch_delta, &roll_config);
            dispatch_note(store, note, next_note, note_idx);
            Shape::rect_filled(note_rect, CornerRadius::same(1), Color32::WHITE)
        })
        .collect();
    note_shapes
}

fn create_note_rect(note: &mut PlacedNote, roll_config: &RollConfig, inverses: &Inverses) -> Rect {
    let offset: f32 = note.offset.into();
    let x1: f32 = offset * inverses.inv_project_length * roll_config.x_size;
    let pitch_value: PitchValue = note.note.pitch_name.into();
    let pitch_ratio = 1.0 - pitch_value as f32 * inverses.inv_max_pitch_value;
    let y1 = roll_config.y_size * pitch_ratio - roll_config.note_height;
    let note_pos = Pos2::new(x1, y1);
    let x2 = note_pos.x + note.note.beats * inverses.inv_project_length * roll_config.x_size;
    let y2 = note_pos.y + roll_config.note_height;
    let note_bottom_right_corner = Pos2::new(x2, y2);
    Rect::from_min_max(note_pos, note_bottom_right_corner)
        .transform(roll_config.to_screen)
}

fn get_note_delta(ui: &Ui, note_rect: Rect, response: &Response, note_idx: usize, roll_config: &RollConfig, inverses: &Inverses) -> (f32, i32) {
    let note_id = response.id.with(note_idx);
    let note_response = ui.interact(note_rect, note_id, Sense::drag());
    let note_delta = note_response.drag_delta();
    let scaled_note_delta = Vec2 {
        x: note_delta.x * inverses.inv_x_size * roll_config.project_length,
        y: -note_delta.y * inverses.inv_y_size * roll_config.max_pitch_value as f32,
    };
    let offset_delta = scaled_note_delta.x;
    let pitch_delta: PitchValue = scaled_note_delta.y.round() as i32;
    (offset_delta, pitch_delta)
}

fn transform_note(note: &mut PlacedNote, offset_delta: f32, pitch_delta: i32, roll_config: &RollConfig) -> PlacedNote {
    let prev_offset: f32 = note.offset.into();
    let next_offset_raw = (prev_offset + offset_delta)
        .clamp(roll_config.project_offset, roll_config.project_length);
    let quantise_ratio = 16.0;
    let next_offset = (next_offset_raw * quantise_ratio).round() / quantise_ratio;
    let prev_pitch_value: PitchValue = note.note.pitch_name.into();
    let next_pitch_value =
        (prev_pitch_value + pitch_delta).clamp(0, roll_config.max_pitch_value);
    PlacedNote {
        note: Note {
            pitch_name: next_pitch_value.into(),
            beats: note.note.beats,
        },
        offset: next_offset.into(),
    }
}

fn dispatch_note(store: &Store, note: &mut PlacedNote, next_note: PlacedNote, note_idx: usize) {
    let track_index = 0;
    let sel = Selector::Note(track_index, note_idx);
    if next_note.offset != note.offset {
        store.dispatch(&sel, Action::SetNoteOffset(next_note.offset.into()));
    };
    if next_note.note.pitch_name != note.note.pitch_name {
        store.dispatch(&sel, Action::SetNoteOctave(next_note.note.pitch_name.octave));
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
