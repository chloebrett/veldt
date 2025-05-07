use crate::{
    transform::{Transform, Yx},
    view::View,
    widget::SequencerObject,
};
use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
    emath::RectTransform, vec2, pos2
};
use shared::{
    model::{Note, PlacedNote, ScaleValue, PitchName},
    types::PitchValue,
};
use log::info;

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
    pub fn new(max_note: PitchValue, min_note: PitchValue, orientation: PianoOrientation, mut size: Vec2) -> Self {
        if orientation == PianoOrientation::Vertical {
            size = size.yx();
        }

        Self {
            max_note,
            min_note,
            size,
            orientation,
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
            note.to_pos(self.range()) + vec2(offset, 0.0)
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
        let black_note_length = 0.6;
        let note_pos = if self.orientation == PianoOrientation::Vertical {
            note.to_pos(self.range())
        } else {
            self.to_horizontal_pos(self.range(), note)
        };
        //let note_pos = note.to_pos(self.range());
        let note_size = vec2(1.0, black_note_length);
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
}

impl View for Piano {
    fn ui(&mut self, ui: &mut Ui) {
        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(self.size, Sense::click());
            let piano_transform = RectTransform::from_to(self.rect(), response.rect);
            let piano_board = self.make_piano_board();
            let piano_notes = self.get_piano_notes();
            let piano_keys = self.make_all_piano_keys(piano_notes);

            // Draw piano board first
            painter.extend(vec![piano_board].transform(piano_transform));

            // Then draw keys on top
            painter.extend(piano_keys.transform(piano_transform));

            let calculate_clicked_note = |position: Pos2| {
                let piano_notes = self.get_piano_notes();
                let mut clicked_note: Option<PlacedNote> = None;
                for note in piano_notes {
                    let current_note = note.clone();
                    info!("Transformed note Corners: {:?}", note.note.pitch_name);
                    let key = self.make_piano_key(note);
                    let transformed_key = key.transform(piano_transform).visual_bounding_rect();

                    let top_left = piano_transform.inverse().transform_pos(transformed_key.min);
                    let top_right = piano_transform.inverse().transform_pos(pos2(transformed_key.max.x, transformed_key.min.y));
                    let bottom_left = piano_transform.inverse().transform_pos(pos2(transformed_key.min.x, transformed_key.max.y));
                    let bottom_right = piano_transform.inverse().transform_pos(transformed_key.max);
                    
                    // debugging
                    info!("  Top-Left: {:?}", top_left);
                    info!("  Top-Right: {:?}", top_right);
                    info!("  Bottom-Left: {:?}", bottom_left);
                    info!("  Bottom-Right: {:?}", bottom_right);

                    // this is for horizontal
                    if position.x >= top_left.x && position.x <= top_right.x && position.y >= top_left.y && position.y <= bottom_left.y {
                        clicked_note = Some(current_note);
                        if top_right.x - top_left.x == 1.0 || bottom_right.y == 0.6 {
                            break // stop iterating if you hit a black note
                        }

                    }
                }
                info!("Clicked Note: {:?}", clicked_note.clone().unwrap());
                // let note_shadow = self.make_piano_key(clicked_note.clone().unwrap()).transform(piano_transform);
                // painter.add(self.make_piano_key(clicked_note.clone().unwrap()).transform(piano_transform));
                clicked_note
            };
       
            if response.clicked() {
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let local_pos = piano_transform.inverse().transform_pos(pointer_pos);

                    info!("Clicked at local position: ({:.1}, {:.1})", local_pos.x, local_pos.y);

                    let clicked_note = calculate_clicked_note(local_pos);
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
