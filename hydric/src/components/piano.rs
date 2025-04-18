use crate::{transform::Transform, view::View, widget::SequencerObject};
use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
    emath::RectTransform, pos2, vec2,
};
use shared::{
    model::{Note, PlacedNote, ScaleValue},
    types::PitchValue,
};

pub struct Piano {
    max_note: PitchValue,
    min_note: PitchValue,
    size: Vec2,
}

impl Piano {
    pub fn new(max_note: PitchValue, min_note: PitchValue) -> Self {
        Piano {
            max_note,
            min_note,
            size: vec2(50.0, 600.0),
        }
    }

    fn make_piano_board(&self, range: Rect) -> Shape {
        Shape::rect_filled(
            Rect::from_min_size(Pos2::ZERO, range.size()),
            CornerRadius::ZERO,
            Color32::WHITE,
        )
    }

    fn get_piano_notes(&self, range: Rect) -> Vec<PlacedNote> {
        (range.top() as i32..=range.bottom() as i32)
            .map(|pitch_value| PlacedNote {
                note: Note {
                    pitch_name: pitch_value.into(),
                    beats: 0.0,
                },
                offset: 0.0.into(),
            })
            .collect()
    }

    fn make_all_piano_keys(&self, notes: Vec<PlacedNote>, range: Rect) -> Vec<Shape> {
        notes
            .into_iter()
            .map(|note| self.make_piano_key(note, range))
            .collect()
    }

    fn make_piano_key(&self, note: PlacedNote, range: Rect) -> Shape {
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
            _ => return self.make_black_key(note, range),
        };
        self.make_white_key(note, y_offset, note_height, range)
    }

    fn make_white_key(
        &self,
        note: PlacedNote,
        y_offset: f32,
        note_height: f32,
        range: Rect,
    ) -> Shape {
        let note_pos = note.to_pos(range) + vec2(0.0, y_offset);
        let note_size = vec2(1.0, note_height);
        let rect = Rect::from_min_size(note_pos, note_size);
        Shape::rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, Color32::from_black_alpha(64)),
            StrokeKind::Inside,
        )
    }

    fn make_black_key(&self, note: PlacedNote, range: Rect) -> Shape {
        let black_note_length = 0.6;
        let note_pos = note.to_pos(range);
        let note_size = vec2(black_note_length, 1.0);
        let rect = Rect::from_min_size(note_pos, note_size);
        Shape::rect_filled(
            rect,
            // No radius on the left and small radius on the right.
            // To reflect the actual shape of black keys on a piano.
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

impl View for Piano {
    fn ui(&mut self, ui: &mut Ui) {
        let Piano {
            max_note,
            min_note,
            size,
        } = *self;
        let range = Rect::from_min_max(pos2(0.0, min_note as f32), pos2(1.0, max_note as f32));
        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(size, Sense::hover());
            let piano_transform = RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, range.size()),
                response.rect,
            );
            let piano_board = self.make_piano_board(range);
            let piano_keys = self.make_all_piano_keys(self.get_piano_notes(range), range);
            painter.extend(vec![piano_board].transform(piano_transform));
            painter.extend(piano_keys.transform(piano_transform))
        });
    }
}
