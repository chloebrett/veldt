use egui::{Pos2, Rect, Shape, emath::RectTransform, epaint::PathShape, epaint::RectShape};

pub trait Transform<T> {
    fn transform(self, rect: RectTransform) -> T;
}

impl Transform<Shape> for Shape {
    fn transform(self, rect: RectTransform) -> Shape {
        match self {
            // TODO: handle `closed` and `fill`
            Shape::Path(PathShape { points, stroke, .. }) => {
                Shape::line(points.into_iter().map(|it| rect * it).collect(), stroke)
            }
            Shape::LineSegment { points, stroke } => Shape::LineSegment {
                points: [rect * points[0], rect * points[1]],
                stroke,
            },
            Shape::Rect(rect_shape) => Shape::Rect(RectShape {
                rect: rect.transform_rect(rect_shape.rect),
                ..rect_shape
            }),
            Shape::Vec(shapes) => Shape::Vec(
                shapes
                    .into_iter()
                    .map(|shape| shape.transform(rect))
                    .collect(),
            ),
            _ => panic!("Shape not implemented."),
        }
    }
}

impl Transform<Vec<Shape>> for Vec<Shape> {
    fn transform(self, rect: RectTransform) -> Vec<Shape> {
        self.iter()
            .map(|shape| shape.clone().transform(rect))
            .collect()
    }
}

impl Transform<[Pos2; 2]> for [Pos2; 2] {
    fn transform(self, rect: RectTransform) -> [Pos2; 2] {
        [rect * self[0], rect * self[1]]
    }
}

impl Transform<Rect> for Rect {
    fn transform(self, rect: RectTransform) -> Rect {
        rect.transform_rect(self)
    }
}

impl Transform<Pos2> for Pos2 {
    fn transform(self, rect: RectTransform) -> Pos2 {
        rect * self
    }
}
