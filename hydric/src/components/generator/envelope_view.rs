use crate::view::View;
use crate::widget::knob;
use egui::{
    Color32, Pos2, Rect, Ui,
    containers::Frame,
    emath::RectTransform,
    epaint::{PathStroke, Shape},
    pos2, vec2,
};
use shared::model::AdsrEnvelope;
use state::{Action, TypeField};

pub struct EnvelopeView<'a, F: Fn(Action), G: Fn()> {
    envelope: &'a AdsrEnvelope,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> EnvelopeView<'a, F, G> {
    pub fn new(envelope: &'a AdsrEnvelope, dispatch: F, on_release: G) -> Self {
        EnvelopeView {
            envelope,
            dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn()> View for EnvelopeView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let envelope = self.envelope.clone();
        let dispatch = &self.dispatch;
        let on_release = &self.on_release;

        knob(
            ui,
            "Attack",
            envelope.attack,
            |attack| {
                dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                    attack,
                    ..envelope
                })))
            },
            0.0..=1.0,
            on_release,
        );
        knob(
            ui,
            "Decay",
            envelope.decay,
            |decay| {
                dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                    decay,
                    ..envelope
                })))
            },
            0.0..=1.0,
            on_release,
        );
        knob(
            ui,
            "Sustain",
            envelope.sustain,
            |sustain| {
                dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                    sustain,
                    ..envelope
                })))
            },
            0.0..=1.0,
            on_release,
        );
        knob(
            ui,
            "Release",
            envelope.release,
            |release| {
                dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                    release,
                    ..envelope
                })))
            },
            0.0..=1.0,
            on_release,
        );

        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let desired_size = vec2(100.0, 50.0);
            let (_id, rect) = ui.allocate_space(desired_size);
            let to_screen =
                RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 1.0..=0.0), rect);

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
