use super::envelope_line;
use crate::components::opacity_percentage_to_alpha;
use crate::transform::Transform;
use crate::view::View;
use crate::widget::{
    TabDisplay, TabOrientation, add_typable_knob, inner_frame, outer_frame, styled_knob,
};
use crate::{GetSet, LocalState};
use egui::{
    Color32, Margin, Rect, Ui, Vec2,
    containers::Frame,
    emath::RectTransform,
    epaint::{PathStroke, Shape},
    pos2, vec2,
};
use egui::{Sense, Stroke};
use lazy_static::lazy_static;
use shared::model::StingrayConfig;
use state::{Action, FloatField};

const CHART_FILL_ALPHA: u8 = opacity_percentage_to_alpha(20.0);

lazy_static! {
    pub static ref BLUE_OUTLINE: Color32 = Color32::from_rgb(79, 176, 255);
    pub static ref BLUE_FILL: Color32 =
        Color32::from_rgba_unmultiplied(79, 176, 255, CHART_FILL_ALPHA);
}

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

        outer_frame()
            .outer_margin(Margin {
                left: 0,
                right: 1,
                top: 10,
                bottom: 5,
            })
            .show(ui, |ui| {
                let original_spacing = ui.spacing().item_spacing; // store original spacing
                ui.spacing_mut().item_spacing = Vec2::ZERO; // set spacing to zero so that the tabs and associated content actually touch each other

                ui.horizontal(|ui| {
                    ui.add_space(2.0);
                    TabDisplay::new(
                        active_env_tab,
                        vec!["ENV 1", "ENV 2", "ENV 3"],
                        TabOrientation::Left,
                        handle_env_tab_click,
                    )
                    .ui(ui);

                    inner_frame()
                        .inner_margin(Margin {
                            left: 20,
                            right: 0,
                            top: 20,
                            bottom: 20,
                        })
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                // Envelope graph
                                ui.add_space(2.0); // left border
                                Frame::canvas(ui.style())
                                    .fill(Color32::from_gray(30))
                                    .stroke(Stroke::NONE)
                                    .show(ui, |ui| {
                                        ui.ctx().request_repaint();
                                        let desired_size = vec2(300.0, 160.0);
                                        let (_id, rect) = ui.allocate_space(desired_size);
                                        let envelope = config.envelopes[active_env_tab].clone();
                                        let x_size =
                                            (envelope.attack + envelope.decay + envelope.release)
                                                * 1.2;

                                        let to_screen = RectTransform::from_to(
                                            Rect::from_x_y_ranges(0.0..=x_size, 1.0..=0.0),
                                            rect,
                                        );

                                        let thickness = 2.5;
                                        let envelope_points = envelope_line(&envelope, x_size);
                                        let shape = Shape::line(
                                            envelope_points.clone(),
                                            PathStroke::new(thickness, *BLUE_OUTLINE),
                                        );

                                        ui.painter().add(shape.transform(to_screen));

                                        let transformed_envelope_points = envelope_points
                                            .into_iter()
                                            .map(|point| to_screen.transform_pos(point))
                                            .collect();
                                        ui.painter().add(Shape::convex_polygon(
                                            transformed_envelope_points,
                                            *BLUE_FILL,
                                            Stroke::NONE,
                                        ));

                                        // handles
                                        let attack_handle_id = ui.id().with("attack_handle");
                                        let attack_handle_pos =
                                            to_screen.transform_pos(pos2(envelope.attack, 1.0));
                                        let attack_handle_rect = Rect::from_center_size(
                                            attack_handle_pos,
                                            vec2(12.0, 12.0),
                                        );
                                        let attack_drag = Shape::rect_filled(
                                            attack_handle_rect,
                                            6.0,
                                            *BLUE_OUTLINE,
                                        );
                                        let attack_response = ui.interact(
                                            attack_handle_rect,
                                            attack_handle_id,
                                            Sense::drag(),
                                        );
                                        ui.painter().add(attack_drag);
                                        if attack_response.dragged() {
                                            if let Some(pointer_pos_screen) =
                                                attack_response.interact_pointer_pos()
                                            {
                                                let pointer_pos_model = to_screen
                                                    .inverse()
                                                    .transform_pos(pointer_pos_screen);
                                                let new_attack_time =
                                                    pointer_pos_model.x.max(0.0).min(x_size);
                                                dispatch(Action::SetFloat(
                                                    FloatField::AdsrAttack,
                                                    f32::min(1000.0, new_attack_time),
                                                ));
                                            }
                                        }

                                        let decay_handle_id = ui.id().with("decay_handle");
                                        let decay_handle_pos = to_screen.transform_pos(pos2(
                                            envelope.attack + envelope.decay,
                                            envelope.sustain,
                                        ));
                                        let decay_handle_rect = Rect::from_center_size(
                                            decay_handle_pos,
                                            vec2(12.0, 12.0),
                                        );
                                        let decay_drag = Shape::rect_filled(
                                            decay_handle_rect,
                                            6.0,
                                            *BLUE_OUTLINE,
                                        );
                                        let decay_response = ui.interact(
                                            decay_handle_rect,
                                            decay_handle_id,
                                            Sense::drag(),
                                        );
                                        ui.painter().add(decay_drag);
                                        if decay_response.dragged() {
                                            if let Some(pointer_pos_screen) =
                                                decay_response.interact_pointer_pos()
                                            {
                                                let pointer_pos_model = to_screen
                                                    .inverse()
                                                    .transform_pos(pointer_pos_screen);
                                                let new_decay_time =
                                                    pointer_pos_model.x.max(0.0).min(x_size);
                                                let new_sustain =
                                                    pointer_pos_model.y.max(0.0).min(1.0);
                                                dispatch(Action::SetFloat(
                                                    FloatField::AdsrSustain,
                                                    new_sustain,
                                                ));
                                                if new_decay_time <= envelope.attack {
                                                    dispatch(Action::SetFloat(
                                                        FloatField::AdsrAttack,
                                                        f32::min(1000.0, new_decay_time),
                                                    ));
                                                    dispatch(Action::SetFloat(
                                                        FloatField::AdsrDecay,
                                                        0.0,
                                                    ));
                                                } else {
                                                    dispatch(Action::SetFloat(
                                                        FloatField::AdsrDecay,
                                                        f32::max(
                                                            0.0,
                                                            f32::min(
                                                                1000.0,
                                                                new_decay_time - envelope.attack,
                                                            ),
                                                        ),
                                                    ));
                                                }
                                            }
                                            // ui.add_space(5.0);
                                        }
                                        ui.add_space(3.0);

                                        let sr_handle_id = ui.id().with("sr_handle");
                                        let sr_handle_pos = to_screen.transform_pos(pos2(
                                            x_size - envelope.release,
                                            envelope.sustain,
                                        ));
                                        let sr_handle_rect =
                                            Rect::from_center_size(sr_handle_pos, vec2(12.0, 12.0));
                                        let sr_drag =
                                            Shape::rect_filled(sr_handle_rect, 6.0, *BLUE_OUTLINE);
                                        let sr_response = ui.interact(
                                            sr_handle_rect,
                                            sr_handle_id,
                                            Sense::drag(),
                                        );
                                        ui.painter().add(sr_drag);
                                        if sr_response.dragged() {
                                            if let Some(pointer_pos_screen) =
                                                sr_response.interact_pointer_pos()
                                            {
                                                let pointer_pos_model = to_screen
                                                    .inverse()
                                                    .transform_pos(pointer_pos_screen);
                                                let new_release_time =
                                                    pointer_pos_model.x.max(0.0).min(x_size);
                                                let new_sustain =
                                                    pointer_pos_model.y.max(0.0).min(1.0);
                                                dispatch(Action::SetFloat(
                                                    FloatField::AdsrSustain,
                                                    new_sustain,
                                                ));
                                                dispatch(Action::SetFloat(
                                                    FloatField::AdsrRelease,
                                                    f32::min(x_size - new_release_time, 1000.0),
                                                ));
                                            }
                                        }
                                    });

                                // All the knobs for envelope modification
                                ui.add_space(20.0); // space btwn graph and knobs
                                ui.horizontal(|ui| {
                                    let a_knob = styled_knob(
                                        config.envelopes[active_env_tab].attack,
                                        |attack| {
                                            dispatch(Action::SetFloat(
                                                FloatField::AdsrAttack,
                                                attack,
                                            ));
                                        },
                                        0.0..=1000.0,
                                    )
                                    .with_neutral(100.0);
                                    let d_knob = styled_knob(
                                        config.envelopes[active_env_tab].decay,
                                        |decay| {
                                            dispatch(Action::SetFloat(
                                                FloatField::AdsrDecay,
                                                decay,
                                            ));
                                        },
                                        0.0..=1000.0,
                                    )
                                    .with_neutral(100.0);
                                    let s_knob = styled_knob(
                                        config.envelopes[active_env_tab].sustain,
                                        |sustain| {
                                            dispatch(Action::SetFloat(
                                                FloatField::AdsrSustain,
                                                sustain,
                                            ));
                                        },
                                        0.0..=1.0,
                                    )
                                    .with_neutral(0.8);
                                    let r_knob = styled_knob(
                                        config.envelopes[active_env_tab].release,
                                        |release| {
                                            dispatch(Action::SetFloat(
                                                FloatField::AdsrRelease,
                                                release,
                                            ));
                                        },
                                        0.0..=1000.0,
                                    )
                                    .with_neutral(100.0);
                                    add_typable_knob(
                                        ui,
                                        a_knob,
                                        "A",
                                        config.envelopes[active_env_tab].attack,
                                        |attack| {
                                            dispatch(Action::SetFloat(
                                                FloatField::AdsrAttack,
                                                attack,
                                            ));
                                        },
                                        0.0..=1000.0,
                                        on_release,
                                    );
                                    add_typable_knob(
                                        ui,
                                        d_knob,
                                        "D",
                                        config.envelopes[active_env_tab].decay,
                                        |decay| {
                                            dispatch(Action::SetFloat(
                                                FloatField::AdsrDecay,
                                                decay,
                                            ));
                                        },
                                        0.0..=1000.0,
                                        on_release,
                                    );
                                    add_typable_knob(
                                        ui,
                                        s_knob,
                                        "S",
                                        config.envelopes[active_env_tab].sustain,
                                        |sustain| {
                                            dispatch(Action::SetFloat(
                                                FloatField::AdsrSustain,
                                                sustain,
                                            ));
                                        },
                                        0.0..=1.0,
                                        on_release,
                                    );
                                    add_typable_knob(
                                        ui,
                                        r_knob,
                                        "R",
                                        config.envelopes[active_env_tab].release,
                                        |release| {
                                            dispatch(Action::SetFloat(
                                                FloatField::AdsrRelease,
                                                release,
                                            ));
                                        },
                                        0.0..=1000.0,
                                        on_release,
                                    );
                                });
                            });
                        });
                });
                ui.spacing_mut().item_spacing = original_spacing; // reset ui spacing back to original
            });
    }
}
