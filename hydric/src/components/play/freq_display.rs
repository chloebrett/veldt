use crate::{AudioState, playback::AudioPlayer, view::View};
use egui::{
    Color32, Ui,
    cache::{ComputerMut, FrameCache},
};
use egui_plot::{Line, Plot, PlotPoints};
use mesic::{
    eq::FrequencyResponsePoint, fft::{fft, hann_window}, FFT_SAMPLE_SIZE, SAMPLE_RATE
};
use ordered_float::OrderedFloat;
use shared::serialize::map_vec;

pub struct FrequencyDisplay<'a> {
    audio_state: Option<&'a AudioState>,
    frame_rate: i32, // The number of times per second the visualisation will be rendered.
    frequency_points: Option<Vec<FrequencyResponsePoint>>, // For charting a static frequency graph 
}

impl<'a> FrequencyDisplay<'a> {
    pub fn new(audio_state: Option<&'a AudioState>, frequency_points: Option<Vec<FrequencyResponsePoint>>) -> Self {
        FrequencyDisplay {
            audio_state,
            frame_rate: 60,
            frequency_points
        }
    }

    /// Create frequency display shapes synced with playing audio.
    fn render_display(
        &self,
        ui: &mut Ui,
        player: Option<&AudioPlayer>,
        audio: Vec<OrderedFloat<f32>>,
    ) -> Option<Vec<f32>> {
        // let current_sample = player.unwrap().effective_pos();
        let current_sample = if let Some(audio_player) = &player {
            audio_player.effective_pos()
        } else {
            0
        };
        let frame_size = (SAMPLE_RATE / self.frame_rate) as usize;
        // Round `current_sample` so that the audio will be broken up into chunks based on
        // the visualisation frame rate.
        let chunk_head = current_sample / frame_size * frame_size;
        if chunk_head + FFT_SAMPLE_SIZE >= audio.len() {
            return None;
        }

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
        let FrequencyDisplay { audio_state, .. } = *self;
        let audio: Vec<f32> = if let Some(audio_state) = &self.audio_state {
            let audio = &audio_state.audio;

            // Note: only visualising the left channel.
            // TODO: decide how to visualise both left and right.
            audio.iter().map(|it| it[0]).collect()
        } else {
            Vec::new()
        };


        // Cast as `OrderedFloat` so that values implement `Eq` required for hashing in cache.
        let ordered_audio: Vec<OrderedFloat<f32>> = map_vec(audio.to_vec());
        let mut plot_shapes = vec![];

        let response = if let Some(audio_state) = &audio_state {
            self.render_display(ui, Some(&audio_state.player), ordered_audio)
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
        Plot::new("Frequency Response")
            .view_aspect(2.0)
            .default_x_bounds(0.0, SAMPLE_RATE as f64 / 2.0)
            .default_y_bounds(-10.0, 5.0)
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
