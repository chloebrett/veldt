use std::collections::BTreeSet;

use crate::{
    app_state::{DataState, WindowState, update_select_data_state},
    view::View,
    widget::{Sequencer, SequencerObject, default_window},
};
use egui::{
    Color32, CornerRadius, Pos2, Rect, ScrollArea, Shape, Stroke, StrokeKind, Ui, pos2, vec2,
};
use ordered_float::OrderedFloat;
use shared::{
    model::{Track, TrackId, TrackPlacement},
    types::Beats,
};
use state::{Action, FloatField, Selector, Store, TypeField, UintField};

pub struct TrackRoll<'a> {
    store: &'a Store,
    window_state: &'a mut WindowState,
}

impl<'a> TrackRoll<'a> {
    pub fn new(store: &'a Store, window_state: &'a mut WindowState) -> Self {
        TrackRoll {
            store,
            window_state,
        }
    }
}

impl View for TrackRoll<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let store = self.store;
        let default_track = Track {
            notes: vec![],
            offset: 0.0.into(),
        };
        let default_track_placement = TrackPlacement {
            track_id: 0 as TrackId,
            offset: (0.0 as Beats).into(),
            clipped_duration: None,
            visual_placement: 0,
        };
        let placed_tracks: Vec<PlacedTrack> = store
            .get()
            .project
            .track_placements
            .iter()
            .map(|placement| PlacedTrack {
                unclipped_duration: store.get().project.tracks[placement.track_id as usize]
                    .unclipped_duration(),
                placement: placement.clone(),
            })
            .collect();
        let track_count = store.get().project.tracks.len();
        let range = Rect::from_min_max(pos2(0.0, 0.0), pos2(16.0, track_count as f32));
        let mut select = DataState::TrackRollSelectMode
            .get_value(ui)
            .unwrap_or(false);
        if !select {
            DataState::SelectedTrackPlacementIndexes.remove_value(ui);
        }
        default_window("Track Roll")
            .default_pos(pos2(30.0, 200.0))
            .resizable(true)
            .open(&mut self.window_state.track_roll)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("New track").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::Track(default_track)));
                    }
                    if ui.button("New track placement").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::TrackPlacement(
                            default_track_placement,
                        )));
                    }
                    ui.checkbox(&mut select, "Select")
                });
                ScrollArea::vertical()
                    .min_scrolled_height(400.0)
                    .show(ui, |ui| {
                        ui.add(
                            Sequencer::new(store, range)
                                .objects(placed_tracks)
                                .size(vec2(600.0, 100.0 * track_count as f32))
                                .select(select)
                                .vertical_bars(4.0, Color32::from_white_alpha(6))
                                .vertical_bars(1.0, Color32::from_white_alpha(3))
                                .horizontal_rects(
                                    |index| index % 2 == 1,
                                    Color32::from_white_alpha(1),
                                ),
                        );
                    });
            });
        DataState::TrackRollSelectMode.set_value(ui, select);
    }
}

struct PlacedTrack {
    placement: TrackPlacement,
    unclipped_duration: OrderedFloat<f32>,
}

impl SequencerObject<PlacedTrack> for PlacedTrack {
    fn to_pos(&self, range: Rect) -> Pos2 {
        // TODO handling multiple channels. Currently all are at `y=0`.
        let y = self.placement.track_id as f32;
        let x = *self.placement.offset - range.left();
        pos2(x, y)
    }

    fn to_pos_horizontal(&self, range: Rect) -> Pos2 {
        let y = self.placement.track_id as f32;
        let x = *self.placement.offset - range.left();
        pos2(x, y)
    }

    fn to_rect(&self, range: Rect) -> Rect {
        let track_pos = self.to_pos(range);
        // If not clipped duration render length based on notes.
        let length: f32 = *self
            .placement
            .clipped_duration
            .unwrap_or(self.unclipped_duration);
        // Min `track_size.x` of 0.4 to ensure part of the object is still visible to interact with.
        let track_size = vec2(length.max(0.4), 1.0);
        Rect::from_min_size(track_pos, track_size)
    }

    fn x_action(&self, x: f32, range: Rect) -> Option<Action> {
        Some(Action::SetFloat(FloatField::Offset, x - range.left()))
    }

    fn y_action(&self, y: f32, _range: Rect) -> Option<Action> {
        // TODO implement multiple tracks.
        Some(Action::SetUint(UintField::TrackId, y as u32))
    }

