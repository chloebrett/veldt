use crate::{playback::AudioPlayer, view::View};
use egui::{
    Color32, Ui,
    cache::{ComputerMut, FrameCache},
};
use egui_plot::{Line, Plot, PlotPoints};
use mesic::{
    eq::FrequencyResponsePoint, fft::{fft, hann_window}, FFT_SAMPLE_SIZE, SAMPLE_RATE
};
use ordered_float::OrderedFloat;
use ringbuffer::RingBuffer;
use shared::serialize::map_vec;
use std::cmp::max;

pub struct FrequencyDisplay<'a> {
    player: Option<&'a AudioPlayer>,
    frequency_points: Option<Vec<FrequencyResponsePoint>>, // For charting a 'static' frequency response graph 
    y_bounds: [f64; 2], // min y bound, max y bound
}

impl<'a> FrequencyDisplay<'a> {
    pub fn new(player: Option<&'a AudioPlayer>, frequency_points: Option<Vec<FrequencyResponsePoint>>, y_bounds: [f64; 2]) -> Self {
        FrequencyDisplay {
            player,
            frequency_points,
            y_bounds
        }
    }

    /// Create frequency display shapes synced with playing audio.
    fn render_display(&self, ui: &mut Ui, audio: Vec<OrderedFloat<f32>>) -> Option<Vec<f32>> {
        // TODO: account for frame rate.
        let chunk_head = max(0, audio.len() as isize - FFT_SAMPLE_SIZE as isize) as usize;

        // Cast as `OrderedFloat` set-length array so that the value can be cached.
        let slice: [OrderedFloat<f32>; FFT_SAMPLE_SIZE] = audio
            [chunk_head..(chunk_head + FFT_SAMPLE_SIZE)]
            .try_into()
            .unwrap_or([OrderedFloat(0.0); FFT_SAMPLE_SIZE]);

        Some(ui.memory_mut(|memory| {
            let cache = memory.caches.cache::<FrequencyDisplayCache<'_>>();
            cache.get(FrequencyDisplayKey { audio: slice })
        }))
    }
}

fn response_points(signal: Vec<f32>) -> Vec<f32> {
    fft(hann_window(signal))
}

#[derive(Default)]
struct FrequencyDisplayComputer;

#[derive(Hash, Copy, Clone, Debug)]
struct FrequencyDisplayKey {
    audio: [OrderedFloat<f32>; FFT_SAMPLE_SIZE],
}

type FrequencyDisplayCache<'a> = FrameCache<Vec<f32>, FrequencyDisplayComputer>;

impl ComputerMut<FrequencyDisplayKey, Vec<f32>> for FrequencyDisplayComputer {
    fn compute(&mut self, key: FrequencyDisplayKey) -> Vec<f32> {
        response_points(map_vec(key.audio.to_vec()))
    }
}

impl View for FrequencyDisplay<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let audio = if let Some(player) = self.player{
            let stereo_audio = player.recent_buf().iter();
            // Note: only visualising the left channel.
            // TODO: decide how to visualise both left and right.
            stereo_audio.map(|it| it[0]).collect()
        } else {
            Vec::new()
        };

        // Cast as `OrderedFloat` so that values implement `Eq` required for hashing in cache.
        let ordered_audio: Vec<OrderedFloat<f32>> = map_vec(audio.to_vec());
        let mut plot_shapes = vec![];

        let response = if let Some(_player) = &self.player {
            self.render_display(ui, ordered_audio)
        } else {
            None
        };
        
        if let Some(response) = response {
            let freq_window = SAMPLE_RATE as f64 / FFT_SAMPLE_SIZE as f64;
            let points: PlotPoints = response
                .into_iter()
                .enumerate()
                // Only keep first half of results.
                .filter(|(index, _it)| *index < FFT_SAMPLE_SIZE / 2)
                // Take log of values to make dB.
                .map(|(index, it)| [freq_window * index as f64, it.log10() as f64])
                .collect();
            plot_shapes.push(Line::new("Response", points).color(Color32::WHITE))
        } else {
            if let Some(points) = self.frequency_points.clone() {
                let eq_plot_points: PlotPoints = points
                    .into_iter()
                    .map(|point| [point.frequency as f64, point.gain_db as f64])
                    .collect();
                plot_shapes.push(Line::new("Response", eq_plot_points).color(Color32::WHITE));
            }
        };
        // TODO: investigate logarithmic x axis.
        Plot::new("Frequency Response")
            .view_aspect(2.0)
            .default_x_bounds(0.0, SAMPLE_RATE as f64 / 2.0)
            .default_y_bounds(self.y_bounds[0], self.y_bounds[1])
            .allow_drag(false)
            .x_axis_label("Frequency (Hz)")
            .y_axis_label("Response (dB)")
            .show(ui, |plot_ui| {
                for shape in plot_shapes {
                    plot_ui.line(shape)
                }
            });
    }
}
