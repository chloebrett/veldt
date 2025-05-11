use crate::AudioState;
use crate::transform::Transform;
use dasp_frame::Stereo;
use egui::{
    Color32, Rect, Sense, Ui, Vec2, containers::Frame, emath, epaint, epaint::PathStroke, pos2,
    vec2,
};
use ringbuffer::RingBuffer;

const PREFER_PRE_RENDERED: bool = false;

pub fn audio_vis(audio_state: &mut AudioState, ui: &mut Ui) {
    let has_audio = !audio_state.audio.is_empty();
    let audio_len = if has_audio && PREFER_PRE_RENDERED {
        audio_state.audio.len()
    } else {
        audio_state.player.recent_buf().len()
    } as f32;

    let canvas_size = vec2(500.0, 100.0);

    if audio_len == 0.0 {
        return;
    }

    Frame::canvas(ui.style()).show(ui, |ui| {
        ui.ctx().request_repaint();
        let (id, rect) = ui.allocate_space(canvas_size);
        let to_screen =
            emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 1.0..=-1.0), rect);

        // Seek on click.
        let response = ui.interact(rect, id, Sense::click());
        if response.clicked() {
            if let Some(point) = response.interact_pointer_pos {
                let point = point.transform(to_screen.inverse());
                let sample = point.x * audio_len;
                audio_state.player.seek(sample as usize);
            }
        }

        let averages = if has_audio && PREFER_PRE_RENDERED {
            calc_averages(canvas_size.x, audio_len, audio_state.audio.iter())
        } else {
            calc_averages(
                canvas_size.x,
                audio_len,
                audio_state.player.recent_buf().iter(),
            )
        };

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
                    vec![*pos, pos2(pos.x, -pos.y)].transform(to_screen),
                    PathStroke::new(thickness, Color32::WHITE),
                )
            })
            .collect();

        if has_audio && PREFER_PRE_RENDERED {
            let position = audio_state.player.effective_pos();
            let playthrough_ratio = position as f32 / audio_len;

            if (0.0..=1.0).contains(&playthrough_ratio) {
                let red_line = epaint::Shape::line(
                    vec![pos2(playthrough_ratio, -1.0), pos2(playthrough_ratio, 1.0)]
                        .transform(to_screen),
                    PathStroke::new(thickness, Color32::RED),
                );
                shapes.push(red_line);
            }
        }
        ui.painter().extend(shapes);
    });
}

fn calc_averages<'a>(
    canvas_width: f32,
    audio_len: f32,
    input: impl Iterator<Item = &'a Stereo<f32>>,
) -> Vec<f32> {
    let mut averages: Vec<f32> = vec![0.0; canvas_width as usize];
    let chunking = (audio_len / canvas_width) as i32;

    for (i, sample) in input.enumerate() {
        // We visualise the average of the left and right channels.
        let [left, right] = sample;

        let index = i / (chunking as usize);
        let value = (left.abs() + right.abs()) * 0.5 / (chunking as f32);
        if index >= averages.len() {
            // sometimes happens due to rounding of floats,
            // okay to just ignore.
            continue;
        }
        averages[index] += value;
    }
    averages
}
