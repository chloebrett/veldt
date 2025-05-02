use crate::{transform::Transform, view::View, widget::SequencerObject};
use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
    emath::RectTransform, vec2,
};
use shared::{
    model::{Note, PlacedNote, ScaleValue},
    types::PitchValue,
};

#[derive(PartialEq)]
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

    fn make_piano_board(&self) -> Shape {
        Shape::rect_filled(self.from_rect(), CornerRadius::ZERO, Color32::WHITE)
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
        let (offset, note_size) = match note.note.pitch_name.scale_value {
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
        self.make_white_key(note, offset, note_size)
    }

    fn make_white_key(&self, note: PlacedNote, offset: f32, note_size: f32) -> Shape {
        // TODO: this whole expression just produces the same rect but inverted, if given the same
        // range each time!
        let rect = match self.orientation {
            PianoOrientation::Vertical => {
                let note_pos = note.to_pos(self.range()) + vec2(0.0, offset);
                let size = vec2(1.0, note_size);
                Rect::from_min_size(note_pos, size)
            }
            PianoOrientation::Horizontal => {
                // TODO: get rid of to_pos_horizontal, and just pass a vertical range here?
                let note_pos = note.to_pos_horizontal(self.range()) + vec2(offset, 0.0);
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

    fn make_black_key(&self, note: PlacedNote) -> Shape {
        let black_note_length = 0.6;

        let (note_pos, note_size, corner_radius) = match self.orientation {
            PianoOrientation::Vertical => {
                let note_pos = note.to_pos(self.range());
                let note_size = vec2(black_note_length, 1.0);
                // No radius on the left and small radius on the right.
                // To reflect the actual shape of black keys on a piano.
                let corner_radius = CornerRadius {
                    nw: 0,
                    ne: 2,
                    sw: 0,
                    se: 2,
                };
                (note_pos, note_size, corner_radius)
            }
            PianoOrientation::Horizontal => {
                let note_pos = note.to_pos_horizontal(self.range());
                let note_size = vec2(1.0, black_note_length);
                // No radius on the top and small radius on the bottom
                let corner_radius = CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: 2,
                    se: 2,
                };
                (note_pos, note_size, corner_radius)
            }
        };

        Shape::rect_filled(
            Rect::from_min_size(note_pos, note_size),
            corner_radius,
            Color32::BLACK,
        )
    }

    fn range(&self) -> Rect {
        let mut range =
            Rect::from_x_y_ranges(self.min_note as f32..=self.max_note as f32, 0.0..=1.0);
        if self.orientation == PianoOrientation::Vertical {
            range = range.yx();
        }
        range
    }

    fn from_rect(&self) -> Rect {
        Rect::from_min_size(Pos2::ZERO, self.range().size())
    }
}

impl View for Piano {
    fn ui(&mut self, ui: &mut Ui) {
        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(self.size, Sense::hover());
            let piano_transform = RectTransform::from_to(self.from_rect(), response.rect);
            let piano_board = self.make_piano_board();
            let piano_keys = self.make_all_piano_keys(self.get_piano_notes());

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
                ui.label(format!("range: {:?}", self.range()));
                ui.label(format!("Total notes: {}", self.max_note - self.min_note));
                let piano_keys = self.make_all_piano_keys(self.get_piano_notes());
                ui.label(format!("Piano keys: {:?}", piano_keys))
            });
        }
    }
}

/// Transposes a 2D object, replacing x values with y values and vice versa.
trait Yx {
    fn yx(&self) -> Self;
}

impl Yx for Pos2 {
    fn yx(&self) -> Self {
        // This function already exists for Vec2.
        self.to_vec2().yx().to_pos2()
    }
}

impl Yx for Rect {
    fn yx(&self) -> Self {
        Rect::from_min_max(self.min.yx(), self.max.yx())
    }
}
