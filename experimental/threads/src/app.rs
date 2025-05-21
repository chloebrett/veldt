use crate::frame_history::View;
use crate::{AudioPlayer, FrameHistory};
use egui::{Color32, Frame, Pos2, Rect, Ui, pos2, vec2};
use epaint::PathStroke;

pub struct App {
    pub player: AudioPlayer,
    pub frame_history: FrameHistory,
    pub freq: f64,
    pub expensiveness: u32,
    pub freq_tx: crossbeam_channel::Sender<f64>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();

        Self {
            player: AudioPlayer::new(rx),
            freq: 0.0,
            expensiveness: 10,
            freq_tx: tx,
            frame_history: FrameHistory::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);

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

            ui.add(
                egui::Slider::new(&mut self.expensiveness, 10..=10000)
                    .text("UI lag")
                    .logarithmic(true),
            );

            expensive_ui(ui, self.expensiveness);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                self.frame_history.ui(ui);
            });
        });
    }
}

fn expensive_ui(ui: &mut Ui, count: u32) {
    let color = if ui.visuals().dark_mode {
        Color32::from_additive_luminance(196)
    } else {
        Color32::from_black_alpha(240)
    };

    Frame::canvas(ui.style()).show(ui, |ui| {
        ui.ctx().request_repaint();
        let time = ui.input(|i| i.time);

        let desired_size = 500.0 * vec2(1.0, 0.35);
        let (_id, rect) = ui.allocate_space(desired_size);

        let to_screen =
            emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);

        let mut shapes = vec![];

        for mode in 2..count {
            let mode = mode as f64;
            let n = 120;
            let speed = 1.5;

            let points: Vec<Pos2> = (0..=n)
                .map(|i| {
                    let t = i as f64 / (n as f64);
                    let amp = (time * speed * mode).sin() / mode;
                    let y = amp * (t * std::f64::consts::TAU / 2.0 * mode).sin();
                    to_screen * pos2(t as f32, y as f32)
                })
                .collect();

            let thickness = 10.0 / mode as f32;
            shapes.push(epaint::Shape::line(
                points,
                PathStroke::new(thickness, color),
            ));
        }

        ui.painter().extend(shapes);
    });
}
