use crate::state::{Action, Store, get_set};
use crate::widget::selectable_value;
use egui::Ui;
use poll_promise::Promise;
use shared::model::Track;
use std::rc::Rc;

pub fn save_button(store: &Store, ui: &mut Ui) {
    if ui.button("Save").clicked() {
        store.dispatch(Action::SaveTrack { track_index: 0 });
    }
}

pub fn load_control(store: &Store, ui: &mut Ui) {
    {
        let save_track_promise: Rc<Option<Promise<Option<()>>>> =
            store.get().save_track_promise.clone();
        let promise_ref: &Option<Promise<Option<()>>> = save_track_promise.as_ref();
        if let Some(promise) = promise_ref {
            if let Some(Some(())) = promise.ready() {
                store.dispatch(Action::ClearSaveTrackPromise);

                // Once a track has been saved, re-load the list.
                store.dispatch(Action::LoadTrackList);
            }
        }
    }

    {
        let track_list_promise: Rc<Option<Promise<Option<Vec<String>>>>> =
            store.get().track_list_promise.clone();
        let promise_ref = track_list_promise.as_ref();
        if let Some(promise) = promise_ref {
            if let Some(Some(list)) = promise.ready() {
                store.dispatch(Action::ClearTrackListPromise);
                store.dispatch(Action::SetTrackList {
                    tracks: list.clone(),
                });
            }
        }
    }

    {
        let load_track_promise: Rc<Option<Promise<Option<Track>>>> =
            store.get().load_track_promise.clone();
        let promise_ref = load_track_promise.as_ref();
        if let Some(promise) = promise_ref {
            if let Some(Some(track)) = promise.ready() {
                store.dispatch(Action::ClearLoadTrackPromise);
                store.dispatch(Action::SetTrack {
                    track_index: 0,
                    track: track.clone(),
                });
            }
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
                            it.map(|it| {
                                store.dispatch(Action::SetLoadTrackName { track_name: it })
                            });
                        }),
                        Some(name.clone()),
                        name,
                    );
                }
            });
        // TODO disable button when no load_name
        if ui.button("Load").clicked() {
            store.dispatch(Action::LoadTrack);
        };
    });
}
