use chrono::TimeDelta;
use egui::{
    Color32, Ui,
    cache::{ComputerMut, FrameCache},
};
use egui_plot::{Line, Plot, PlotPoints};
use mesic::{
    FFT_SAMPLE_SIZE, SAMPLE_RATE,
    fft::{fft, hann_window},
};
use ordered_float::OrderedFloat;
use shared::serialize::map_vec;
use std::ops::Sub;

use crate::{app_state::AudioState, audio_player::AudioPlayer, view::View};

pub struct FrequencyDisplay<'a> {
    audio_state: &'a AudioState,
    frame_rate: i32, // The number of times per second the visualisation will be rendered.
}

impl<'a> FrequencyDisplay<'a> {
    pub fn new(audio_state: &'a AudioState) -> Self {
        FrequencyDisplay {
            audio_state,
            frame_rate: 60,
        }
    }

    /// Create frequency display shapes synced with playing audio.
    fn render_display(
        &self,
        ui: &mut Ui,
        player: &AudioPlayer,
        audio: Vec<OrderedFloat<f32>>,
    ) -> Option<Vec<f32>> {
        let start_timestamp = player.start_timestamp?;
        let current_timestamp = chrono::offset::Utc::now();
        let time_delta: TimeDelta = current_timestamp.sub(start_timestamp);
        let time_delta_ms: i64 = time_delta.num_milliseconds();
        let current_sample: usize = (time_delta_ms * (SAMPLE_RATE as i64) / 1000) as usize;
        let frame_size = (SAMPLE_RATE / self.frame_rate) as usize;
        // Round `current_sample` so that the audio will be broken up into chunks based on
        // the visualisation frame rate.
        let chunk_head = current_sample / frame_size * frame_size;
        if chunk_head + FFT_SAMPLE_SIZE < audio.len() {
            // Cast as `OrderedFloat` set-length array so that the value can be cached.
            let slice: [OrderedFloat<f32>; FFT_SAMPLE_SIZE] = audio
                [chunk_head..(chunk_head + FFT_SAMPLE_SIZE)]
                .try_into()
                .unwrap_or([OrderedFloat(0.0); FFT_SAMPLE_SIZE]);

            Some(ui.memory_mut(|memory| {
                let cache = memory.caches.cache::<FrequencyDisplayCache<'_>>();
                cache.get(FrequencyDisplayKey { audio: slice })
            }))
        } else {
            None
        }
    }
}

fn resopnse_points(signal: Vec<f32>) -> Vec<f32> {
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
        resopnse_points(map_vec(key.audio.to_vec()))
    }
}

impl View for FrequencyDisplay<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let FrequencyDisplay {
            audio_state,
            frame_rate: _frame_rate,
        } = *self;
        let audio = &self.audio_state.audio;
        if audio.is_empty() {
            return;
        }

        // Note: only visualising the left channel.
        // TODO: decide how to visualise both left and right.
        let audio = audio.iter().map(|it| it[0]).collect();

        // Cast as `OrderedFloat` so that values implement `Eq` required for hashing in cache.
        let ordered_audio: Vec<OrderedFloat<f32>> = map_vec(audio);
        if let Some(player) = &audio_state.player {
            if let Some(response) = self.render_display(ui, player, ordered_audio) {
                let freq_window = SAMPLE_RATE as f64 / FFT_SAMPLE_SIZE as f64;
                let points: PlotPoints = response
                    .into_iter()
                    .enumerate()
                    // Only keep first half of results.
                    .filter(|(index, _it)| *index < FFT_SAMPLE_SIZE / 2)
                    // Take log of values to make dB.
                    .map(|(index, it)| [freq_window * index as f64, it.log10() as f64])
                    .collect();
                let line = Line::new("Response", points).color(Color32::WHITE);
                Plot::new("Frequency Response")
                    .view_aspect(2.0)
                    .default_y_bounds(-10.0, 5.0)
                    .x_axis_label("Frequency (Hz)")
                    .y_axis_label("Response (dB)")
                    .show(ui, |plot_ui| plot_ui.line(line));
            }
        }
    }
}
