use egui::{Color32, CornerRadius, Pos2, Rect, Shape, Stroke, StrokeKind};

pub enum NoteRollShape {
    InteractiveNote { note_rect: Rect },
    BackgroundNote { note_rect: Rect },
    WhiteKey { note_rect: Rect },
    BlackKey { note_rect: Rect },
    PianoBoard { rect: Rect },
    BarLine { line: [Pos2; 2], order: u32 }, //Significance of line starting at 0.
}

impl NoteRollShape {
    pub fn make_shape(self) -> Shape {
        match self {
            Self::InteractiveNote { note_rect } => {
                Shape::rect_filled(note_rect, CornerRadius::same(1), Color32::WHITE)
            }
            Self::BackgroundNote { note_rect } => Shape::rect_filled(
                note_rect,
                CornerRadius::same(0),
                Color32::from_white_alpha(2),
            ),
            Self::WhiteKey { note_rect } => Shape::rect_stroke(
                note_rect,
                CornerRadius::same(0),
                Stroke::new(0.5, Color32::from_black_alpha(128)),
                StrokeKind::Inside,
            ),
            Self::BlackKey { note_rect } => {
                let corner_radius = 2;
                Shape::rect_filled(
                    note_rect,
                    CornerRadius {
                        nw: 0,
                        ne: corner_radius,
                        sw: 0,
                        se: corner_radius,
                    },
                    Color32::BLACK,
                )
            }
            Self::PianoBoard { rect } => {
                Shape::rect_filled(rect, CornerRadius::same(0), Color32::WHITE)
            }
            NoteRollShape::BarLine { line, order } => Shape::line_segment(
                line,
                Stroke {
                    width: 1.0,
                    color: Color32::from_white_alpha(2.0f32.powf((3 - order) as f32) as u8),
                },
            ),
        }
    }
}
