use crate::transform::Transform;
use crate::view::View;
use crate::widget::{TabDisplay, TabOrientation, knob};
use crate::{GetSet, LocalState};
use egui::{
    Color32, Pos2, Rect, Stroke, Ui, Vec2,
    containers::Frame,
    emath::RectTransform,
    epaint::{PathStroke, Shape},
    pos2, vec2,
};
use log::info;
use shared::model::{AdsrEnvelope, SubSynthConfig};
use state::{Action, TypeField};

pub struct SubSynthEnvelopeView<'a, F: Fn(Action), G: Fn()> {
    config: &'a SubSynthConfig,
    dispatch: F,
    on_release: G,
    local_state: &'a LocalState,
}

impl<'a, F: Fn(Action), G: Fn()> SubSynthEnvelopeView<'a, F, G> {
    pub fn new(
        config: &'a SubSynthConfig,
        dispatch: F,
        on_release: G,
        local_state: &'a LocalState,
    ) -> Self {
        Self {
            config,
            dispatch,
            on_release,
            local_state,
        }
    }
}

fn envelope_line(envelope: AdsrEnvelope) -> Vec<Pos2> {
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

impl<F: Fn(Action), G: Fn()> View for SubSynthEnvelopeView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let config = self.config.clone();
        let dispatch = &self.dispatch;
        let on_release = &self.on_release;

        let handle_env_tab_click = |index| {
            self.local_state.subsynth_env_tab.set(index);
        };
        let active_env_tab = self.local_state.subsynth_env_tab.get();

        let outer_frame = Frame::new()
            .fill(Color32::from_gray(50))
            .stroke(Stroke::new(1.0, Color32::from_gray(50)))
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

                Frame::new()
                    .fill(Color32::from_gray(30))
                    .stroke(Stroke::new(1.0, Color32::from_gray(30)))
                    .corner_radius(8.0)
                    .inner_margin(15.0)
                    .show(ui, |ui| {
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
                                info!("Envelope number: {}", active_env_tab);
                                let shape = Shape::line(
                                    envelope_line(config.envelopes[active_env_tab].clone()),
                                    PathStroke::new(2.0, Color32::WHITE),
                                );

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
                                        dispatch(Action::SetChild(TypeField::Envelope(
                                            AdsrEnvelope {
                                                attack,
                                                ..config.envelopes[active_env_tab]
                                            },
                                        )))
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
                                        dispatch(Action::SetChild(TypeField::Envelope(
                                            AdsrEnvelope {
                                                decay,
                                                ..config.envelopes[active_env_tab]
                                            },
                                        )))
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
                                        dispatch(Action::SetChild(TypeField::Envelope(
                                            AdsrEnvelope {
                                                sustain,
                                                ..config.envelopes[active_env_tab]
                                            },
                                        )))
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
                                        dispatch(Action::SetChild(TypeField::Envelope(
                                            AdsrEnvelope {
                                                release,
                                                ..config.envelopes[active_env_tab]
                                            },
                                        )))
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
