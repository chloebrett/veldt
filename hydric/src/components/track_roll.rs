use crate::{
    GetSet, LocalState,
    view::View,
    widget::{Sequencer, SequencerObject, StateWindow, default_window},
    window_state::{WindowKind, WindowState2},
};
use egui::{
    Color32, CornerRadius, Pos2, Rect, ScrollArea, Shape, Stroke, StrokeKind, Ui, pos2, vec2,
};
use ordered_float::OrderedFloat;
use shared::{
    model::{Placement, PlacementType, SamplePlacement, Track, TrackPlacement},
    types::Beats,
};
use state::{
    Action, FloatField, MultiIndexField, PlacementSelector, SelectorTrait, Store, TrackSelector,
    TypeField, UintField,
};
use std::cmp::max;
use std::collections::HashSet;

pub struct TrackRoll<'a> {
    store: &'a Store,
    window_state: &'a mut WindowState2,
    local_state: &'a LocalState,
}

impl<'a> TrackRoll<'a> {
    pub fn new(
        store: &'a Store,
        window_state: &'a mut WindowState2,
        local_state: &'a LocalState,
    ) -> Self {
        Self {
            store,
            window_state,
            local_state,
        }
    }
}

impl View for TrackRoll<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let store = self.store;
        let project = &store.get().project;

        // TODO: rename PlacedTrack to something encompassing both tracks and samples.
        // They can be a single object, but just contain a tag/enum specifying which one they are.
        let placed_tracks: Vec<PlacedTrack> = project
            .placements
            .iter()
            .map(|placement| match placement.kind {
                PlacementType::Track(TrackPlacement { track_index, .. }) => PlacedTrack {
                    unclipped_duration: project.tracks[track_index].unclipped_duration(),
                    placement: placement.clone(),
                },
                PlacementType::Sample(SamplePlacement { .. }) => PlacedTrack {
                    // TODO: use real sample duration.
                    unclipped_duration: 1.0.into(),
                    placement: placement.clone(),
                },
            })
            .collect();

        let min_rows = 4;
        let max_visual_placement = max(
            project
                .placements
                .iter()
                .map(|it| it.visual_placement)
                .max()
                .unwrap_or(0),
            min_rows,
        );
        let range = Rect::from_min_max(Pos2::ZERO, pos2(16.0, max_visual_placement as f32));
        let mut select = self.local_state.track_roll_select_enabled.get();
        if !select {
            self.local_state.selected_placements.set(HashSet::default());
        }

        let window = StateWindow(
            default_window("Track Roll")
                .default_pos(self.window_state.get_pos(WindowKind::TrackRoll))
                .resizable(true),
        );
        window.show_with_closure(
            ui,
            self.window_state.get_visible(WindowKind::TrackRoll),
            |_| self.window_state.set_visible(WindowKind::TrackRoll, false),
            |ui| {
                ui.horizontal(|ui| {
                    if ui.button("New track").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::Track(Track::default())));
                    }
                    if ui.button("New track placement").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::Placement(
                            Placement::default(),
                        )));
                    }
                    if ui.button("New sample placement").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::Placement(Placement {
                            kind: PlacementType::Sample(SamplePlacement::default()),
                            offset: 0.0.into(),
                            clipped_duration: None,
                            visual_placement: 0,
                        })));
                    }
                    ui.checkbox(&mut select, "Select")
                });
                ui.separator();
                ScrollArea::vertical()
                    .min_scrolled_height(400.0)
                    .show(ui, |ui| {
                        ui.add(
                            Sequencer::new(store, self.local_state, range)
                                .objects(placed_tracks)
                                .size(vec2(600.0, 100.0 * max_visual_placement as f32))
                                .select(select)
                                .vertical_bars(4.0, Color32::from_white_alpha(6))
                                .vertical_bars(1.0, Color32::from_white_alpha(3))
                                .horizontal_rects(
                                    |index| index % 2 == 1,
                                    Color32::from_white_alpha(1),
                                ),
                        );
                    });
            },
        );
        self.local_state.track_roll_select_enabled.set(select);
    }
}

struct PlacedTrack {
    placement: Placement,
    unclipped_duration: OrderedFloat<f32>,
}

impl SequencerObject<PlacedTrack> for PlacedTrack {
    fn to_pos(&self, range: Rect) -> Pos2 {
        let y = self.placement.visual_placement as f32;
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
        Some(Action::SetUint(UintField::VisualPlacement, y as u32))
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

    fn get_active(store: &Store, local_state: &LocalState) -> Option<PlacedTrack> {
        let index = local_state.active_placement.get()?;
        let selector = PlacementSelector(index);
        let placement = store.select(&selector);
        let track_placement: Option<&TrackPlacement> = placement.try_into().ok();
        Some(PlacedTrack {
            // TODO: sample duration
            unclipped_duration: track_placement
                .map(|it| {
                    store
                        .select(&TrackSelector(it.track_index))
                        .unclipped_duration()
                })
                .unwrap_or(1.0.into()),
            placement: placement.clone(),
        })
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

    // TODO: remove all these unused _ui params.
    fn get_selected(_ui: &Ui, store: &Store, local_state: &LocalState) -> Vec<PlacedTrack> {
        local_state
            .selected_placements
            .get()
            .into_iter()
            .map(|index| {
                let placement_sel = PlacementSelector(index);
                let placement = store.select(&placement_sel);
                let track_placement: &TrackPlacement = placement.try_into().unwrap();
                PlacedTrack {
                    unclipped_duration: store
                        .select(&TrackSelector(track_placement.track_index))
                        .unclipped_duration(),
                    placement: placement.clone(),
                }
            })
            .collect()
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

    fn selector(index: usize, _parent_index: Option<usize>) -> impl SelectorTrait {
        PlacementSelector(index)
    }

    fn set_active(&self, local_state: &LocalState, index: usize) {
        let track_placement: Option<&TrackPlacement> = (&self.placement).try_into().ok();

        local_state.note_roll_window.set(true);
        local_state.placement_window.set(true);
        local_state
            .active_track
            .set(track_placement.map(|it| TrackSelector(it.track_index)));
        local_state.active_placement.set(Some(index));
    }

    fn set_selected(local_state: &LocalState, index: Option<usize>) {
        let Some(index) = index else {
            local_state.selected_placements.set(HashSet::default());
            return;
        };

        local_state.selected_placements.update(|mut it| {
            if it.contains(&index) {
                it.remove(&index);
            } else {
                it.insert(index);
            }
            it
        });
    }

    fn add_new(&self, store: &Store, _parent_index: Option<usize>) {
        store.dispatchr(Action::AddChild(TypeField::Placement(
            self.placement.clone(),
        )));
    }

    fn from_pos(pos: Pos2, range: Rect) -> PlacedTrack {
        let track_index = pos.y as usize;
        let offset = range.left() + pos.x;
        PlacedTrack {
            placement: Placement {
                kind: PlacementType::Track(TrackPlacement {
                    track_index,
                    generator_index: 0,
                }),
                offset: offset.into(),
                clipped_duration: None,
                visual_placement: 0,
            },
            unclipped_duration: 0.0.into(),
        }
    }

    fn delete_selected(store: &Store, local_state: &LocalState, _parent_index: Option<usize>) {
        local_state
            .active_placement
            .update(|placement| match placement {
                // If the active placement is selected, "de-activate" it.
                Some(index) if local_state.selected_placements.get().contains(&index) => None,
                _ => placement,
            });
        store.dispatchr(Action::DeleteChildren(MultiIndexField::Placement(
            local_state.selected_placements.get().into_iter().collect(),
        )));
    }
}
