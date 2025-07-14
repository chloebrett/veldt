use super::MixerMatrixView;
use crate::GetSet;
use crate::local_state::LocalState;
use crate::playback::AudioPlayer;
use crate::view::View;
use crate::widget::StateWindow;
use crate::window_state::WindowKind;
use egui::{Button, Ui};
use egui_fader::Fader;
use mesic::from_db;
use mesic::to_db;
use state::TypeField;
use state::{Action, FloatField, MixerSelector, Store};

pub struct MixerView<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
    player: &'a AudioPlayer,
}

impl<'a> MixerView<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState, player: &'a AudioPlayer) -> Self {
        Self {
            store,
            local_state,
            player,
        }
    }
}

impl View for MixerView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            store,
            local_state,
            player,
            ..
        } = self;
        let active_mixer_sel = local_state
            .active_mixer_channel
            .get()
            .unwrap_or(MixerSelector(0));
        let MixerSelector(active_mixer_index) = active_mixer_sel;

        let on_release = || store.dispatchr(Action::Release);

        StateWindow::show_from_window_state(
            ui,
            &local_state.window_state,
            WindowKind::Mixer,
            "Mixer",
            |ui| {
                let matrix_open = local_state
                    .window_state
                    .get_visible(WindowKind::MixerMatrix);
                if ui
                    .add(Button::new("Matrix").selected(matrix_open))
                    .clicked()
                {
                    local_state
                        .window_state
                        .set_visible(WindowKind::MixerMatrix, !matrix_open);
                }
                StateWindow::show_from_window_state(
                    ui,
                    &local_state.window_state,
                    WindowKind::MixerMatrix,
                    "Mixer Matrix",
                    |ui| {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                let matrix = &store.get().project.mixer.matrix;
                                let row_titles: Vec<String> =
                                    (0..matrix.channels).map(|i| format!("Ch{i}")).collect();
                                let col_titles: Vec<String> = (0..matrix.channels)
                                    .map(|i| {
                                        if i == 0 {
                                            "Main in".to_string()
                                        } else {
                                            format!("Ch{i} in")
                                        }
                                    })
                                    .collect();
                                MixerMatrixView::new(
                                    matrix, row_titles, col_titles, store, on_release,
                                )
                                .ui(ui);
                            });
                            ui.add_space(8.0);
                        });
                        ui.add_space(8.0);
                    },
                );
                ui.separator();
                ui.columns(store.get().project.mixer.channels.len(), |columns| {
                    for (mixer_index, mixer) in
                        store.get().project.mixer.channels.iter().enumerate()
                    {
                        let mixer_sel = MixerSelector(mixer_index);
                        let dispatch_mixer = |action| store.dispatch(&mixer_sel, action);
                        columns[mixer_index].vertical(|ui| {
                            let heading = if mixer_index == 0 {
                                "Main"
                            } else {
                                &format!("Ch{mixer_index}")
                            };
                            ui.heading(heading);

                            let dispatch_volume = |it| {
                                dispatch_mixer(Action::SetFloat(FloatField::Volume, from_db(it)))
                            };
                            let mut level = to_db(mixer.volume);
                            ui.add(
                                Fader::stereo(&mut level, player.level(mixer_index))
                                    .rect_handle_shape(0.5)
                                    .text_size(12.0),
                            );
                            if level != to_db(mixer.volume) {
                                dispatch_volume(level)
                            }
                            let show_effect = local_state
                                .window_state
                                .get_visible(WindowKind::ChannelEffect);
                            ui.horizontal(|ui| {
                                if ui.add(Button::new("M").selected(mixer.mute)).clicked() {
                                    store.dispatch(
                                        &mixer_sel,
                                        Action::SetChild(TypeField::Mute(!mixer.mute)),
                                    );
                                }
                                if ui
                                    .add(
                                        Button::new("Effects").selected(
                                            show_effect && mixer_index == active_mixer_index,
                                        ),
                                    )
                                    .clicked()
                                {
                                    if mixer_index != active_mixer_index {
                                        // Update active mixer to what was just selected.
                                        local_state.active_mixer_channel.set(Some(mixer_sel));
                                        if !show_effect {
                                            local_state
                                                .window_state
                                                .set_visible(WindowKind::ChannelEffect, true);
                                        }
                                    } else {
                                        local_state
                                            .window_state
                                            .set_visible(WindowKind::ChannelEffect, !show_effect);
                                    }
                                };
                            });
                        });
                    }
                });
            },
        );
    }
}
