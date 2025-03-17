use crate::state::{Action, Store};
use egui::{Color32, Rect, Ui, containers::Frame, emath, epaint, epaint::PathStroke, pos2, vec2};
use shared::model::{AdsrEnvelope, GeneratorType};

fn update_envelope(
    store: &mut Store,
    prev: AdsrEnvelope,
    attack: Option<f64>,
    decay: Option<f64>,
    sustain: Option<f64>,
    release: Option<f64>,
) {
    let next = AdsrEnvelope {
        attack: attack.map(|it| it as f32).unwrap_or(prev.attack),
        decay: decay.map(|it| it as f32).unwrap_or(prev.decay),
        sustain: sustain.map(|it| it as f32).unwrap_or(prev.sustain),
        release: release.map(|it| it as f32).unwrap_or(prev.release),
    };
    if next != prev {
        store.dispatch(Action::SetEnvelope {
            generator_index: 0,
            envelope: next,
        })
    }
}

pub fn envelope_control(store: &mut Store, ui: &mut Ui) {
    let generator_type = store.project.generators[0].kind.clone();
    let config = match generator_type {
        GeneratorType::SimpleWave { config } => config,
    };
    let envelope = config.envelope.clone();
    ui.add(
        egui::Slider::from_get_set(0.0..=1.0, |it| {
            update_envelope(store, envelope.clone(), it, None, None, None);
            envelope.attack.into()
        })
        .text("Attack"),
    );
    ui.add(
        egui::Slider::from_get_set(0.0..=1.0, |it| {
            update_envelope(store, envelope.clone(), None, it, None, None);
            envelope.decay.into()
        })
        .text("Decay"),
    );
    ui.add(
        egui::Slider::from_get_set(0.0..=1.0, |it| {
            update_envelope(store, envelope.clone(), None, None, it, None);
            envelope.sustain.into()
        })
        .text("Sustain"),
    );
    ui.add(
        egui::Slider::from_get_set(0.0..=1.0, |it| {
            update_envelope(store, envelope.clone(), None, None, None, it);
            envelope.release.into()
        })
        .text("Release"),
    );

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
