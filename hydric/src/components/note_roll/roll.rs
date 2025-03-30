use egui::{Shape, pos2};
use shared::{
    model::{Note, PlacedNote},
    types::PitchValue,
};

use super::{make_note_rect, note_to_pos, shapes::NoteRollShape};

pub struct Roll {
    notes: Vec<PlacedNote>,
    max_note: PitchValue,
    min_note: PitchValue,
    offset: f32,
    bars: f32,
    bar_length: f32,
}

impl Roll {
    pub fn new(
        notes: Vec<PlacedNote>,
        max_note: PitchValue,
        min_note: PitchValue,
        offset: f32,
        bars: f32,
        bar_length: f32,
    ) -> Self {
        Roll {
            notes,
            max_note,
            min_note,
            offset,
            bars,
            bar_length,
        }
    }

    pub fn make_roll_shapes(&self) -> Vec<Shape> {
        let mut roll_objects = vec![];
        roll_objects.extend(self.make_all_background_notes(self.get_background_notes()));
        roll_objects.extend(self.make_all_bar_lines());
        roll_objects.extend(self.make_all_interactive_notes());
        roll_objects
            .into_iter()
            .map(|object| object.make_shape())
            .collect()
    }

    fn get_background_notes(&self) -> Vec<PlacedNote> {
        (self.min_note..=self.max_note)
            .filter(|pitch_value| pitch_value % 2 == 0)
            .map(|pitch_value| PlacedNote{
                note: Note {
                    pitch_name: pitch_value.into(),
                    beats: self.bar_length * self.bars
                },
                offset: 0.0.into()
            })
            .collect()
    }

    fn make_all_background_notes(&self, notes: Vec<PlacedNote>) -> Vec<NoteRollShape> {
        notes
            .into_iter()
            .map(|note| self.make_background_note(note))
            .collect()
    }

    fn make_background_note(&self, note: PlacedNote) -> NoteRollShape {
        let note_pos = note_to_pos(&note, self.max_note, self.offset); 
        NoteRollShape::BackgroundNote {
            note_rect: make_note_rect(&note, note_pos),
        }
    }

    fn make_all_interactive_notes(&self) -> Vec<NoteRollShape> {
        self.notes
            .iter()
            .map(|note| self.make_interactive_note(note.clone()))
            .collect()
    }

    fn make_interactive_note(&self, note: PlacedNote) -> NoteRollShape {
        let note_pos = note_to_pos(&note, self.max_note, self.offset); 
        NoteRollShape::InteractiveNote {
            note_rect: make_note_rect(&note, note_pos),
        }
    }

    fn make_all_bar_lines(&self) -> Vec<NoteRollShape> {
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

    fn make_bar_line(&self, beat: f32, order: u32) -> NoteRollShape {
        let line = [pos2(beat, 0.0_f32), pos2(beat, self.max_note as f32)];
        NoteRollShape::BarLine { line, order }
    }
}
