use egui::{
    Color32, CornerRadius, Frame, Pos2, Rect, Shape, Ui,
    cache::{ComputerMut, FrameCache},
    emath::RectTransform,
    pos2, vec2,
};
use egui_plot::{Line, Plot, PlotPoints};
use mesic::{dft::{self, dft, hann_window}, SAMPLE_RATE};
use ordered_float::OrderedFloat;
use shared::serialize::map_vec;

use crate::{transform::Transform, view::View};

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

fn resopnse_points(signal: Vec<f32>) -> Vec<f32> {
    dft(hann_window(signal))
}

#[derive(Default)]
struct FrequencyDisplayComputer;

#[derive(Hash, Copy, Clone, Debug)]
struct FrequencyDisplayKey {
    audio: [OrderedFloat<f32>; 1024],
}

type FrequencyDisplayCache<'a> = FrameCache<Vec<f32>, FrequencyDisplayComputer>;

impl ComputerMut<FrequencyDisplayKey, Vec<f32>> for FrequencyDisplayComputer {
    fn compute(&mut self, key: FrequencyDisplayKey) -> Vec<f32> {
        resopnse_points(map_vec(key.audio.to_vec()))
    }
}

impl View for FrequencyDisplay {
    fn ui(&mut self, ui: &mut Ui) {
        let bin_count = self.bin_count;
        let audio = &self.audio;
        if audio.is_empty() {
            return;
        }
        let ordered_audio: Vec<OrderedFloat<f32>> = map_vec(audio.to_vec());
        // Create hashable slice for Cache
        // TODO either improve this so it can take windows of any size
        // Or render as audio is played to avoid any caching.
        let slice: [OrderedFloat<f32>; 1024] = ordered_audio[0..1024]
            .try_into()
            .unwrap_or([OrderedFloat(0.0); 1024]);

        let response = ui.memory_mut(|memory| {
            let cache = memory.caches.cache::<FrequencyDisplayCache<'_>>();
            cache.get(FrequencyDisplayKey { audio: slice })
        });
        let freq_window = SAMPLE_RATE as f64 / 1024 as f64;
        let points: PlotPoints = response
            .iter()
            .enumerate()
            .filter(|(index, _)| *index < response.len() / 2)
            .map(|(index, value)| [freq_window * index as f64, value.log10() as f64])
            .collect();
        let line = Line::new("Response", points);
        Plot::new("Frequency Response")
            .view_aspect(2.0)
            .show(ui, |plot_ui| plot_ui.line(line));
    }
}
