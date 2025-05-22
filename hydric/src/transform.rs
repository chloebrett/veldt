use egui::{
    CornerRadius, Pos2, Rect, Shape, Vec2,
    emath::RectTransform,
    epaint::{PathShape, RectShape, TextShape},
};

pub trait Transform<T> {
    fn transform(&self, rect: RectTransform) -> T;
}

impl Transform<Self> for Shape {
    fn transform(&self, rect: RectTransform) -> Self {
        match self {
            // Note: currently ignores `closed` and `fill`.
            // Add these if needed.
            Shape::Path(PathShape { points, stroke, .. }) => {
                Shape::line(points.transform(rect), stroke.clone())
            }
            Shape::LineSegment { points, stroke } => Shape::LineSegment {
                points: [rect * points[0], rect * points[1]],
                stroke: *stroke,
            },
            Shape::Rect(rect_shape) => Shape::Rect(RectShape {
                rect: rect.transform_rect(rect_shape.rect),
                ..rect_shape.clone()
            }),
            Shape::Vec(shapes) => {
                Shape::Vec(shapes.iter().map(|shape| shape.transform(rect)).collect())
            }
            Shape::Text(text_shape) => Shape::Text(TextShape {
                pos: text_shape.pos.transform(rect),
                ..text_shape.clone()
            }),
            _ => panic!("Shape not implemented."),
        }
    }
}

impl Transform<Self> for Vec<Shape> {
    fn transform(&self, rect: RectTransform) -> Self {
        self.iter()
            .map(|shape| shape.clone().transform(rect))
            .collect()
    }
}

impl Transform<Self> for [Pos2; 2] {
    fn transform(&self, rect: RectTransform) -> Self {
        [rect * self[0], rect * self[1]]
    }
}

impl Transform<Self> for Rect {
    fn transform(&self, rect: RectTransform) -> Self {
        rect.transform_rect(*self)
    }
}

impl Transform<Self> for Pos2 {
    fn transform(&self, rect: RectTransform) -> Self {
        rect * *self
    }
}

impl Transform<Self> for Vec<Pos2> {
    fn transform(&self, rect: RectTransform) -> Self {
        self.iter().map(|it| it.transform(rect)).collect()
    }
}

impl Transform<Self> for Vec2 {
    fn transform(&self, rect: RectTransform) -> Self {
        (rect * self.to_pos2()).to_vec2()
    }
}

/// Transposes a 2D object, replacing x values with y values and vice versa.
pub trait Yx {
    fn yx(&self) -> Self;
}

// Implementing so that we can pass a Vec2 to functions that expect an `impl Yx`.
impl Yx for Vec2 {
    fn yx(&self) -> Self {
        // This function already exists for Vec2.
        Vec2::yx(*self)
    }
}

impl Yx for Pos2 {
    fn yx(&self) -> Self {
        self.to_vec2().yx().to_pos2()
    }
}

impl Yx for Rect {
    fn yx(&self) -> Self {
        Rect::from_min_max(self.min.yx(), self.max.yx())
    }
}

impl Yx for CornerRadius {
    fn yx(&self) -> Self {
        CornerRadius {
            ne: self.sw,
            sw: self.ne,
            ..*self
        }
    }
}
