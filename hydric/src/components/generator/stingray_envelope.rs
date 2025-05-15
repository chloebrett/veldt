use super::envelope_line;
use crate::transform::Transform;
use crate::view::View;
use crate::widget::{TabDisplay, TabOrientation, inner_frame, knob, outer_frame};
use crate::{GetSet, LocalState};
use egui::{
    Color32, Rect, Ui, Vec2,
    containers::Frame,
    emath::RectTransform,
    epaint::{PathStroke, Shape},
    vec2,
};
use shared::model::{AdsrEnvelope, StingrayConfig};
use state::{Action, TypeField};

pub struct StingrayEnvelopeView<'a, F: Fn(Action), G: Fn()> {
    config: &'a StingrayConfig,
    dispatch: F,
    on_release: G,
    local_state: &'a LocalState,
}

impl<'a, F: Fn(Action), G: Fn()> StingrayEnvelopeView<'a, F, G> {
    pub fn new(
        config: &'a StingrayConfig,
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

impl<F: Fn(Action), G: Fn()> View for StingrayEnvelopeView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let config = self.config.clone();
        let dispatch = &self.dispatch;
        let on_release = &self.on_release;

        let handle_env_tab_click = |index| {
            self.local_state.stingray_env_tab.set(index);
        };
        let active_env_tab = self.local_state.stingray_env_tab.get();

        outer_frame().show(ui, |ui| {
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

                inner_frame().inner_margin(15.0).show(ui, |ui| {
                    ui.vertical(|ui| {
                        // Envelope graph
                        Frame::canvas(ui.style()).show(ui, |ui| {
                            ui.ctx().request_repaint();
                            let desired_size = vec2(300.0, 160.0);
                            let (_id, rect) = ui.allocate_space(desired_size);
                            let envelope = config.envelopes[active_env_tab].clone();
                            let x_size =
                                (envelope.attack + envelope.decay + envelope.release) * 1.2;
                            let to_screen = RectTransform::from_to(
                                Rect::from_x_y_ranges(0.0..=x_size, 1.0..=0.0),
                                rect,
                            );

                            let thickness = 2.0;
                            let shape = Shape::line(
                                envelope_line(&envelope, x_size),
                                PathStroke::new(thickness, Color32::WHITE),
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
                                    dispatch(Action::SetChild(TypeField::Envelope(AdsrEnvelope {
                                        attack,
                                        ..config.envelopes[active_env_tab]
                                    })))
                                },
                                0.0..=1000.0,
                                /* neutral= */ 100.0,
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
                                0.0..=1000.0,
                                /* neutral= */ 100.0,
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
                                /* neutral= */ 0.1,
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
                                0.0..=1000.0,
                                /* neutral= */ 100.0,
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
