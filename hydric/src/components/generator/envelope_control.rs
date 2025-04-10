use crate::widget::{FloatRange, knob};
use egui::{
    Color32, Pos2, Rect, Ui,
    containers::Frame,
    emath::RectTransform,
    epaint::{PathStroke, Shape},
    pos2, vec2,
};
use shared::model::{AdsrEnvelope, GeneratorType};
use state::{Action, Selector, Store};

pub fn envelope_control(store: &Store, ui: &mut Ui, generator_index: usize) {
    let sel = Selector::Generator(generator_index);
    let dispatch = |action| store.dispatch(&sel, action);

    let generator_type = store.get().project.generators[generator_index].kind.clone();
    let config = match generator_type {
        GeneratorType::SimpleWave { config } => config,
        GeneratorType::Noise { .. } => todo!(),
    };
    let envelope = config.envelope.clone();
    knob(
        ui,
        "Attack",
        envelope.attack,
        |attack| dispatch(Action::SetEnvelope(AdsrEnvelope { attack, ..envelope })),
        FloatRange(0.0, 1.0),
    );
    knob(
        ui,
        "Decay",
        envelope.decay,
        |decay| dispatch(Action::SetEnvelope(AdsrEnvelope { decay, ..envelope })),
        FloatRange(0.0, 1.0),
    );
    knob(
        ui,
        "Sustain",
        envelope.sustain,
        |sustain| {
            dispatch(Action::SetEnvelope(AdsrEnvelope {
                sustain,
                ..envelope
            }))
        },
        FloatRange(0.0, 1.0),
    );
    knob(
        ui,
        "Release",
        envelope.release,
        |release| {
            dispatch(Action::SetEnvelope(AdsrEnvelope {
                release,
                ..envelope
            }))
        },
        FloatRange(0.0, 1.0),
    );

    Frame::canvas(ui.style()).show(ui, |ui| {
        ui.ctx().request_repaint();
        let desired_size = vec2(100.0, 50.0);
        let (_id, rect) = ui.allocate_space(desired_size);
        let to_screen = RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 1.0..=0.0), rect);

        let thickness = 2.0;
        let shape = Shape::line(
            envelope_line(&envelope)
                .into_iter()
                .map(|it| to_screen * it)
                .collect(),
            PathStroke::new(thickness, Color32::WHITE),
        );
        ui.painter().extend(vec![shape]);
    });
}

fn envelope_line(envelope: &AdsrEnvelope) -> Vec<Pos2> {
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
    points
}
