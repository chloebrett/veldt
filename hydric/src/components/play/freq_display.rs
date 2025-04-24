use chrono::TimeDelta;
use egui::{
    Ui,
    cache::{ComputerMut, FrameCache},
};
use egui_plot::{Line, Plot, PlotPoints};
use mesic::{
    SAMPLE_RATE,
    dft::{dft, hann_window},
};
use ordered_float::OrderedFloat;
use shared::serialize::map_vec;
use std::ops::Sub;

use crate::{app_state::AudioState, view::View};

const SLICE_LENGTH: usize = 512;

pub struct FrequencyDisplay<'a> {
    audio_state: &'a AudioState,
    frame_rate: i32, // The number of times per second the visualisation will be rendered.
}

impl<'a> FrequencyDisplay<'a> {
    pub fn new(audio_state: &'a AudioState) -> Self {
        FrequencyDisplay {
            audio_state,
            // Currently hard-coded to fit window length of DFT.
            frame_rate: 10,
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
    audio: [OrderedFloat<f32>; SLICE_LENGTH],
}

type FrequencyDisplayCache<'a> = FrameCache<Vec<f32>, FrequencyDisplayComputer>;

impl ComputerMut<FrequencyDisplayKey, Vec<f32>> for FrequencyDisplayComputer {
    fn compute(&mut self, key: FrequencyDisplayKey) -> Vec<f32> {
        resopnse_points(map_vec(key.audio.to_vec()))
    }
}

impl View for FrequencyDisplay<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let FrequencyDisplay {
            audio_state,
            frame_rate,
        } = *self;
        let audio = &self.audio_state.audio;
        if audio.is_empty() {
            return;
        }
        // Cast as `OrderedFloat` so that values implement `Eq` required for hashing in cache.
        let ordered_audio: Vec<OrderedFloat<f32>> = map_vec(audio.to_vec());
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

                let response = ui.memory_mut(|memory| {
                    let cache = memory.caches.cache::<FrequencyDisplayCache<'_>>();
                    cache.get(FrequencyDisplayKey { audio: slice })
                });
                let freq_window = SAMPLE_RATE as f64 / SLICE_LENGTH as f64;
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
    }
}
