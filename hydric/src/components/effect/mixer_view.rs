use super::MixerMatrixView;
use crate::GetSet;
use crate::local_state::LocalState;
use crate::playback::AudioPlayer;
use crate::view::View;
use crate::widget::StateWindow;
use crate::widget::int_slider;
use crate::window_state::WindowKind;
use egui::{Button, Ui};
use egui_fader::Fader;
use mesic::from_db;
use mesic::to_db;
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
        let mixer_sel = local_state
            .active_mixer_channel
            .get()
            .unwrap_or(MixerSelector(0));
        let MixerSelector(mixer_index) = mixer_sel;

        let mixer = &store.select(&mixer_sel);
        let dispatch_mixer = |action| store.dispatch(&mixer_sel, action);
        let on_release = || store.dispatchr(Action::Release);

        StateWindow::show_from_window_state(
            ui,
            &local_state.window_state,
            WindowKind::Mixer,
            "Mixer",
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
                        MixerMatrixView::new(matrix, row_titles, col_titles, store, on_release)
                            .ui(ui);
                    });
                });

                ui.add_space(8.0);

                // TODO: better UI than a slider for this!
                let max_channel_index = (store.get().project.mixer.channels.len() - 1) as i32;
                int_slider(
                    ui,
                    "Selected channel",
                    mixer_index as f64,
                    |it| {
                        local_state
                            .active_mixer_channel
                            .set(Some(MixerSelector(it as usize)))
                    },
                    0..=max_channel_index,
                    /* on_release= */
                    || {}, // no-op on_release since this doesn't use the store.
                );

                ui.separator();

                ui.horizontal(|ui| {
                    let heading = if mixer_index == 0 {
                        "Main channel"
                    } else {
                        &format!("Channel {mixer_index}")
                    };
                    ui.heading(heading);
                });

                ui.separator();
                let dispatch_volume =
                    |it| dispatch_mixer(Action::SetFloat(FloatField::Volume, from_db(it)));
                let mut level = to_db(mixer.volume);
                ui.add(
                    Fader::stereo(&mut level, player.level())
                        .rect_handle_shape(0.5)
                        .text_size(12.0),
                );
                if level != to_db(mixer.volume) {
                    dispatch_volume(level)
                }
                let show_effect = local_state
                    .window_state
                    .get_visible(WindowKind::ChannelEffect);
                if ui
                    .add(Button::new("Effects").selected(show_effect))
                    .clicked()
                {
                    local_state
                        .window_state
                        .set_visible(WindowKind::ChannelEffect, !show_effect);
                };
            },
        );
    }
}
