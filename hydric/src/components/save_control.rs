use super::AsyncState;
use crate::rpc::{load_track, load_track_list, save_track};
use crate::widget::selectable_value;
use egui::Ui;
use poll_promise::Promise;
use shared::model::Track;
use state::{Action, Store, get_set};

pub fn save_button(store: &Store, async_state: &mut AsyncState, ui: &mut Ui) {
    if ui.button("Save").clicked() {
        let track_index = 0;
        let track_name = store.get().project.name.clone();
        let track = store.get().project.tracks[track_index].clone();
        async_state.save_track = Some(Promise::spawn_local(async move {
            save_track(track_name, track).await
        }));
    }
}

pub fn load_control(store: &Store, async_state: &mut AsyncState, ui: &mut Ui) {
    {
        let save_track_promise: &Option<Promise<Option<()>>> = &async_state.save_track;
        let promise_ref: Option<&Promise<Option<()>>> = save_track_promise.as_ref();
        if let Some(promise) = promise_ref {
            if let Some(Some(())) = promise.ready() {
                async_state.save_track = None;

                // Once a track has been saved, re-load the list.
                async_state.track_list =
                    Some(Promise::spawn_local(async move { load_track_list().await }));
            }
        }
    }

    {
        let track_list_promise: &Option<Promise<Option<Vec<String>>>> = &async_state.track_list;
        let promise_ref = track_list_promise.as_ref();
        let mut clear = false;
        if let Some(promise) = promise_ref {
            if let Some(Some(list)) = promise.ready() {
                clear = true;

                store.dispatchr(Action::SetTrackList {
                    tracks: list.clone(),
                });
            }
        }
        if clear {
            async_state.track_list = None;
        }
    }

    // TODO: generalise this promise handling logic.
    {
        let load_track_promise: &Option<Promise<Option<Track>>> = &async_state.load_track;
        let promise_ref = load_track_promise.as_ref();
        let mut clear = false;
        if let Some(promise) = promise_ref {
            if let Some(Some(track)) = promise.ready() {
                clear = true;

                store.dispatchr(Action::SetTrack {
                    track_index: 0,
                    track: track.clone(),
                });
            }
        }
        if clear {
            async_state.load_track = None;
        }
    }

    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt(1) // TODO Correct Id Salt
            .selected_text(
                store
                    .get()
                    .load_track_name
                    .clone()
                    .unwrap_or("".to_string())
                    .to_string(),
            )
            .show_ui(ui, |ui| {
                for name in store.get().track_list.iter() {
                    selectable_value(
                        ui,
                        get_set(store.get().load_track_name.clone(), |it| {
                            if let Some(it) = it {
                                store.dispatchr(Action::SetLoadTrackName { track_name: it });
                            }
                        }),
                        Some(name.clone()),
                        name,
                    );
                }
            });
        // TODO disable button when no load_name
        if ui.button("Load").clicked() {
            if let Some(name) = store.get().load_track_name.clone() {
                async_state.load_track =
                    Some(Promise::spawn_local(async move { load_track(name).await }));
            }
        };
    });
}
