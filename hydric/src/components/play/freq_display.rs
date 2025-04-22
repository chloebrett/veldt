use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Shape, Ui,
    cache::{ComputerMut, FrameCache},
    emath::RectTransform,
    pos2, vec2,
};
use mesic::dft::{self, dft, hann_window};
use ordered_float::OrderedFloat;
use shared::serialize::map_vec;

use crate::{transform::Transform, view::View};

const FRAME_SIZE: usize = 1024;

pub struct FrequencyDisplay {
    audio: Vec<f32>,
    bin_count: usize,
}

impl FrequencyDisplay {
    pub fn new(audio: Vec<f32>) -> Self {
        FrequencyDisplay {
            audio,
            // Currently hard-coded to fit window length of DFT.
            bin_count: 8,
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

#[derive(Default)]
struct FrequencyDisplayComputer;

#[derive(Hash, Copy, Clone, Debug)]
struct FrequencyDisplayKey {
    audio: [OrderedFloat<f32>; FRAME_SIZE],
    bin_count: usize,
    y_max: OrderedFloat<f32>,
}

type FrequencyDisplayCache<'a> = FrameCache<Shape, FrequencyDisplayComputer>;

impl ComputerMut<FrequencyDisplayKey, Shape> for FrequencyDisplayComputer {
    fn compute(&mut self, key: FrequencyDisplayKey) -> Shape {
        let y_max = *key.y_max;
        let points = response_points(map_vec(key.audio.to_vec()), key.bin_count);
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
                    let clamped_pos = pos.clamp(pos2(0.0, 0.0), pos2(key.bin_count as f32, y_max));
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
}

impl View for FrequencyDisplay {
    fn ui(&mut self, ui: &mut Ui) {
        let bin_count = self.bin_count;
        let audio = &self.audio;
        let canvas_size = vec2(500.0, 100.0);
        if audio.is_empty() {
            return;
        }
        let ordered_audio: Vec<OrderedFloat<f32>> = map_vec(audio.to_vec());
        // Create hashable slice for Cache
        // TODO either improve this so it can take windows of any size
        // Or render as audio is played to avoid any caching.
        let slice: [OrderedFloat<f32>; FRAME_SIZE] = ordered_audio[0..FRAME_SIZE]
            .try_into()
            .unwrap_or([OrderedFloat(0.0); FRAME_SIZE]);

        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let (_id, rect) = ui.allocate_space(canvas_size);
            // Set a maximum response value.
            let y_max = 1.0;
            let to_screen = RectTransform::from_to(
                Rect::from_min_max(pos2(0.0, 0.0), pos2(bin_count as f32, y_max)),
                rect,
            );

            let shapes = ui.memory_mut(|memory| {
                let cache = memory.caches.cache::<FrequencyDisplayCache<'_>>();
                cache.get(FrequencyDisplayKey {
                    audio: slice,
                    bin_count,
                    y_max: y_max.into(),
                })
            });
            ui.painter().add(shapes.transform(to_screen))
        });
    }
}
