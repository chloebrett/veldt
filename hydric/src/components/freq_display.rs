use egui::{cache::{ComputerMut, FrameCache}, emath::RectTransform, epaint::PathStroke, pos2, vec2, Color32, CornerRadius, Frame, Pos2, Rect, Shape, Ui};
use mesic::{dft::{dft, get_freq_response}, render};
use ordered_float::OrderedFloat;
use shared::serialize::map_vec;
use state::Store;
use std::rc::Rc;

use crate::{app_state::AudioState, transform::Transform, view::View};

pub struct FrequencyDisplay {
    audio: Vec<f32>,
    bin_count: usize,
}

impl FrequencyDisplay {
    pub fn new(audio: Vec<f32>) -> Self {
        FrequencyDisplay {
            audio,
            bin_count: 10,
        }
    }
}

fn response_line(signal: Vec<f32>, bin_count: usize) -> Vec<Pos2> {
    let interval = 1.0;
    (1..bin_count+1).map(|bin| {
        let freq = 2f32.powf(bin as f32) * 10.0;
        let response = get_freq_response(signal.clone(), freq);
        pos2(bin as f32 -1.0, response)
    }).collect() 
}

#[derive(Default)]
struct FrequencyDisplayComputer;

#[derive(Hash, Copy, Clone, Debug)]
struct FrequencyDisplayKey {
    audio: [OrderedFloat<f32>; 1024],
    bin_count: usize,
    y_max: OrderedFloat<f32>
}

type FrequencyDisplayCache<'a> = FrameCache<Shape, FrequencyDisplayComputer>;


impl ComputerMut<FrequencyDisplayKey, Shape> for FrequencyDisplayComputer {
    fn compute(&mut self, key: FrequencyDisplayKey) -> Shape {
        let y_max = *key.y_max;
        let points = response_line(map_vec(key.audio.to_vec()), key.bin_count); 
        Shape::Vec(points.iter().map(|pos| {
            let pos =pos.clamp(pos2(0.0, 0.0), pos2(key.bin_count as f32, y_max));
            Shape::rect_filled(Rect::from_min_size(pos2(pos.x, y_max-pos.y), vec2(1.0, pos.y)), CornerRadius::same(0), Color32::WHITE)
        }).collect())
    }
}

impl View for FrequencyDisplay {
    fn ui(&mut self, ui: &mut Ui) {
        let bin_count = self.bin_count;
        let canvas_size = vec2(500.0, 100.0);

        let audio = &self.audio;
        if audio.is_empty() {
            return
        }
        let ordered_audio: Vec<OrderedFloat<f32>> = map_vec(audio.to_vec());
        let slice: [OrderedFloat<f32>; 1024] = ordered_audio[0..1024].try_into().unwrap_or([OrderedFloat(0.0); 1024]);

        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let (_id, rect) = ui.allocate_space(canvas_size);
            let y_max = 0.5;
            let to_screen =
                RectTransform::from_to(Rect::from_min_max(pos2(0.0, 0.0), pos2(bin_count as f32, y_max)), rect);

            let shapes = ui.memory_mut(|memory| {
                let cache = memory.caches.cache::<FrequencyDisplayCache<'_>>();
                cache.get(FrequencyDisplayKey{
                    audio: slice, bin_count, y_max: y_max.into()
                })
            });
            ui.painter().add(shapes.transform(to_screen))
        });
    }
}
