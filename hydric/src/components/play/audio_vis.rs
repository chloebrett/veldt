use crate::AudioState;
use crate::transform::Transform;
use egui::{
    Color32, Rect, Ui, containers::Frame, emath, epaint, epaint::PathStroke, pos2,
    vec2,
};
use ringbuffer::RingBuffer;

pub fn audio_vis(audio_state: &mut AudioState, sample_count: Option<usize>, ui: &mut Ui) {
    let audio_len = audio_state.player.recent_buf().len();

    let canvas_size = vec2(500.0, 100.0);
    let sample_count = sample_count.unwrap_or(audio_len);

    Frame::canvas(ui.style()).show(ui, |ui| {
        ui.ctx().request_repaint();
        let (_id, rect) = ui.allocate_space(canvas_size);
        let to_screen =
            emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 1.0..=-1.0), rect);

        let buf: Vec<_> = audio_state
            .player
            .recent_buf()
            .iter()
            .skip(audio_len - sample_count)
            .collect();

        let points: Vec<_> = if canvas_size.x > sample_count as f32 {
            (0..sample_count)
                .map(|i| {
                    let x = i as f32 / (sample_count - 1) as f32;

                    let y = *buf.get(i).unwrap_or(&&[0.0; 2]);
                    let y = (y[0] + y[1]) * 0.5;

                    pos2(x, y)
                })
                .collect()
        } else {
            // Additional samples rendered at the cost of more processing power.
            let fidelity = 2.0;

            // Render every N=skip samples.
            let skip = (sample_count as f32 / canvas_size.x / fidelity) as usize;

            // For more consistent drawing.
            let offset = audio_state.player.recent_buf_offset() % skip;

            (0..(canvas_size.x * fidelity) as usize)
                .map(|i| {
                    let x = i as f32 / (canvas_size.x - 1.0) / fidelity;

                    let y = *buf.get(i * skip - offset).unwrap_or(&&[0.0; 2]);
                    let y = (y[0] + y[1]) * 0.5;
                    pos2(x, y)
                })
                .collect()
        };

        let thickness = 1.0;
        let line = epaint::Shape::line(
            points.transform(to_screen),
            PathStroke::new(thickness, Color32::WHITE),
        );

        ui.painter().add(line);
    });
}
