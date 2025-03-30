use egui::{Rect, Shape, pos2, vec2};
use shared::{
    model::{Note, PlacedNote, ScaleValue},
    types::PitchValue,
};

use super::{note_to_pos, shapes::NoteRollShape};

pub struct Piano {
    max_note: PitchValue,
    min_note: PitchValue,
}

impl Piano {
    pub fn new(max_note: PitchValue, min_note: PitchValue) -> Self {
        Piano { max_note, min_note }
    }

    pub fn make_piano_shapes(&self) -> Vec<Shape> {
        let mut objects = vec![self.make_piano_board()];
        objects.extend(self.make_all_piano_keys(self.get_piano_notes()));
        objects
            .into_iter()
            .map(|object| object.make_shape())
            .collect()
    }

    fn make_piano_board(&self) -> NoteRollShape {
        NoteRollShape::PianoBoard {
            rect: Rect::from_min_size(
                pos2(0.0, 0.0),
                vec2(1.0, (self.max_note - self.min_note) as f32),
            ),
        }
    }

    fn get_piano_notes(&self) -> Vec<PlacedNote> {
        (self.min_note..=self.max_note)
            .map(|pitch_value| PlacedNote {
                note: Note {
                    pitch_name: pitch_value.into(),
                    beats: 0.0,
                },
                offset: 0.0.into(),
            })
            .collect()
    }

    fn make_all_piano_keys(&self, notes: Vec<PlacedNote>) -> Vec<NoteRollShape> {
        notes
            .into_iter()
            .map(|note| self.make_piano_key(note))
            .collect()
    }

    fn make_piano_key(&self, note: PlacedNote) -> NoteRollShape {
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
            _ => return self.make_black_key(note),
        };
        self.make_white_key(note, y_offset, note_height)
    }

    fn make_white_key(&self, note: PlacedNote, y_offset: f32, note_height: f32) -> NoteRollShape {
        let note_pos = note_to_pos(&note, self.max_note, 0.0) + vec2(0.0, y_offset);
        let note_size = vec2(1.0, note_height);
        NoteRollShape::WhiteKey {
            note_rect: Rect::from_min_size(note_pos, note_size),
        }
    }

    fn make_black_key(&self, note: PlacedNote) -> NoteRollShape {
        let black_note_length = 0.6;
        let note_pos = note_to_pos(&note, self.max_note, 0.0);
        let note_size = vec2(black_note_length, 1.0);
        NoteRollShape::BlackKey {
            note_rect: Rect::from_min_size(note_pos, note_size),
        }
    }
}
