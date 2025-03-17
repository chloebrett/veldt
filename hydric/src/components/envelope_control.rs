use crate::state::{Action, Store, get_set};
use egui::{Color32, Rect, Ui, containers::Frame, emath, epaint, epaint::PathStroke, pos2, vec2};
use shared::model::{AdsrEnvelope, GeneratorType};

pub fn envelope_control(store: &mut Store, ui: &mut Ui) {
    let generator_type = store.get().project.generators[0].kind.clone();
    let config = match generator_type {
        GeneratorType::SimpleWave { config } => config,
    };
    let envelope = config.envelope.clone();
    ui.add(
        egui::Slider::from_get_set(
            0.0..=1.0,
            get_set(envelope.attack.into(), |it| {
                store.dispatch(Action::SetEnvelope {
                    generator_index: 0,
                    envelope: AdsrEnvelope {
                        attack: it as f32,
                        ..envelope
                    },
                })
            }),
        )
        .text("Attack"),
    );
    ui.add(
        egui::Slider::from_get_set(
            0.0..=1.0,
            get_set(envelope.decay.into(), |it| {
                store.dispatch(Action::SetEnvelope {
                    generator_index: 0,
                    envelope: AdsrEnvelope {
                        decay: it as f32,
                        ..envelope
                    },
                })
            }),
        )
        .text("Decay"),
    );
    ui.add(
        egui::Slider::from_get_set(
            0.0..=1.0,
            get_set(envelope.sustain.into(), |it| {
                store.dispatch(Action::SetEnvelope {
                    generator_index: 0,
                    envelope: AdsrEnvelope {
                        sustain: it as f32,
                        ..envelope
                    },
                })
            }),
        )
        .text("Sustain"),
    );
    ui.add(
        egui::Slider::from_get_set(
            0.0..=1.0,
            get_set(envelope.release.into(), |it| {
                store.dispatch(Action::SetEnvelope {
                    generator_index: 0,
                    envelope: AdsrEnvelope {
                        release: it as f32,
                        ..envelope
                    },
                })
            }),
        )
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