    fn resize_action(&self, x: f32, _range: Rect) -> Option<Action> {
        let clipped_duration = x - *self.placement.offset;
        let max_note_length = *self.unclipped_duration;
        let clipped_duration = if clipped_duration < max_note_length {
            Some(clipped_duration as Beats)
        } else {
            None
        };
        Some(Action::SetChild(TypeField::ClippedDuration(
            clipped_duration,
        )))
    }

    fn shape(&self, range: Rect) -> Shape {
        if *self.unclipped_duration == 0.0 {
            Shape::rect_filled(
                self.to_rect(range),
                CornerRadius::same(1),
                Color32::from_white_alpha(32),
            )
        } else {
            Shape::rect_filled(self.to_rect(range), CornerRadius::same(1), Color32::WHITE)
        }
    }

    fn get_active(ui: &Ui, store: &Store) -> Option<PlacedTrack> {
        if let Some(index) = DataState::ActiveTrackPlacementIndex.get_value::<usize>(ui) {
            let track_placement = store
                .get()
                .project
                .track_placements
                .get(index)
                .expect("Should have been track placement at index");
            Some(PlacedTrack {
                unclipped_duration: store
                    .get()
                    .project
                    .tracks
                    .get(track_placement.track_id as usize)
                    .expect("Should have been track at index.")
                    .unclipped_duration(),
                placement: track_placement.clone(),
            })
        } else {
            None
        }
    }

    fn active_shape(&self, range: Rect) -> Shape {
        Shape::Vec(vec![
            self.shape(range),
            Shape::rect_stroke(
                self.to_rect(range),
                CornerRadius::same(0),
                Stroke {
                    width: 1.0,
                    color: Color32::BLUE,
                },
                StrokeKind::Inside,
            ),
        ])
    }

    fn get_selected(ui: &Ui, store: &Store) -> Option<Vec<PlacedTrack>> {
        let index_list: BTreeSet<usize> = DataState::SelectedTrackPlacementIndexes.get_value(ui)?;
        Some(
            index_list
                .into_iter()
                .map(|index| {
                    let track_placement = store
                        .get()
                        .project
                        .track_placements
                        .get(index)
                        .expect("Should have been track placement at index");
                    PlacedTrack {
                        unclipped_duration: store
                            .get()
                            .project
                            .tracks
                            .get(track_placement.track_id as usize)
                            .expect("Should have been track at index.")
                            .unclipped_duration(),
                        placement: track_placement.clone(),
                    }
                })
                .collect(),
        )
    }

    fn selected_shape(&self, range: Rect) -> Shape {
        Shape::Vec(vec![
            self.shape(range),
            Shape::rect_stroke(
                self.to_rect(range),
                CornerRadius::same(0),
                Stroke {
                    width: 1.0,
                    color: Color32::RED,
                },
                StrokeKind::Inside,
            ),
        ])
    }

    fn selector(index: usize, _parent_index: Option<usize>) -> Selector {
        Selector::TrackPlacement(index)
    }

    fn set_active(&self, ui: &mut Ui, index: usize) {
        DataState::NoteRollWindow.set_value(ui, true);
        DataState::TrackPlacementViewWindow.set_value(ui, true);
        DataState::ActiveTrackIndex.set_value(ui, self.placement.track_id as usize);
        DataState::ActiveTrackPlacementIndex.set_value(ui, index);
    }

    fn set_selected(ui: &mut Ui, index: Option<usize>) {
        update_select_data_state(ui, DataState::SelectedTrackPlacementIndexes, index);
    }

    fn add_new(&self, store: &Store, _parent_index: Option<usize>) {
        store.dispatchr(Action::AddChild(TypeField::TrackPlacement(
            self.placement.clone(),
        )));
    }

    fn from_pos(pos: Pos2, range: Rect) -> PlacedTrack {
        let track_index = pos.y as u32;
        let offset = range.left() + pos.x;
        PlacedTrack {
            placement: TrackPlacement {
                track_id: track_index,
                offset: offset.into(),
                clipped_duration: None,
                visual_placement: 0,
            },
            unclipped_duration: 0.0.into(),
        }
    }

    fn delete(store: &Store, index: usize, _parent_index: Option<usize>) {
        store.dispatchr(Action::DeleteChild(state::IndexField::TrackPlacement(
            index,
        )));
    }

    fn delete_selected(ui: &mut Ui, store: &Store, parent_index: Option<usize>) {
        for index in DataState::SelectedTrackPlacementIndexes
            .get_value::<BTreeSet<usize>>(ui)
            .unwrap_or_default()
            .iter()
            .rev()
        {
            PlacedTrack::delete(store, *index, parent_index);
        }
    }
}
