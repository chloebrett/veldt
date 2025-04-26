use crate::AudioPlayer;

pub struct App {
    pub player: AudioPlayer,
    pub freq: f64,
    pub freq_tx: crossbeam_channel::Sender<f64>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();

        Self {
            player: AudioPlayer::new(rx),
            freq: 0.0,
            freq_tx: tx,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut freq = self.freq;
            ui.add(
                egui::Slider::new(&mut freq, 100.0..=10000.0)
                    .text("Frequency")
                    .logarithmic(true),
            );
            if freq != self.freq {
                self.freq = freq;
                self.freq_tx.try_send(freq).unwrap();
            }

            if ui.button("Play").clicked() {
                log::info!("Clicked play");
                self.player.init();
            }
        });
    }
}
