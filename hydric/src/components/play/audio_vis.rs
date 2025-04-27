use crate::AudioState;
use egui::{Color32, Rect, Ui, containers::Frame, emath, epaint, epaint::PathStroke, pos2, vec2};

pub fn audio_vis(audio_state: &AudioState, ui: &mut Ui) {
    let audio_len = audio_state.audio.len() as f32;
    let canvas_size = vec2(500.0, 100.0);

    if audio_len == 0.0 {
        return;
    }

    Frame::canvas(ui.style()).show(ui, |ui| {
        ui.ctx().request_repaint();
        let (_id, rect) = ui.allocate_space(canvas_size);
        let to_screen =
            emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 1.0..=-1.0), rect);

        let mut averages: Vec<f32> = vec![0.0; canvas_size.x as usize];
        let chunking = (audio_len / canvas_size.x) as i32;
        // TODO: put this into a generic util.
        for (i, sample) in audio_state.audio.iter().enumerate() {
            // For now, only visualise the left.
            let [left, _right] = sample;

            let index = i / (chunking as usize);
            let value = left.abs() / (chunking as f32);
            if index >= averages.len() {
                // sometimes happens due to rounding of floats,
                // okay to just ignore.
                continue;
            }
            averages[index] += value;
        }

        let points: Vec<_> = averages
            .iter()
            .enumerate()
            .map(|(x, sample)| pos2(x as f32 / canvas_size.x, sample.clamp(-1.0, 1.0)))
            .collect();

        let thickness = 1.0;
        let mut shapes: Vec<_> = points
            .iter()
            .map(|pos| {
                epaint::Shape::line(
                    vec![to_screen * *pos, to_screen * pos2(pos.x, -pos.y)],
                    PathStroke::new(thickness, Color32::WHITE),
                )
            })
            .collect();

        let position = audio_state.player.position.samples;
        let playthrough_ratio = position as f32 / audio_len;

        if (0.0..=1.0).contains(&playthrough_ratio) {
            let red_line = epaint::Shape::line(
                vec![
                    to_screen * pos2(playthrough_ratio, -1.0),
                    to_screen * pos2(playthrough_ratio, 1.0),
                ],
                PathStroke::new(thickness, Color32::RED),
            );
            shapes.push(red_line);
        }
        ui.painter().extend(shapes);
    });
}
