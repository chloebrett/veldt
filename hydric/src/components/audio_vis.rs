use crate::audio_player::{Handle, play};
use crate::audio_render::render;
use crate::envelope_control;
use crate::note_save::{load_note_list, load_notes, save_notes};
use egui::{
    Color32, Rect, ScrollArea, Ui, containers::Frame, emath, epaint, epaint::PathStroke, pos2,
    scroll_area::ScrollBarVisibility, vec2,
};
use mesic::{SAMPLE_RATE, create_scale_values, create_track, render as local_render};
use poll_promise::Promise;
use shared::model::{
    AdsrEnvelope, DelayConfig, Effect, EffectInstance, EffectMeta, EqConfig, EqType,
    GeneratorInstance, GeneratorMeta, GeneratorType, Note, PitchName, Scale, ScaleValue,
    SimpleWaveConfig, WaveType,
};
use strum::IntoEnumIterator;

pub fn audio_vis(app: &App, ui: &mut Ui) {
    let audio_len = self.audio.len() as f32;
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
        for (i, sample) in self.audio.iter().enumerate() {
            let index = i / (chunking as usize);
            let value = sample.abs() / (chunking as f32);
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
            .map(|(x, sample)| pos2(x as f32 / canvas_size.x, *sample))
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

        if let Some(handle) = &self.handle {
            let current_timestamp = chrono::offset::Utc::now();
            let time_delta_ms: i64 =
                (current_timestamp - handle.start_timestamp).num_milliseconds();
            let audio_duration_ms: f32 = audio_len / (SAMPLE_RATE as f32) * 1000.0;
            let playthrough_ratio: f32 = (time_delta_ms as f32) / audio_duration_ms;

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
        }
        ui.painter().extend(shapes);
    });
}
