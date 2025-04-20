use egui::{emath::RectTransform, epaint::PathStroke, pos2, vec2, Color32, CornerRadius, Frame, Pos2, Rect, Shape, Ui};
use mesic::dft::{dft, get_freq_response};

use crate::{app_state::AudioState, transform::Transform, view::View};

pub struct FrequencyDisplay<'a> {
    audio_state: &'a AudioState,
    bin_count: usize
}

impl<'a> FrequencyDisplay<'a> {
    pub fn new(audio_state: &'a mut AudioState) -> Self {
        FrequencyDisplay {
            audio_state,
            bin_count: 25
        }
    }

    fn response_line(&self, signal: Vec<f32>, bin_count: usize) -> Vec<Pos2> {
        let interval = 0.18;
        (0..bin_count).map(|bin| {
            let response = get_freq_response(signal.clone(), 10f32.powf(bin as f32 * interval));
            pos2(bin as f32, response)
        }).collect() 
    }
}

impl View for FrequencyDisplay<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let audio_state = &mut self.audio_state;
        let bin_count = self.bin_count;
        let audio_len = audio_state.audio.len() as f32;
        let canvas_size = vec2(500.0, 100.0);

        if audio_len == 0.0 {
            return;
        }

        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let (_id, rect) = ui.allocate_space(canvas_size);
            let to_screen =
                RectTransform::from_to(Rect::from_min_max(pos2(0.0, 0.0), pos2(bin_count as f32, 1.0)), rect);

            let audio = audio_state.audio.clone();
            let points = self.response_line(audio, bin_count);

            let shapes: Vec<_> = points
                .iter()
                .enumerate()
                .map(|(_, pos)| {
                    log::info!("{:?}", pos);
                    Shape::rect_filled(Rect::from_min_size(pos2(pos.x, 1.0-pos.y), vec2(1.0, pos.y)), CornerRadius::same(0), Color32::WHITE)
                })
                .collect();
            ui.painter().extend(shapes.transform(to_screen))
        });
    }
}
