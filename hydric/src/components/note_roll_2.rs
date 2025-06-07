use std::ops::RangeInclusive;

use egui::{
    lerp, pos2, remap_clamp, vec2, CornerRadius, CursorIcon, Pos2, Rangef, Rect, Response, Sense, StrokeKind, Ui, Vec2, Widget
};
use shared::model::{self, PlacedNote};

use super::piano::Piano;

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
            duration: object.note.beats,
        }
    }
}

impl From<Note> for PlacedNote {
    fn from(object: Note) -> Self {
        Self {
            note: model::Note {
                pitch_name: (object.y as i32).into(),
                beats: object.duration,
            },
            offset: object.x.into(),
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
    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

    #[inline]
    pub fn duration(mut self, duration: RangeInclusive<f32>) -> Self {
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
        pos_from_normalised(normalised, self.get_duration(), self.get_range())
    }

    fn position_from_pos(
        &self,
        pos: Pos2,
        x_position_range: Rangef,
        y_position_range: Rangef,
    ) -> Pos2 {
        let normalised = normalised_from_pos(pos, self.get_duration(), self.get_range());
        lerp_pos(x_position_range, y_position_range, normalised)
    }

    fn note_rect(&self, rect: &Rect, index: usize) -> Rect {
        let note = self.get_note(index);
        let x_position_range = rect.x_range();
        let y_position_range = rect.y_range();
        let note_min = pos2(note.x, note.y);
        let note_size = vec2(note.duration, -1.0);
        let min = self.position_from_pos(note_min, x_position_range, y_position_range);
        let max = self.position_from_pos(note_min + note_size, x_position_range, y_position_range);
        Rect::from_min_max(min, max)
    }

    fn note_drag(&mut self, roll_response: &Response, response: &Response, index: usize) {
        let rect = response.rect;
        let x_position_range = roll_response.rect.x_range();
        let y_position_range = roll_response.rect.y_range();
        if let Some(pointer_position) = response.interact_pointer_pos() {
            let delta = response.drag_delta();
            let new_position = pos2(rect.left() + delta.x, pointer_position.y);
            let new_pos = self.pos_from_position(new_position, x_position_range, y_position_range);
            self.set_note_pos(index, new_pos);
        }
    }

    fn note_resize(&mut self, roll_response: &Response, response: &Response, index: usize) {
        let rect = response.rect;
        let x_position_range = roll_response.rect.x_range();
        let y_position_range = roll_response.rect.y_range();
        if let Some(pointer_position) = response.interact_pointer_pos() {
            let delta = response.drag_delta();
            let new_position = pos2(rect.right() + delta.x, pointer_position.y);
            let prev_x = self
                .pos_from_position(rect.right_top(), x_position_range, y_position_range)
                .x;
            let next_x = self
                .pos_from_position(new_position, x_position_range, y_position_range)
                .x;
            let duration_delta = next_x - prev_x;
            self.set_note_duration(index, self.get_note(index).duration + duration_delta);
        }
    }

    fn note_interact(&mut self, ui: &mut Ui, response: &Response, index: usize) {
        let rect = self.note_rect(&response.rect, index);
        let drag_id = response.id.with(format!("drag_{index}"));
        let resize_id = response.id.with(format!("resize_{index}"));
        let drag_resize_split = 0.8;
        let (drag_rect, resize_rect) = rect.split_left_right_at_fraction(0.8);
        let drag_response = ui.interact(drag_rect, drag_id, Sense::drag());
        let resize_response = ui.interact(resize_rect, resize_id, Sense::drag());
        self.note_drag(&response, &drag_response, index);
        self.note_resize(&response, &resize_response, index);
        self.note_ui(ui, &rect, &drag_response, &resize_response);
    }

    fn notes_interact(&mut self, ui: &mut Ui, response: &Response) {
        for index in 0..self.get_notes().len() {
            self.note_interact(ui, response, index);
        }
    }

    fn note_ui(&self, ui: &Ui, rect: &Rect, drag_response: &Response, resize_response: &Response) {
        let active = |response: &Response| {
            response.is_pointer_button_down_on() || response.has_focus() || response.clicked()
        };
        let hovered = |response: &Response| response.hovered() || response.highlighted();
        let visuals = &if active(drag_response) || active(resize_response) {
            ui.style().visuals.widgets.active
        } else if hovered(drag_response) || hovered(resize_response) {
            ui.style().visuals.widgets.hovered
        } else {
            ui.style().visuals.widgets.inactive
        };
        let centre = rect.center();
        let size = rect.size() + Vec2::splat(visuals.expansion);
        let rect = Rect::from_center_size(centre, size);
        ui.painter().rect(
            rect,
            visuals.corner_radius,
            visuals.bg_fill,
            visuals.fg_stroke,
            StrokeKind::Inside,
        );
        if resize_response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::ResizeColumn);
        }
    }

    fn background_ui(&self, ui: &Ui, response: &Response) {
        let white_notes = [0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0];
        let x_position_range = response.rect.x_range();
        let y_position_range = response.rect.y_range();
        let duration = self.get_duration();
        let range = self.get_range();
        for pitch in *range.end() as usize..=*range.start() as usize {
            let pitch = pitch as f32;
            let min = pos2(*duration.start(), pitch);
            let size = vec2(*duration.end(), -1.0);
            let min_position = self.position_from_pos(min, x_position_range, y_position_range);
            let max_position =
                self.position_from_pos(min + size, x_position_range, y_position_range);
            let rect = Rect::from_min_max(min_position, max_position);
            let colour = if white_notes.contains(&(pitch % 12.0)) {
                ui.style().visuals.faint_bg_color
            } else {
                ui.style().visuals.extreme_bg_color
            };
            ui.painter().rect_filled(rect, CornerRadius::ZERO, colour);
        }
    }
    fn notes_context_menu() {}
    fn roll_context_menu() {}
    fn add_contents(&mut self, ui: &mut Ui) -> Response {
        let response = ui.allocate_response(self.size, Sense::drag());
        self.background_ui(ui, &response);
        self.notes_interact(ui, &response);
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
    duration: RangeInclusive<f32>,
    range: RangeInclusive<f32>,
) -> Pos2 {
    remap_clamp_pos(pos, duration, range, 0.0..=1.0, 0.0..=1.0)
}

fn pos_from_normalised(
    normalised: Pos2,
    duration: RangeInclusive<f32>,
    range: RangeInclusive<f32>,
) -> Pos2 {
    lerp_pos(duration, range, normalised)
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
