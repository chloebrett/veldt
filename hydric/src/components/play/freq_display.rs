use chrono::TimeDelta;
use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Shape, Ui,
    cache::{ComputerMut, FrameCache},
    emath::RectTransform,
    pos2, vec2,
};
use mesic::{
    SAMPLE_RATE,
    dft::{self, dft, hann_window},
};
use ordered_float::OrderedFloat;
use shared::serialize::map_vec;
use std::ops::Sub;

use crate::{app_state::AudioState, transform::Transform, view::View};

const SLICE_LENGTH: usize = 512;

pub struct FrequencyDisplay<'a> {
    audio_state: &'a AudioState,
    bin_count: usize, // The number of bins the response will be grouped into.
    frame_rate: i32,  // The number of times per second the visualisation will be rendered.
    y_max: f32,       // The maximum response value that will be displayed.
}

impl<'a> FrequencyDisplay<'a> {
    pub fn new(audio_state: &'a AudioState) -> Self {
        FrequencyDisplay {
            audio_state,
            // Currently hard-coded to fit window length of DFT.
            bin_count: 8,
            frame_rate: 10,
            y_max: 0.5,
        }
    }
}

fn response_points(signal: Vec<f32>, bin_count: usize) -> Vec<Pos2> {
    let response = dft(hann_window(signal));
    dft::make_log_buckets(response, bin_count)
        .into_iter()
        .enumerate()
        .map(|(index, bucket)| pos2(index as f32, bucket))
        .collect()
}

fn make_frequency_shape(points: Vec<Pos2>, y_max: f32, bin_count: usize) -> Shape {
    Shape::Vec(
        points
            .iter()
            .map(|pos| {
                // Show frequencies whose response is clipped.
                let colour = if pos.y > y_max {
                    Color32::LIGHT_RED
                } else {
                    Color32::WHITE
                };
                // Clip response.
                let clamped_pos = pos.clamp(pos2(0.0, 0.0), pos2(bin_count as f32, y_max));
                Shape::rect_filled(
                    Rect::from_min_size(
                        // Pos y value is top == 0.0
                        // Therefore subtract y value from max y value to render correctly
                        pos2(pos.x, y_max - clamped_pos.y),
                        vec2(1.0, clamped_pos.y),
                    ),
                    CornerRadius::same(0),
                    colour,
                )
            })
            .collect(),
    )
}

#[derive(Default)]
struct FrequencyDisplayComputer;

#[derive(Hash, Copy, Clone, Debug)]
struct FrequencyDisplayKey {
    audio: [OrderedFloat<f32>; SLICE_LENGTH],
    bin_count: usize,
    y_max: OrderedFloat<f32>,
}

type FrequencyDisplayCache<'a> = FrameCache<Shape, FrequencyDisplayComputer>;

impl ComputerMut<FrequencyDisplayKey, Shape> for FrequencyDisplayComputer {
    fn compute(&mut self, key: FrequencyDisplayKey) -> Shape {
        make_frequency_shape(
            response_points(map_vec(key.audio.to_vec()), key.bin_count),
            *key.y_max,
            key.bin_count,
        )
    }
}

impl View for FrequencyDisplay<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let FrequencyDisplay {
            audio_state,
            bin_count,
            frame_rate,
            y_max,
        } = *self;
        let audio = &self.audio_state.audio;
        if audio.is_empty() {
            return;
        }
        // Cast as `OrderedFloat` so that values implement `Eq` required for hashing in cache.
        let ordered_audio: Vec<OrderedFloat<f32>> = map_vec(audio.to_vec());
        let canvas_size = vec2(500.0, 100.0);
        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let (_id, rect) = ui.allocate_space(canvas_size);
            let to_screen = RectTransform::from_to(
                Rect::from_min_max(pos2(0.0, 0.0), pos2(bin_count as f32, y_max)),
                rect,
            );
            if let Some(handle) = &audio_state.handle {
                let current_timestamp = chrono::offset::Utc::now();
                let time_delta: TimeDelta = current_timestamp.sub(handle.start_timestamp);
                let time_delta_ms: i64 = time_delta.num_milliseconds();
                let current_sample: usize = (time_delta_ms * (SAMPLE_RATE as i64) / 1000) as usize;
                let frame_size = (SAMPLE_RATE / frame_rate) as usize;
                // Round `current_sample` so that the audio will be broken up into chunks based on
                // the visualisation frame rate.
                let chunk_head = current_sample / frame_size * frame_size;
                if chunk_head + SLICE_LENGTH < ordered_audio.len() {
                    // Cast as `OrderedFloat` set-length array so that the value can be cached.
                    let slice: [OrderedFloat<f32>; SLICE_LENGTH] = ordered_audio
                        [chunk_head..(chunk_head + SLICE_LENGTH)]
                        .try_into()
                        .unwrap_or([OrderedFloat(0.0); SLICE_LENGTH]);

                    let shapes = ui.memory_mut(|memory| {
                        let cache = memory.caches.cache::<FrequencyDisplayCache<'_>>();
                        cache.get(FrequencyDisplayKey {
                            audio: slice,
                            bin_count,
                            y_max: y_max.into(),
                        })
                    });
                    ui.painter().add(shapes.transform(to_screen));
                }
            }
        });
    }
}
