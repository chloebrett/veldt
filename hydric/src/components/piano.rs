use crate::{
    components::note_roll::NoteSequencerObject,
    playback::AudioPlayer,
    transform::{Transform, Yx},
    view::View,
};
use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
    emath::RectTransform, pos2, vec2,
};
use shared::{
    model::{Note, PlacedNote, ScaleValue},
    types::PitchValue,
};
use state::GeneratorSelector;

#[derive(PartialEq)]
pub enum PianoOrientation {
    Vertical,
    Horizontal,
}

pub struct Piano<'a> {
    max_note: PitchValue,
    min_note: PitchValue,
    size: Vec2,
    orientation: PianoOrientation,
    audio_player: Option<&'a mut AudioPlayer>,
    generator_selector: Option<GeneratorSelector>,
}

const BLACK_NOTE_LENGTH: f32 = 0.6;
const BLACK_NOTE_WIDTH: f32 = 1.0;

impl<'a> Piano<'a> {
    pub fn new(
        max_note: PitchValue,
        min_note: PitchValue,
        orientation: PianoOrientation,
        mut size: Vec2,
        audio_player: Option<&'a mut AudioPlayer>,
        audio_generator: Option<GeneratorSelector>,
    ) -> Self {
        if orientation == PianoOrientation::Vertical {
            size = size.yx();
        }

        Self {
            max_note,
            min_note,
            size,
            orientation,
            audio_player,
            generator_selector: audio_generator,
        }
    }

    fn make_piano_board(&self) -> Shape {
        Shape::rect_filled(self.rect(), CornerRadius::ZERO, Color32::WHITE)
    }

    fn get_piano_notes(&self) -> Vec<PlacedNote> {
        (self.min_note..=self.max_note)
            .map(|pitch_value| PlacedNote {
                note: Note {
                    pitch_name: pitch_value.into(),
                    beats: 0.0,
                },
                offset: 0.0.into(),
                pitch_offset: 0,
            })
            .collect()
    }

    fn make_all_piano_keys(&self, notes: Vec<PlacedNote>) -> Vec<Shape> {
        notes
            .into_iter()
            .map(|note| self.make_piano_key(note))
            .collect()
    }

    fn make_piano_key(&self, note: PlacedNote) -> Shape {
        // White notes are arranged so that the edge of B and C and the edge of
        // E and F lines align with the edge of the equivalent background.
        // This helps visual align background notes with the piano keys.
        // As a result white notes C, D, and E are slightly larger, spread out
        // over 5 background notes and F, G, A, and B slight smaller spread out
        // over 7.
        let (offset, note_size) = if self.orientation == PianoOrientation::Vertical {
            match note.note.pitch_name.scale_value {
                ScaleValue::C => (-2.0 / 3.0, 5.0 / 3.0),
                ScaleValue::D => (-1.0 / 3.0, 5.0 / 3.0),
                ScaleValue::E => (0.0, 5.0 / 3.0),
                ScaleValue::F => (-3.0 / 4.0, 7.0 / 4.0),
                ScaleValue::G => (-1.0 / 2.0, 7.0 / 4.0),
                ScaleValue::A => (-1.0 / 4.0, 7.0 / 4.0),
                ScaleValue::B => (0.0, 7.0 / 4.0),
                _ => return self.make_black_key(note),
            }
        } else {
            match note.note.pitch_name.scale_value {
                ScaleValue::C => (0.0, 5.0 / 3.0),
                ScaleValue::D => (-1.0 / 3.0, 5.0 / 3.0),
                ScaleValue::E => (-2.0 / 3.0, 5.0 / 3.0),
                ScaleValue::F => (0.0, 7.0 / 4.0),
                ScaleValue::G => (-1.0 / 4.0, 7.0 / 4.0),
                ScaleValue::A => (-1.0 / 2.0, 7.0 / 4.0),
                ScaleValue::B => (-3.0 / 4.0, 7.0 / 4.0),
                _ => return self.make_black_key(note),
            }
        };
        self.make_white_key(note, offset, note_size)
    }

    fn make_white_key(&self, note: PlacedNote, offset: f32, note_size: f32) -> Shape {
        let note_pos = if self.orientation == PianoOrientation::Vertical {
            NoteSequencerObject(note).to_pos(self.range()) + vec2(offset, 0.0)
        } else {
            self.to_horizontal_pos(self.range(), note) + vec2(offset, 0.0)
        };
        let size = vec2(note_size, 1.0);
        let rect = Rect::from_min_size(note_pos, size);

        Shape::rect_stroke(
            self.transpose_if_vertical(rect),
            CornerRadius::ZERO,
            Stroke::new(1.0, Color32::from_black_alpha(64)),
            StrokeKind::Inside,
        )
    }

    fn make_black_key(&self, note: PlacedNote) -> Shape {
        let note_pos = if self.orientation == PianoOrientation::Vertical {
            NoteSequencerObject(note).to_pos(self.range())
        } else {
            self.to_horizontal_pos(self.range(), note)
        };
        let note_size = vec2(BLACK_NOTE_WIDTH, BLACK_NOTE_LENGTH);
        let rect = self.transpose_if_vertical(Rect::from_min_size(note_pos, note_size));

        // Black key shape.
        let corner_radius = self.transpose_if_vertical(CornerRadius {
            nw: 0,
            ne: 0,
            sw: 2,
            se: 2,
        });

        Shape::rect_filled(rect, corner_radius, Color32::BLACK)
    }

