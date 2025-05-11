use crate::AudioState;
use crate::transform::Transform;
use dasp_frame::Stereo;
use egui::{
    Color32, Rect, Sense, Ui, Vec2, containers::Frame, emath, epaint, epaint::PathStroke, pos2,
    vec2,
};
use ringbuffer::RingBuffer;
use std::cmp::min;

pub fn audio_vis(audio_state: &mut AudioState, sample_count: Option<usize>, ui: &mut Ui) {
    let audio_len = audio_state.player.recent_buf().len();
    let canvas_size = vec2(500.0, 100.0);
    let sample_count = sample_count.unwrap_or(audio_len);

    Frame::canvas(ui.style()).show(ui, |ui| {
        ui.ctx().request_repaint();
        let (id, rect) = ui.allocate_space(canvas_size);
        let to_screen = emath::RectTransform::from_to(
            Rect::from_x_y_ranges(0.0..=canvas_size.x, 1.0..=-1.0),
            rect,
        );

        let buf: Vec<_> = audio_state
            .player
            .recent_buf()
            .iter()
            .skip(audio_len - sample_count)
            .collect();

        let mut factor = sample_count as f32 / canvas_size.x;
        if factor > 1.0 {
            factor = 1.0;
        }
        let points: Vec<_> = (0..(canvas_size.x * factor) as usize)
            .map(|x| {
                let i = (x as f32 / canvas_size.x / factor * sample_count as f32) as usize;
                let y = *buf.get(i).unwrap_or(&&[0.0; 2]);
                let y = (y[0] + y[1]) * 0.5;
                pos2(x as f32 / factor, y)
            })
            .collect();

        let thickness = 1.0;
        let line = epaint::Shape::line(
            points.transform(to_screen),
            PathStroke::new(thickness, Color32::WHITE),
        );

        ui.painter().add(line);
    });
}
