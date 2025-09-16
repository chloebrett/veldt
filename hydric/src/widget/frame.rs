use egui::{Color32, Frame, Stroke};

pub fn outer_frame() -> Frame {
    Frame::new()
        .fill(Color32::from_gray(50))
        .stroke(Stroke::new(1.0, Color32::from_gray(60)))
        .corner_radius(8.0)
        .inner_margin(6.0)
}

pub fn inner_frame() -> Frame {
    Frame::new()
        .fill(Color32::from_gray(30))
        .corner_radius(8.0)
        .inner_margin(10.0)
}

pub fn inner_frame_dark() -> Frame {
    Frame::new()
        .fill(Color32::from_gray(20))
        .corner_radius(5.0)
        .inner_margin(10.0)
}
