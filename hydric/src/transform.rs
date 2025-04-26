use egui::{Pos2, Rect, Shape, emath::RectTransform, epaint::PathShape, epaint::RectShape};

pub trait Transform<T> {
    fn transform(&self, rect: RectTransform) -> T;
}

impl Transform<Shape> for Shape {
    fn transform(&self, rect: RectTransform) -> Shape {
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
            _ => panic!("Shape not implemented."),
        }
    }
}

impl Transform<Vec<Shape>> for Vec<Shape> {
    fn transform(&self, rect: RectTransform) -> Vec<Shape> {
        self.iter()
            .map(|shape| shape.clone().transform(rect))
            .collect()
    }
}

impl Transform<[Pos2; 2]> for [Pos2; 2] {
    fn transform(&self, rect: RectTransform) -> [Pos2; 2] {
        [rect * self[0], rect * self[1]]
    }
}

impl Transform<Rect> for Rect {
    fn transform(&self, rect: RectTransform) -> Rect {
        rect.transform_rect(*self)
    }
}

impl Transform<Pos2> for Pos2 {
    fn transform(&self, rect: RectTransform) -> Pos2 {
        rect * *self
    }
}

impl Transform<Vec<Pos2>> for Vec<Pos2> {
    fn transform(&self, rect: RectTransform) -> Vec<Pos2> {
        self.into_iter().map(|it| it.transform(rect)).collect()
    }
}
