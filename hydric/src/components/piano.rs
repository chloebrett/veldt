use crate::{transform::Transform, view::View, widget::SequencerObject};
use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
    emath::RectTransform, pos2, vec2,
};
use shared::{
    model::{Note, PlacedNote, ScaleValue},
    types::PitchValue,
};

pub enum PianoOrientation {
    Vertical,
    Horizontal,
}

pub struct Piano {
    max_note: PitchValue,
    min_note: PitchValue,
    size: Vec2,
    orientation: PianoOrientation,
}

impl Piano {
    pub fn new(max_note: PitchValue, min_note: PitchValue) -> Self {
        Self {
            max_note,
            min_note,
            size: vec2(50.0, 600.0),
            orientation: PianoOrientation::Vertical,
        }
    }

    pub fn with_orientation(mut self, orientation: PianoOrientation) -> Self {
        // Adjust size based on orientation
        let dims = vec2(600.0, 50.0);
        self.size = match orientation {
            PianoOrientation::Horizontal => dims,
            PianoOrientation::Vertical => dims.yx(),
        };

        self.orientation = orientation;
        self
    }

    fn make_piano_board(&self, range: Rect) -> Shape {
        Shape::rect_filled(
            Rect::from_min_size(Pos2::ZERO, range.size()),
            CornerRadius::ZERO,
            Color32::WHITE,
        )
    }

    fn get_piano_notes(&self, rect: Rect) -> Vec<PlacedNote> {
        let note = |pitch_value: i32| PlacedNote {
            note: Note {
                pitch_name: pitch_value.into(),
                beats: 0.0,
            },
            offset: 0.0.into(),
        };
        match self.orientation {
            PianoOrientation::Horizontal => (rect.left() as i32..=rect.right() as i32)
                .map(note)
                .collect(),
            PianoOrientation::Vertical => (rect.top() as i32..=rect.bottom() as i32)
                .map(note)
                .collect(),
        }
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
        let (offset, note_size) = match note.note.pitch_name.scale_value {
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
        self.make_white_key(note, offset, note_size, range)
    }

    fn make_white_key(&self, note: PlacedNote, offset: f32, note_size: f32, range: Rect) -> Shape {
        let rect = match self.orientation {
            PianoOrientation::Vertical => {
                let note_pos = note.to_pos(range) + vec2(0.0, offset);
                let size = vec2(1.0, note_size);
                Rect::from_min_size(note_pos, size)
            }
            PianoOrientation::Horizontal => {
                let note_pos = note.to_pos_horizontal(range) + vec2(offset, 0.0);
                let size = vec2(note_size, 1.0);
                Rect::from_min_size(note_pos, size)
            }
        };
        Shape::rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, Color32::from_black_alpha(64)),
            StrokeKind::Inside,
        )
    }

    fn make_black_key(&self, note: PlacedNote, range: Rect) -> Shape {
        let black_note_length = 0.6;

        match self.orientation {
            PianoOrientation::Vertical => {
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
            PianoOrientation::Horizontal => {
                let note_pos = note.to_pos_horizontal(range);
                let note_size = vec2(1.0, black_note_length);
                let rect = Rect::from_min_size(note_pos, note_size);

                Shape::rect_filled(
                    rect,
                    // No radius on the top and small radius on the bottom
                    CornerRadius {
                        nw: 0,
                        ne: 0,
                        sw: 2,
                        se: 2,
                    },
                    Color32::BLACK,
                )
            }
        }
    }
}

impl View for Piano {
    fn ui(&mut self, ui: &mut Ui) {
        // Define range based on orientation
        let range = match self.orientation {
            PianoOrientation::Vertical => Rect::from_min_max(
                pos2(0.0, self.min_note as f32),
                pos2(1.0, self.max_note as f32),
            ),
            PianoOrientation::Horizontal => {
                // min is top left corner which is 0,0 and max is bottom right which is 76,1
                Rect::from_min_max(
                    pos2(0.0, 0.0),
                    pos2((self.max_note - self.min_note) as f32, 1.0),
                )
            }
        };

        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(self.size, Sense::hover());
            let piano_transform = RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, range.size()),
                response.rect,
            );
            let piano_board = self.make_piano_board(range);
            let piano_keys = self.make_all_piano_keys(self.get_piano_notes(range), range);

            // Draw piano board first
            painter.extend(vec![piano_board].transform(piano_transform));

            // Then draw keys on top
            painter.extend(piano_keys.transform(piano_transform));
        });

        if cfg!(feature = "extra_debug") {
            egui::Window::new("Piano Debug").show(ui.ctx(), |ui| {
                ui.label(format!(
                    "min_note: {}, max_note: {}",
                    self.min_note, self.max_note
                ));
                ui.label(format!("range: {:?}", range));
                ui.label(format!("Total notes: {}", self.max_note - self.min_note));
                let piano_keys = self.make_all_piano_keys(self.get_piano_notes(range), range);
                ui.label(format!("Piano keys: {:?}", piano_keys))
            });
        }
    }
}
