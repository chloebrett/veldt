use crate::local_state::{GetSet, LocalState};
use crate::widget::FrequencyDisplay as FreqDisplay2;
use crate::{playback::AudioPlayer, view::View};
use egui::{Button, pos2};
use egui::{
    Color32, Ui,
    cache::{ComputerMut, FrameCache},
};
use egui_plot::{Line, Plot, PlotPoints};
use mesic::{
    FFT_SAMPLE_SIZE, SAMPLE_RATE,
    fft::{fft, hann_window},
    to_db,
};
use ordered_float::OrderedFloat;
use ringbuffer::RingBuffer;
use shared::serialize::map_vec;
use std::cmp::max;

pub struct FrequencyDisplay<'a> {
    player: &'a AudioPlayer,
    local_state: &'a LocalState,
}

impl<'a> FrequencyDisplay<'a> {
    pub fn new(player: &'a AudioPlayer, local_state: &'a LocalState) -> Self {
        FrequencyDisplay {
            player,
            local_state,
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
        let audio = self.player.recent_buf().iter();

        // Note: only visualising the left channel.
        // TODO: decide how to visualise both left and right.
        let audio: Vec<f32> = audio.map(|it| it[0]).collect();

        // Cast as `OrderedFloat` so that values implement `Eq` required for hashing in cache.
        let ordered_audio: Vec<OrderedFloat<f32>> = map_vec(audio.to_vec());
        let mut plot_shapes = vec![];
        let mut plot_points = vec![];
        if let Some(response) = self.render_display(ui, ordered_audio) {
            let freq_window = SAMPLE_RATE as f64 / FFT_SAMPLE_SIZE as f64;
            let points: PlotPoints = response
                .into_iter()
                .enumerate()
                // Only keep first half of results.
                .filter(|(index, _it)| *index < FFT_SAMPLE_SIZE / 2)
                // Take log of values to make dB.
                .map(|(index, it)| {
                    let [x, y] = [freq_window * index as f64, to_db(it) as f64];
                    plot_points.push(pos2(x as f32, y as f32));
                    [x, y]
                })
                .collect();
            plot_shapes.push(Line::new("Response", points).color(Color32::WHITE));
        };
        let log = self.local_state.log_frequency_display.get();
        ui.add(FreqDisplay2::new(&plot_points).logarithmic(log));
        if ui
            .add(Button::new("Log Frequencies").selected(log))
            .clicked()
        {
            self.local_state.log_frequency_display.set(!log);
        }
        // TODO: investigate logarithmic x axis.
        Plot::new("Frequency Response")
            .view_aspect(2.0)
            .default_x_bounds(0.0, SAMPLE_RATE as f64 / 2.0)
            .default_y_bounds(-60.0, 6.0)
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
