use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
    Widget, emath::RectTransform, pos2, vec2,
};
use shared::{
    model::{Note, PlacedNote, ScaleValue},
    types::PitchValue,
};

use crate::transform::Transform;

pub struct Piano {
    max_note: PitchValue,
    min_note: PitchValue,
    size: Option<Vec2>,
}

impl Piano {
    pub fn new(max_note: PitchValue, min_note: PitchValue) -> Self {
        Piano {
            max_note,
            min_note,
            size: None,
        }
    }

    fn make_piano_board(&self, min_note: i32, max_note: i32) -> Shape {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(1.0, (max_note - min_note) as f32));
        Shape::rect_filled(rect, CornerRadius::ZERO, Color32::WHITE)
    }

    fn get_piano_notes(&self, min_note: i32, max_note: i32) -> Vec<PlacedNote> {
        (min_note..=max_note)
            .map(|pitch_value| PlacedNote {
                note: Note {
                    pitch_name: pitch_value.into(),
                    beats: 0.0,
                },
                offset: 0.0.into(),
            })
            .collect()
    }

    fn make_all_piano_keys(&self, notes: Vec<PlacedNote>, max_note: i32) -> Vec<Shape> {
        notes
            .into_iter()
            .map(|note| self.make_piano_key(note, max_note))
            .collect()
    }

    fn make_piano_key(&self, note: PlacedNote, max_note: i32) -> Shape {
        // White notes are arranged so that the edge of B and C and the edge of
        // E and F lines align with the edge of the equivalent background.
        // This helps visual align background notes with the piano keys.
        // As a result white notes C, D, and E are slightly larger, spread out
        // over 5 background notes and F, G, A, and B slight smaller spread out
        // over 7.
        // y_offset indicates where the white note shape should start in
        // relation to the background note and note_height corresponding height
        // of that note.
        let (y_offset, note_height) = match note.note.pitch_name.scale_value {
            ScaleValue::C => (-2.0 / 3.0, 5.0 / 3.0),
            ScaleValue::D => (-1.0 / 3.0, 5.0 / 3.0),
            ScaleValue::E => (0.0, 5.0 / 3.0),
            ScaleValue::F => (-3.0 / 4.0, 7.0 / 4.0),
            ScaleValue::G => (-1.0 / 2.0, 7.0 / 4.0),
            ScaleValue::A => (-1.0 / 4.0, 7.0 / 4.0),
            ScaleValue::B => (0.0, 7.0 / 4.0),
            // return black key note as regular size and offset
            _ => return self.make_black_key(note, max_note),
        };
        self.make_white_key(note, y_offset, note_height, max_note)
    }

    fn make_white_key(
        &self,
        note: PlacedNote,
        y_offset: f32,
        note_height: f32,
        max_note: i32,
    ) -> Shape {
        let note_pos = note_to_pos(&note, max_note, 0.0) + vec2(0.0, y_offset);
        let note_size = vec2(1.0, note_height);
        let rect = Rect::from_min_size(note_pos, note_size);
        Shape::rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, Color32::from_black_alpha(64)),
            StrokeKind::Inside,
        )
    }

    fn make_black_key(&self, note: PlacedNote, max_note: i32) -> Shape {
        let black_note_length = 0.6;
        let note_pos = note_to_pos(&note, max_note, 0.0);
        let note_size = vec2(black_note_length, 1.0);
        let rect = Rect::from_min_size(note_pos, note_size);
        Shape::rect_filled(
            rect,
            CornerRadius {
                nw: 0,
                ne: 2,
                sw: 0,
                se: 2,
            },
            Color32::BLACK,
        )
    }
}

impl Widget for Piano {
    fn ui(self, ui: &mut Ui) -> Response {
        let Piano {
            max_note,
            min_note,
            size,
        } = self;
        let size = size.unwrap_or(vec2(50.0, 600.0));
        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(size, Sense::hover());
            let piano_transform = RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, vec2(1.0, (max_note - min_note) as f32)),
                response.rect,
            );
            let piano_board = self.make_piano_board(min_note, max_note);
            let piano_keys =
                self.make_all_piano_keys(self.get_piano_notes(min_note, max_note), max_note);
            painter.extend(vec![piano_board].transform(piano_transform));
            painter.extend(piano_keys.transform(piano_transform))
        });
        let (_rect, response) = ui.allocate_at_least(Vec2::ZERO, Sense::hover());
        response
    }
}

pub fn note_to_pos(note: &PlacedNote, max_note: i32, project_offset: f32) -> Pos2 {
    let offset: f32 = note.offset.into();
    let x = offset - project_offset;
    let pitch_value: PitchValue = note.note.pitch_name.into();
    let y = max_note - pitch_value;
    pos2(x, y as f32)
}
