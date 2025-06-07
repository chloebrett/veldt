use std::ops::RangeInclusive;

use egui::{lerp, pos2, remap_clamp, vec2, Pos2, Rangef, Rect, Response, Sense, StrokeKind, Ui, Vec2, Widget};
use shared::model::PlacedNote;

const MIN_PITCH_VALUE: i32 = 9;

const MAX_PITCH_VALUE: i32 = 96;

const PITCH_RANGE: RangeInclusive<f32> = MAX_PITCH_VALUE as f32..=MIN_PITCH_VALUE as f32;

#[derive(Clone, Debug)]
pub struct Note {
    x: f32,
    y: f32,
    duration: f32,
}

impl From<PlacedNote> for Note {
    fn from(object: PlacedNote) -> Self {
        let pitch_value: i32 = object.note.pitch_name.into();
        Self {
            x: object.offset.into(),
            y: pitch_value as f32,
            duration: object.note.beats
        }
    }
}

pub struct NoteRoll2<'a> {
    notes: &'a mut Vec<Note>,
    range: RangeInclusive<f32>,
    duration: RangeInclusive<f32>,
    size: Vec2,
}

impl<'a> NoteRoll2<'a> {
    pub fn new(notes: &'a mut Vec<Note>) -> Self {
        Self {
            notes,
            range: PITCH_RANGE,
            duration: 0.0..=8.0,
            size: vec2(400.0, 600.0),
        }
    }

    #[inline]
    fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

    #[inline]
    fn duration(mut self, duration: RangeInclusive<f32>) -> Self {
        self.duration = duration;
        self
    }

    fn get_notes(&self) -> Vec<Note> {
        self.notes.clone()
    }

    fn set_notes(&mut self, notes: Vec<Note>) {
        *self.notes = notes;
    }

    fn get_note(&self, index: usize) -> Note {
        self.notes[index].clone()
    }

    fn set_note(&mut self, index: usize, note: Note) {
        self.notes[index] = note
    }

    fn set_note_pos(&mut self, index: usize, pos: Pos2) {
        let mut note = self.get_note(index);
        note.x = pos.x;
        note.y = pos.y;
        self.set_note(index, note);
    }

    fn set_note_duration(&mut self, index: usize, duration: f32) {
        let mut note = self.get_note(index);
        note.duration = duration;
        self.set_note(index, note);
    }

    fn get_range(&self) -> RangeInclusive<f32> {
        self.range.clone()
    }

    fn get_duration(&self) -> RangeInclusive<f32> {
        self.duration.clone()
    }

    fn pos_from_position(
        &self,
        position: Pos2,
        x_position_range: Rangef,
        y_position_range: Rangef,
    ) -> Pos2 {
        let normalised = remap_clamp_pos(
            position,
            x_position_range,
            y_position_range,
            0.0..=1.0,
            0.0..=1.0,
        );
        pos_from_normalised(normalised, self.get_range(), self.get_duration())
    }

    fn position_from_pos(
        &self,
        pos: Pos2,
        x_position_range: Rangef,
        y_position_range: Rangef,
    ) -> Pos2 {
        let normalised = normalised_from_pos(pos, self.get_range(), self.get_duration());
        lerp_pos(x_position_range, y_position_range, normalised)
    }

    fn note_rect(&self, rect: &Rect, index: usize) -> Rect {
        let note = self.get_note(index);
        let x_position_range = rect.x_range();
        let y_position_range = rect.y_range();
        let note_pos = pos2(note.x, note.y);
        let note_size = vec2(note.duration, 1.0);
        let min = self.position_from_pos(note_pos, x_position_range, y_position_range);
        let size = self
            .position_from_pos(note_size.to_pos2(), x_position_range, y_position_range)
            .to_vec2();
        Rect::from_min_size(min, size)
    }

    fn rect_drag(&mut self, response: &Response, roll_response: &Response) -> Option<Pos2> {
        let rect = response.rect;
        let x_position_range = roll_response.rect.x_range();
        let y_position_range = roll_response.rect.y_range();
        if response.interact_pointer_pos().is_some() {
            let delta = response.drag_delta();
            let new_position = rect.left_top() + delta;
            let new_pos = self.pos_from_position(new_position, x_position_range, y_position_range);
            Some(new_pos)
        } else {
            None
        }
    }

    fn note_interact(&mut self, ui: &mut Ui, response: &Response, index: usize) {
        let rect = self.note_rect(&response.rect, index);
        // Split rect for drag and resize portions.
        let drag_resize_ratio = 0.2;
        let (drag_rect, resize_rect) = rect.split_left_right_at_fraction(drag_resize_ratio);
        let drag_id = response.id.with(format!("drag_{index}"));
        let resize_id = response.id.with(format!("resize_{index}"));
        let drag_response = ui.interact(drag_rect, drag_id, Sense::drag());
        let resize_response = ui.interact(resize_rect, resize_id, Sense::drag());
        if let Some(new_pos) = self.rect_drag(&drag_response, &response) {
            self.set_note_pos(index, new_pos);
        };
        if let Some(new_pos) = self.rect_drag(&resize_response, &response) {
            self.set_note_duration(index, new_pos.x);
        }
    }

    fn notes_interact(&mut self, ui: &mut Ui, response: &Response) {
        for index in 0..self.get_notes().len() {
            self.note_interact(ui, response, index);
        }
    }

    fn notes_ui(&self, ui: &Ui, response: &Response) {
        let visuals = ui.style().interact(response);
        let widget_visuals = &ui.visuals().widgets;
        for index in 0..self.get_notes().len() {
            let note_rect = self.note_rect(&response.rect, index);
            let centre = note_rect.center();
            let size = note_rect.size();
            let size = size + Vec2::splat(visuals.expansion);
            let rect = Rect::from_center_size(centre, size);
            ui.painter().rect(
                rect,
                visuals.corner_radius,
                visuals.bg_fill,
                visuals.fg_stroke,
                StrokeKind::Inside,
            );
        }

    }
    fn piano_ui() {}
    fn notes_context_menu() {}
    fn roll_context_menu() {}
    fn add_contents(&mut self, ui: &mut Ui) -> Response {
        let response = ui.allocate_response(self.size, Sense::drag());
        self.notes_interact(ui, &response);
        self.notes_ui(ui, &response);
        response
    }
}

impl Widget for NoteRoll2<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        self.add_contents(ui)
    }
}

/// Helper functions for converting Pos2 to and from [0, 1] range.

fn normalised_from_pos(
    pos: Pos2,
    range: RangeInclusive<f32>,
    duration: RangeInclusive<f32>,
) -> Pos2 {
    remap_clamp_pos(pos, range, duration, 0.0..=1.0, 0.0..=1.0)
}

fn pos_from_normalised(
    normalised: Pos2,
    range: RangeInclusive<f32>,
    duration: RangeInclusive<f32>,
) -> Pos2 {
    lerp_pos(range, duration, normalised)
}

fn remap_clamp_pos(
    pos: Pos2,
    x_from: impl Into<RangeInclusive<f32>>,
    y_from: impl Into<RangeInclusive<f32>>,
    x_to: impl Into<RangeInclusive<f32>>,
    y_to: impl Into<RangeInclusive<f32>>,
) -> Pos2 {
    pos2(
        remap_clamp(pos.x, x_from, x_to),
        remap_clamp(pos.y, y_from, y_to),
    )
}

fn lerp_pos(
    x_range: impl Into<RangeInclusive<f32>>,
    y_range: impl Into<RangeInclusive<f32>>,
    pos: Pos2,
) -> Pos2 {
    pos2(lerp(x_range, pos.x), lerp(y_range, pos.y))
}
