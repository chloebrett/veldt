use crate::components::{PlacedTrack, TrackSequencer};
use crate::playback::AudioPlayer;
use crate::{GetSet, LocalState};
use crate::{view::View, widget::StateWindow, window_state::WindowKind};
use egui::{Color32, Pos2, Rect, ScrollArea, Ui, pos2, vec2};
use mesic::samples_to_beats;
use ordered_float::OrderedFloat;
use shared::model::DrumTrackPlacement;
use shared::model::{
    Colour, Placement, PlacementId, PlacementType, SamplePlacement, Track, TrackPlacement,
};
use state::{Action, SampleSelector, Store, TypeField};
use std::cmp::max;
use std::collections::{HashMap, HashSet};

pub struct TrackRoll<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
    player: &'a mut AudioPlayer,
}

impl<'a> TrackRoll<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState, player: &'a mut AudioPlayer) -> Self {
        Self {
            store,
            local_state,
            player,
        }
    }
}

impl View for TrackRoll<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let store = self.store;
        let project = &store.get().project;

        // TODO: rename PlacedTrack to something encompassing both tracks and samples.
        // They can be a single object, but just contain a tag/enum specifying which one they are.
        let placed_tracks: HashMap<PlacementId, PlacedTrack> = project
            .placements
            .clone()
            .into_iter()
            .map(|(id, placement)| {
                (
                    id,
                    match placement.kind {
                        PlacementType::Track(TrackPlacement { track_id, .. }) => PlacedTrack {
                            unclipped_duration: project.tracks[&track_id].unclipped_duration(),
                            placement: placement.clone(),
                            store: self.store,
                            local_state: self.local_state,
                        },
                        PlacementType::Sample(SamplePlacement { sample_id }) => {
                            let duration = store
                                .try_select(&SampleSelector(sample_id))
                                .map(|sample| {
                                    samples_to_beats(
                                        max(sample.left.len(), sample.right.len()),
                                        store.get().project.bpm,
                                    )
                                })
                                .unwrap_or(1.0);
                            PlacedTrack {
                                unclipped_duration: duration.into(),
                                placement: placement.clone(),
                                store: self.store,
                                local_state: self.local_state,
                            }
                        }
                        PlacementType::DrumTrack(DrumTrackPlacement { track_id, sample_id }) => {
                            let max_offset = project.tracks[&track_id]
                                .notes
                                .iter()
                                .max_by_key(|placed_note| placed_note.offset)
                                .map(|last_note| last_note.offset.into())
                                .unwrap_or(0.0);
                            let sample_duration = store
                                .try_select(&SampleSelector(sample_id))
                                .map(|sample| {
                                    samples_to_beats(
                                        max(sample.left.len(), sample.right.len()),
                                        store.get().project.bpm,
                                    )
                                })
                                .unwrap_or(0.5);
                             // TODO: currently multiplying the sample duration by 2 to account for if the sample is pitched lower (assuming tuning approach will change duration), find a better way to do this
                            let duration = OrderedFloat(max_offset + sample_duration * 2.0);
                            PlacedTrack {
                                unclipped_duration: duration,
                                placement: placement.clone(),
                                store: self.store,
                                local_state: self.local_state,
                            }
                        }
                    },
                )
            })
            .collect();

        let min_rows = 4;
        let max_visual_placement = max(
            project
                .placements
                .values()
                .map(|it| it.visual_placement)
                .max()
                .unwrap_or(0),
            min_rows,
        );

        let mut select = self.local_state.track_roll_select_enabled.get();
        if !select {
            self.local_state.selected_placements.set(HashSet::default());
        }
        StateWindow::show_from_window_state_resizable(
            ui,
            &self.local_state.window_state,
            WindowKind::TrackRoll,
            "Track Roll",
            |ui| {
                ui.horizontal(|ui| {
                    if ui.button("New track/drum sequence").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::Track(Track::default())));
                    }
                    if ui.button("New track placement").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::Placement(Placement {
                            kind: PlacementType::Track(TrackPlacement::default()),
                            offset: 0.0.into(),
                            clipped_duration: None,
                            visual_placement: 0,
                            colour: Colour::from_8bit(67, 206, 222),
                        })));
                    }
                    if ui.button("New drum placement").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::Placement(Placement {
                            kind: PlacementType::DrumTrack(DrumTrackPlacement::default()),
                            offset: 0.0.into(),
                            clipped_duration: None,
                            visual_placement: 0,
                            colour: Colour::from_8bit(102, 67, 0),
                        })));
                    }
                    if ui.button("New sample placement").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::Placement(Placement {
                            kind: PlacementType::Sample(SamplePlacement::default()),
                            offset: 0.0.into(),
                            clipped_duration: None,
                            visual_placement: 0,
                            colour: Colour::from_8bit(71, 44, 114),
                        })));
                    }
                    ui.checkbox(&mut select, "Select")
                });
                ui.separator();
                let window_size = ui.available_size();
                const PADDING_AROUND_TRACK_SQUENCER: f32 = 6.0;
                const MINIMUM_SIZE: f32 = 600.0;
                const INCREMENT_SIZE: f32 = 37.5;
                let range = Rect::from_min_max(
                    Pos2::ZERO,
                    pos2(
                        ((window_size.x - PADDING_AROUND_TRACK_SQUENCER).max(MINIMUM_SIZE)
                            / INCREMENT_SIZE)
                            .ceil(),
                        max_visual_placement as f32,
                    ),
                );

                ScrollArea::vertical()
                    .min_scrolled_height(400.0)
                    .show(ui, |ui| {
                        ui.add(
                            TrackSequencer::new(store, self.local_state, range, self.player)
                                .objects(placed_tracks)
                                .size(vec2(
                                    (window_size.x - 6.0).max(600.0),
                                    100.0 * max_visual_placement as f32,
                                ))
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
