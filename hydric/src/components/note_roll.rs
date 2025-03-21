use crate::state::{Action, Selector, Store};

use egui::{
    Color32, CornerRadius, Frame, Painter, Pos2, Rect, Response, ScrollArea, Sense, Shape, Stroke,
    Ui, Vec2, emath::RectTransform,
};
use ordered_float::OrderedFloat;
use shared::{
    model::{Note, PitchName, PlacedNote, ScaleValue},
    types::PitchValue,
};

struct RollConfig {
    to_screen: RectTransform,
    x_size: f32,
    y_size: f32,
    inv_x_size: f32,
    inv_y_size: f32,
    project_length: f32,
    inv_project_length: f32,
    max_pitch_value: PitchValue,
    inv_max_pitch_value: f32,
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
            inv_x_size: 1.0 / x_size,
            inv_y_size: 1.0 / y_size,
            project_length,
            inv_project_length: 1.0 / project_length,
            max_pitch_value,
            inv_max_pitch_value: 1.0 / max_pitch_value as f32,
            project_offset,
            note_height: y_size / max_pitch_value as f32,
        }
    }
}

pub fn note_roll(store: &Store, ui: &mut Ui) {
    ScrollArea::vertical()
        .min_scrolled_height(200.0)
        .show(ui, |ui| {
            note_roll_canvas(store, ui);
        });
}

pub fn note_roll_canvas(store: &Store, ui: &mut Ui) {
    Frame::canvas(ui.style()).show(ui, |ui| {
        let (response, painter) =
            ui.allocate_painter(Vec2::new(ui.available_width(), 600.0), Sense::hover());
        let to_screen = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );
        let project_length = 16.0;
        let project_offset = 0.0;
        let max_pitch_value: PitchValue = PitchName {
            scale_value: ScaleValue::GSharp,
            octave: 8,
        }
        .into();
        let roll_config =
            RollConfig::new(to_screen, project_length, project_offset, max_pitch_value);
        let note_shapes = create_note_shapes(&store, &ui, &response, &roll_config);
        let pitch_value_shapes = create_pitch_value_shapes(&roll_config);
        let major_beat = 4.0;
        let minor_beat = 1.0;
        let major_stroke = Stroke::new(1.0, Color32::from_white_alpha(6));
        let minor_stroke = Stroke::new(1.0, Color32::from_white_alpha(3));
        create_beat_lines(&painter, major_beat, major_stroke, &roll_config);
        create_beat_lines(&painter, minor_beat, minor_stroke, &roll_config);
        painter.extend(pitch_value_shapes);
        painter.extend(note_shapes);
        response
    });
}

fn create_note_shapes(
    store: &Store,
    ui: &Ui,
    response: &Response,
    roll_config: &RollConfig,
) -> Vec<Shape> {
    let note_shapes: Vec<Shape> = store.get().project.tracks[0]
        .notes
        .clone()
        .iter_mut()
        .enumerate()
        .map(|(i, note)| {
            let offset: f32 = note.offset.into();
            let x1: f32 = offset * roll_config.inv_project_length * roll_config.x_size;
            let pitch_value: PitchValue = note.note.pitch_name.into();
            let pitch_ratio = 1.0 - pitch_value as f32 * roll_config.inv_max_pitch_value;
            let y1 = roll_config.y_size * pitch_ratio - roll_config.note_height;
            let note_pos = Pos2::new(x1, y1);
            let x2 =
                note_pos.x + note.note.beats * roll_config.inv_project_length * roll_config.x_size;
            let y2 = note_pos.y + roll_config.note_height;
            let note_bottom_right_corner = Pos2::new(x2, y2);
            let min_corner = roll_config.to_screen.transform_pos(note_pos);
            let max_corner = roll_config
                .to_screen
                .transform_pos(note_bottom_right_corner);
            let note_rect = Rect::from_min_max(min_corner, max_corner);
            let note_id = response.id.with(i);
            let note_response = ui.interact(note_rect, note_id, Sense::drag());
            let note_delta = note_response.drag_delta();
            let scaled_note_delta = Vec2 {
                x: note_delta.x * roll_config.inv_x_size * roll_config.project_length,
                y: -note_delta.y * roll_config.inv_y_size * roll_config.max_pitch_value as f32,
            };
            let offset_delta = scaled_note_delta.x;
            let pitch_delta: PitchValue = scaled_note_delta.y.round() as i32;
            let prev_offset: f32 = note.offset.into();
            let next_offset_raw = (prev_offset + offset_delta)
                .clamp(roll_config.project_offset, roll_config.project_length);
            let quantise_ratio = 16.0;
            let next_offset = (next_offset_raw * quantise_ratio).round() / quantise_ratio;
            let prev_pitch_value: PitchValue = note.note.pitch_name.into();
            let next_pitch_value =
                (prev_pitch_value + pitch_delta).clamp(0, roll_config.max_pitch_value);
            *note = PlacedNote {
                note: Note {
                    pitch_name: next_pitch_value.into(),
                    beats: note.note.beats,
                },
                offset: OrderedFloat(next_offset),
            };

            if (next_offset != prev_offset) || (next_pitch_value != prev_pitch_value) {
                let track_index = 0;
                let sel = Selector::Note(track_index, i);
                store.dispatch(&sel, Action::SetNoteOctave(note.note.pitch_name.octave));
                store.dispatch(
                    &sel,
                    Action::SetNoteScaleValue(note.note.pitch_name.scale_value),
                );
                store.dispatch(&sel, Action::SetNoteOffset(*note.offset));
            }
            Shape::rect_filled(note_rect, CornerRadius::same(1), Color32::WHITE)
        })
        .collect();
    note_shapes
}

fn create_pitch_value_shapes(roll_config: &RollConfig) -> Vec<Shape> {
    let mut pitch_value_shapes = vec![];
    for pitch_value in 0..roll_config.max_pitch_value {
        if pitch_value % 2 == 0 {
            let x1 = 0.0;
            let pitch_ratio = 1.0 - pitch_value as f32 * roll_config.inv_max_pitch_value;
            let y1 = roll_config.y_size * pitch_ratio - roll_config.note_height;
            let upper_corner = Pos2::new(x1, y1);
            let x2 = roll_config.x_size;
            let y2 = upper_corner.y + roll_config.note_height;
            let lower_right_corner = Pos2::new(x2, y2);
            let min_corner = roll_config.to_screen.transform_pos(upper_corner);
            let max_corner = roll_config.to_screen.transform_pos(lower_right_corner);
            let background_rect = Rect::from_min_max(min_corner, max_corner);
            let shape = Shape::rect_filled(
                background_rect,
                CornerRadius::same(0),
                Color32::from_white_alpha(2),
            );
            pitch_value_shapes.push(shape)
        }
    }
    pitch_value_shapes
}

fn create_beat_lines(
    painter: &Painter,
    beat_increment: f32,
    stroke: Stroke,
    roll_config: &RollConfig,
) {
    for beat in
        (roll_config.project_offset as i32)..=(roll_config.project_length / beat_increment) as i32
    {
        let x = roll_config.x_size * (beat as f32) * beat_increment / roll_config.project_length;
        let y1: f32 = 0.0;
        let y2: f32 = roll_config.y_size;
        let top_pos = roll_config.to_screen.transform_pos(Pos2::new(x, y1));
        let bottom_pos = roll_config.to_screen.transform_pos(Pos2::new(x, y2));
        painter.line(vec![top_pos, bottom_pos], stroke);
    }
}
