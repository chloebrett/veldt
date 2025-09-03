use crate::transform::Transform;
use crate::view::View;
use crate::widget::{add_typable_knob, styled_knob};
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
        Self {
            envelope,
            dispatch,
            on_release,
        }
    }
}

pub fn envelope_line(envelope: &AdsrEnvelope, x_size: f32) -> Vec<Pos2> {
    vec![
        pos2(0.0, 0.0),
        pos2(envelope.attack, 1.0),
        pos2(envelope.attack + envelope.decay, envelope.sustain),
        pos2(x_size - envelope.release, envelope.sustain),
        pos2(x_size, 0.0),
    ]
}

impl<F: Fn(Action), G: Fn()> View for EnvelopeView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let envelope = self.envelope.clone();
        let dispatch = &self.dispatch;
        let on_release = &self.on_release;
        let attack_knob = styled_knob(
            envelope.attack,
            |attack| {
                dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                    attack,
                    ..envelope
                })))
            },
            0.0..=1000.0,
        )
        .with_neutral(100.0);
        let decay_knob = styled_knob(
            envelope.decay,
            |decay| {
                dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                    decay,
                    ..envelope
                })))
            },
            0.0..=1000.0,
        )
        .with_neutral(100.0);
        let sustain_knob = styled_knob(
            envelope.sustain,
            |sustain| {
                dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                    sustain,
                    ..envelope
                })))
            },
            0.0..=1.0,
        )
        .with_neutral(0.8);
        let release_knob = styled_knob(
            envelope.release,
            |release| {
                dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                    release,
                    ..envelope
                })))
            },
            0.0..=1000.0,
        )
        .with_neutral(100.0);
        ui.horizontal(|ui| {
            Frame::canvas(ui.style()).show(ui, |ui| {
                ui.ctx().request_repaint();
                let desired_size = vec2(200.0, 115.0);
                let (_id, rect) = ui.allocate_space(desired_size);
                let x_size = (envelope.attack + envelope.decay + envelope.release) * 1.2;
                let to_screen =
                    RectTransform::from_to(Rect::from_x_y_ranges(0.0..=x_size, 1.0..=0.0), rect);

                let thickness = 2.0;
                let shape = Shape::line(
                    envelope_line(&envelope, x_size),
                    PathStroke::new(thickness, Color32::WHITE),
                );

                ui.painter().add(shape.transform(to_screen));
            });
            ui.vertical(|ui| {
                add_typable_knob(
                    ui,
                    attack_knob,
                    "Attack (ms)",
                    envelope.attack,
                    |attack| {
                        dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                            attack,
                            ..envelope
                        })))
                    },
                    0.0..=1000.0,
                    on_release,
                    60.0,
                );
                add_typable_knob(
                    ui,
                    decay_knob,
                    "Decay (ms)",
                    envelope.decay,
                    |decay| {
                        dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                            decay,
                            ..envelope
                        })))
                    },
                    0.0..=1000.0,
                    on_release,
                    60.0,
                );
                add_typable_knob(
                    ui,
                    sustain_knob,
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
                    60.0,
                );
                add_typable_knob(
                    ui,
                    release_knob,
                    "Release (ms)",
                    envelope.release,
                    |release| {
                        dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                            release,
                            ..envelope
                        })))
                    },
                    0.0..=1000.0,
                    on_release,
                    60.0,
                );
            });
        });
    }
}
