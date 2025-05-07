use crate::DataState;
use crate::transform::Transform;
use crate::view::View;
use crate::widget::{TabDisplay, TabOrientation, knob};
use egui::cache::{ComputerMut, FrameCache};
use egui::{
    Color32, Pos2, Rect, Stroke, Ui, Vec2,
    containers::Frame,
    emath::RectTransform,
    epaint::{PathStroke, Shape},
    pos2, vec2,
};
use log::info;
use ordered_float::OrderedFloat;
use shared::model::{AdsrEnvelope, SubSynthConfig};
use state::{Action, TypeField};

pub struct SubSynthEnvelopeView<'a, F: Fn(Action), G: Fn()> {
    config: &'a SubSynthConfig,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> SubSynthEnvelopeView<'a, F, G> {
    pub fn new(config: &'a SubSynthConfig, dispatch: F, on_release: G) -> Self {
        Self {
            config,
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
        Self {
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

impl<F: Fn(Action), G: Fn()> View for SubSynthEnvelopeView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let config = self.config.clone();
        let dispatch = &self.dispatch;
        let on_release = &self.on_release;

        let handle_env_tab_click: Box<dyn Fn(&mut egui::Ui, usize) + Send + Sync + 'static> =
            Box::new(move |ui, index| {
                DataState::SubSynthEnvTab.set_value(ui, index);
            });

        let active_env_tab = DataState::SubSynthEnvTab
            .get_value::<usize>(ui)
            .unwrap_or_default();

        let outer_frame = Frame::new()
            .fill(Color32::from_rgb(50, 50, 50))
            .stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 50)))
            .corner_radius(8.0)
            .inner_margin(6.0);

        outer_frame.show(ui, |ui| {
            let original_spacing = ui.spacing().item_spacing; // store original spacing
            ui.spacing_mut().item_spacing = Vec2::ZERO; // set spacing to zero so that the tabs and associated content actually touch each other

            ui.horizontal(|ui| {
                TabDisplay::new(
                    active_env_tab,
                    vec!["ENV 1", "ENV 2", "ENV 3"],
                    TabOrientation::Left,
                    handle_env_tab_click,
                )
                .ui(ui);
                let inner_frame = Frame::new()
                    .fill(Color32::from_rgb(30, 30, 30))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(30, 30, 30)))
                    .corner_radius(8.0)
                    .inner_margin(15.0);

                inner_frame.show(ui, |ui| {
                    ui.vertical(|ui| {
                        // Envelope graph
                        Frame::canvas(ui.style()).show(ui, |ui| {
                            ui.ctx().request_repaint();
                            let desired_size = vec2(300.0, 160.0);
                            let (_id, rect) = ui.allocate_space(desired_size);
                            let to_screen = RectTransform::from_to(
                                Rect::from_x_y_ranges(0.0..=1.0, 1.0..=0.0),
                                rect,
                            );
                            let shape = ui.memory_mut(|memory| {
                                let cache = memory.caches.cache::<AdsrEnvelopeCache<'_>>();
                                info!("Active Env: {:?}", active_env_tab);
                                cache.get(config.envelopes[active_env_tab].clone().into())
                            });
                            ui.painter().add(shape.transform(to_screen));
                        });
                        ui.add_space(10.0);
                        // All the knobs for envelope modification
                        ui.horizontal(|ui| {
                            knob(
                                ui,
                                "A",
                                config.envelopes[active_env_tab].attack,
                                |attack| {
                                    dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                                        attack,
                                        ..config.envelopes[active_env_tab]
                                    })))
                                },
                                0.0..=1.0,
                                /* neutral= */ 0.1,
                                on_release,
                            );
                            knob(
                                ui,
                                "D",
                                config.envelopes[active_env_tab].decay,
                                |decay| {
                                    dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                                        decay,
                                        ..config.envelopes[active_env_tab]
                                    })))
                                },
                                0.0..=1.0,
                                /* neutral= */ 0.1,
                                on_release,
                            );
                            knob(
                                ui,
                                "S",
                                config.envelopes[active_env_tab].sustain,
                                |sustain| {
                                    dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                                        sustain,
                                        ..config.envelopes[active_env_tab]
                                    })))
                                },
                                0.0..=1.0,
                                /* neutral= */ 0.8,
                                on_release,
                            );
                            knob(
                                ui,
                                "R",
                                config.envelopes[active_env_tab].release,
                                |release| {
                                    dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                                        release,
                                        ..config.envelopes[active_env_tab]
                                    })))
                                },
                                0.0..=1.0,
                                /* neutral= */ 0.1,
                                on_release,
                            );
                        });
                    })
                })
            });
            ui.spacing_mut().item_spacing = original_spacing; // reset ui spacing back to original
        });
    }
}