    fn range(&self) -> Rect {
        Rect::from_x_y_ranges(self.min_note as f32..=self.max_note as f32, 0.0..=1.0)
    }

    fn rect(&self) -> Rect {
        self.transpose_if_vertical(Rect::from_min_size(Pos2::ZERO, self.range().size()))
    }

    fn transpose_if_vertical<T: Yx>(&self, object: T) -> T {
        match self.orientation {
            PianoOrientation::Horizontal => object,
            PianoOrientation::Vertical => object.yx(),
        }
    }

    fn to_horizontal_pos(&self, range: Rect, note: PlacedNote) -> Pos2 {
        let offset: f32 = note.offset.into();
        let y = offset - range.top();
        let pitch_value: PitchValue = note.note.pitch_name.into();
        let x = pitch_value as f32 - range.left();
        pos2(x, y)
    }

    fn create_click_feedback(&self, shape: &Shape, is_black: bool) -> Shape {
        let corner_radius = self.transpose_if_vertical(CornerRadius {
            nw: 0,
            ne: 0,
            sw: 2,
            se: 2,
        });

        let click_feedback_colour = if is_black {
            Color32::from_gray(100) // slightly lighten black key
        } else {
            Color32::from_rgba_unmultiplied(0, 0, 0, ((30.0 / 100.0) * 255.0) as u8) // slightly darken black key but use transparent true black instead of gray so it doesn't appear on overlapping black keys
        };

        Shape::rect_filled(
            shape.visual_bounding_rect(),
            corner_radius,
            click_feedback_colour,
        )
    }

    fn calculate_clicked_note(
        &self,
        position: Pos2,
        piano_transform: RectTransform,
    ) -> (Option<PlacedNote>, bool) {
        let piano_notes = self.get_piano_notes();
        let mut clicked_note: Option<_> = None;
        let mut is_black = false;
        for note in piano_notes {
            let current_note = note.clone();
            let key = self.make_piano_key(note);
            let transformed_key = key.transform(piano_transform);
            let key_rect = piano_transform
                .inverse()
                .transform_rect(transformed_key.visual_bounding_rect());

            if key_rect.contains(position) {
                clicked_note = Some(current_note);
                let top_left = key_rect.min;
                let top_right = pos2(key_rect.max.x, key_rect.min.y);
                let bottom_left = pos2(key_rect.min.x, key_rect.max.y);
                let bottom_right = key_rect.max;
                // Check if intercepted with black note, if black note is hit then stop iterating.
                match self.orientation {
                    PianoOrientation::Horizontal => {
                        if top_right.x - top_left.x == BLACK_NOTE_WIDTH
                            || bottom_right.y == BLACK_NOTE_LENGTH
                        {
                            is_black = true;
                            break;
                        }
                    }
                    PianoOrientation::Vertical => {
                        if bottom_left.y - top_left.y == BLACK_NOTE_WIDTH
                            || top_right.x == BLACK_NOTE_LENGTH
                        {
                            is_black = true;
                            break;
                        }
                    }
                }
            }
        }
        (clicked_note, is_black)
    }
}

impl View for Piano<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(self.size, Sense::drag());
            let piano_transform = RectTransform::from_to(self.rect(), response.rect);
            let piano_board = self.make_piano_board();
            let piano_notes = self.get_piano_notes();
            let piano_keys = self.make_all_piano_keys(piano_notes);

            // Draw piano board first
            painter.extend(vec![piano_board].transform(piano_transform));

            // Then draw keys on top
            painter.extend(piano_keys.transform(piano_transform));

            let did_interact = response.drag_started()
                || response.drag_stopped()
                || response.is_pointer_button_down_on();
            if !did_interact {
                return;
            }
            let (Some(sel), Some(pointer_pos)) =
                (self.generator_selector, response.interact_pointer_pos())
            else {
                return;
            };
            let local_pos = piano_transform.inverse().transform_pos(pointer_pos);
            let (clicked_note, is_black) = self.calculate_clicked_note(local_pos, piano_transform);
            if let Some(clicked_note) = clicked_note {
                let clicked_note_feedback = self.create_click_feedback(
                    &self
                        .make_piano_key(clicked_note.clone())
                        .transform(piano_transform),
                    is_black,
                );
                painter.add(clicked_note_feedback);

                if let Some(player) = self.audio_player.as_mut() {
                    if response.drag_started() {
                        player.send_note_on(sel, clicked_note.note.pitch_name);
                    } else if response.drag_stopped() {
                        player.send_note_off(sel, clicked_note.note.pitch_name);
                    }
                }
            }
        });

        if cfg!(feature = "extra_debug") {
            egui::Window::new("Piano Debug").show(ui.ctx(), |ui| {
                ui.label(format!("size: {}", self.size));
                ui.label(format!(
                    "min_note: {}, max_note: {}",
                    self.min_note, self.max_note
                ));
                ui.label(format!(
                    "range: {:?}",
                    self.transpose_if_vertical(self.range())
                ));
                ui.label(format!("Total notes: {}", self.max_note - self.min_note));
                let piano_keys = self.make_all_piano_keys(self.get_piano_notes());
                ui.label(format!("Piano keys: {:?}", piano_keys))
            });
        }
    }
}
