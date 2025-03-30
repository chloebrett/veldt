use egui::{Pos2, Rect, pos2, vec2};
use shared::{
    model::{PitchName, PlacedNote},
    types::PitchValue,
};

use super::shapes::NoteRollShape;

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

    pub fn make_roll_shapes(&self) -> Vec<NoteRollShape> {
        let mut roll_objects = vec![];
        roll_objects.extend(self.make_all_background_notes(self.get_background_notes()));
        roll_objects.extend(self.make_all_bar_lines());
        roll_objects.extend(self.make_all_interactive_notes());
        roll_objects
    }

    fn get_background_notes(&self) -> Vec<PitchName> {
        (self.min_note..=self.max_note)
            .filter(|pitch_value| pitch_value % 2 == 0)
            .map(|pitch_value| pitch_value.into())
            .collect()
    }

    fn make_all_background_notes(&self, notes: Vec<PitchName>) -> Vec<NoteRollShape> {
        notes
            .iter()
            .map(|&note| self.make_background_note(note))
            .collect()
    }

    fn make_background_note(&self, note: PitchName) -> NoteRollShape {
        let pitch_value: PitchValue = note.into();
        let note_pos = pos2(0.0, (self.max_note - pitch_value) as f32);
        let note_size = vec2(self.bars * self.bar_length, 1.0);
        NoteRollShape::BackgroundNote {
            note_rect: Rect::from_min_size(note_pos, note_size),
        }
    }

    fn make_all_interactive_notes(&self) -> Vec<NoteRollShape> {
        self.notes
            .iter()
            .map(|note| self.make_interactive_note(note.clone()))
            .collect()
    }

    fn make_interactive_note(&self, note: PlacedNote) -> NoteRollShape {
        NoteRollShape::InteractiveNote {
            note_rect: self.make_interactive_rect(note),
        }
    }

    pub fn make_interactive_rect(&self, note: PlacedNote) -> Rect {
        let pitch_value: PitchValue = note.note.pitch_name.into();
        let offset: f32 = note.offset.into();
        let note_pos = pos2(offset - self.offset, (self.max_note - pitch_value) as f32);
        let note_size = vec2(note.note.beats, 1.0);
        Rect::from_min_size(note_pos, note_size)
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
