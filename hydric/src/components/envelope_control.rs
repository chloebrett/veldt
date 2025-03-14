use crate::audio_player::{Handle, play};
use egui::{Color32, Rect, Ui, containers::Frame, emath, epaint, epaint::PathStroke, pos2, vec2};
use mesic::{SAMPLE_RATE, create_scale_values, create_track, render as local_render};
use poll_promise::Promise;
use shared::model::AdsrEnvelope;
use strum::IntoEnumIterator;

pub fn envelope_control(envelope: &mut AdsrEnvelope, ui: &mut Ui) {
    let headroom = 1.0 - envelope.attack - envelope.decay - envelope.release;
    let max_attack = headroom + envelope.attack;
    let max_decay = headroom + envelope.decay;
    let max_release = headroom + envelope.release;

    if envelope.attack > max_attack {
        envelope.attack = max_attack;
    }
    if envelope.decay > max_decay {
        envelope.decay = max_decay;
    }
    if envelope.release > max_release {
        envelope.release = max_release;
    }

    ui.add(egui::Slider::new(&mut envelope.attack, 0.0..=1.0).text("Attack"));
    ui.add(egui::Slider::new(&mut envelope.decay, 0.0..=1.0).text("Decay"));
    ui.add(egui::Slider::new(&mut envelope.sustain, 0.0..=1.0).text("Sustain"));
    ui.add(egui::Slider::new(&mut envelope.release, 0.0..=1.0).text("Release"));

    Frame::canvas(ui.style()).show(ui, |ui| {
        ui.ctx().request_repaint();
        let desired_size = vec2(100.0, 50.0);
        let (_id, rect) = ui.allocate_space(desired_size);
        let to_screen =
            emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 1.0..=0.0), rect);

        let mut points = vec![];
        if envelope.attack > 0.0 {
            points.push(pos2(0.0, 0.0));
        }
        points.push(pos2(envelope.attack, 1.0));
        points.push(pos2(envelope.attack + envelope.decay, envelope.sustain));
        points.push(pos2(1.0 - envelope.release, envelope.sustain));
        if envelope.release > 0.0 {
            points.push(pos2(1.0, 0.0));
        }

        let thickness = 2.0;
        let shapes = vec![epaint::Shape::line(
            points.into_iter().map(|it| to_screen * it).collect(),
            PathStroke::new(thickness, Color32::WHITE),
        )];
        ui.painter().extend(shapes);
    });
}
