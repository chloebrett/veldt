use crate::transform::Transform;
use crate::view::View;
use crate::widget::knob;
use egui::cache::{ComputerMut, FrameCache};
use egui::{
    Color32, Pos2, Rect, Ui,
    containers::Frame,
    emath::RectTransform,
    epaint::{PathStroke, Shape},
    pos2, vec2,
};
use log::info;
use ordered_float::OrderedFloat;
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

fn envelope_line(envelope: &EnvelopeKey) -> Vec<Pos2> {
    let mut points = vec![];
    if *envelope.attack > 0.0 {
        points.push(pos2(0.0, 0.0));
    }
    points.push(pos2(*envelope.attack, 1.0));
    points.push(pos2(*envelope.attack + *envelope.decay, *envelope.sustain));
    points.push(pos2(1.0 - *envelope.release, *envelope.sustain));
    if *envelope.release > 0.0 {
        points.push(pos2(1.0, 0.0));
    }
    points
}

#[derive(Default)]
struct AdsrEnvelopeComputer;

#[derive(Hash, Copy, Clone, Debug)]
struct EnvelopeKey {
    attack: OrderedFloat<f32>,
    decay: OrderedFloat<f32>,
    sustain: OrderedFloat<f32>,
    release: OrderedFloat<f32>,
}

impl From<AdsrEnvelope> for EnvelopeKey {
    fn from(other: AdsrEnvelope) -> Self {
        EnvelopeKey {
            attack: OrderedFloat(other.attack),
            decay: OrderedFloat(other.decay),
            sustain: OrderedFloat(other.sustain),
            release: OrderedFloat(other.release),
        }
    }
}

impl ComputerMut<EnvelopeKey, Shape> for AdsrEnvelopeComputer {
    fn compute(&mut self, envelope: EnvelopeKey) -> Shape {
        info!("Computing shapes for ADSR envelope: {:?}", envelope);
        let thickness = 2.0;
        Shape::line(
            envelope_line(&envelope),
            PathStroke::new(thickness, Color32::WHITE),
        )
    }
}

type AdsrEnvelopeCache<'a> = FrameCache<Shape, AdsrEnvelopeComputer>;

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

            let shape = ui.memory_mut(|memory| {
                let cache = memory.caches.cache::<AdsrEnvelopeCache<'_>>();
                cache.get(self.envelope.clone().into())
            });

            ui.painter().extend(vec![shape].transform(to_screen));
        });
    }
}
