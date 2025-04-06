use egui::{
    Color32, Pos2, Rect, Slider, Ui,
    containers::Frame,
    emath::RectTransform,
    epaint::{PathStroke, Shape},
    pos2, vec2,
};
use shared::model::{AdsrEnvelope, GeneratorType};
use state::{Action, Selector, Store, get_set};

pub fn envelope_control(store: &Store, ui: &mut Ui, generator_index: usize) {
    let sel = Selector::Generator(generator_index);

    let generator_type = store.get().project.generators[generator_index].kind.clone();
    let config = match generator_type {
        GeneratorType::SimpleWave { config } => config,
    };
    let envelope = config.envelope.clone();
    ui.add(
        Slider::from_get_set(
            0.0..=1.0,
            get_set(envelope.attack.into(), |it| {
                store.dispatch(
                    &sel,
                    Action::SetEnvelope(AdsrEnvelope {
                        attack: it as f32,
                        ..envelope
                    }),
                )
            }),
        )
        .text("Attack"),
    );
    ui.add(
        Slider::from_get_set(
            0.0..=1.0,
            get_set(envelope.decay.into(), |it| {
                store.dispatch(
                    &sel,
                    Action::SetEnvelope(AdsrEnvelope {
                        decay: it as f32,
                        ..envelope
                    }),
                )
            }),
        )
        .text("Decay"),
    );
    ui.add(
        Slider::from_get_set(
            0.0..=1.0,
            get_set(envelope.sustain.into(), |it| {
                store.dispatch(
                    &sel,
                    Action::SetEnvelope(AdsrEnvelope {
                        sustain: it as f32,
                        ..envelope
                    }),
                )
            }),
        )
        .text("Sustain"),
    );
    ui.add(
        Slider::from_get_set(
            0.0..=1.0,
            get_set(envelope.release.into(), |it| {
                store.dispatch(
                    &sel,
                    Action::SetEnvelope(AdsrEnvelope {
                        release: it as f32,
                        ..envelope
                    }),
                )
            }),
        )
        .text("Release"),
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
