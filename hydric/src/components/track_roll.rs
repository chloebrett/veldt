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
                        PlacementType::DrumTrack(ref drum_track_placement) => PlacedTrack {
                            unclipped_duration: drum_track_placement
                                .duration(&self.store.get().project),
                            placement: placement,
                            store: self.store,
                            local_state: self.local_state,
                        },
                    },
                )
            })
            .collect();

        // determine how many rows need to be displayed
        let max_visual_placement =  if self.local_state.visual_placement_rows.get() == 0 { 
            max(
            project
                .placements
                .values()
                .map(|it| it.visual_placement)
                .max()
                .unwrap_or(0) + 1,
            4,
        )
        } else { self.local_state.visual_placement_rows.get() };
        self.local_state.visual_placement_rows.set(max_visual_placement);

        // calculate how wide the track sequencer needs to be
        const MINIMUM_BEATS: OrderedFloat<f32> = OrderedFloat(20.0);
        const EXTRA_BEATS_FOR_PADDING: OrderedFloat<f32> = OrderedFloat(2.0);

        let extra_beats = self.local_state.extra_track_roll_beats.get();
        let beats_to_display = placed_tracks.values().map(|it| it.unclipped_duration + it.placement.offset + EXTRA_BEATS_FOR_PADDING).max().unwrap_or(MINIMUM_BEATS) + OrderedFloat(extra_beats);

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
                    if ui.button("Add Row").clicked() {
                        let num_curr_rows = self.local_state.visual_placement_rows.get();
                        self.local_state.visual_placement_rows.set(num_curr_rows + 1);
                    }
                    if ui.button("Add Bar").clicked() {
                        let num_curr_extra_beats = self.local_state.extra_track_roll_beats.get();
                        self.local_state.extra_track_roll_beats.set(num_curr_extra_beats + 4.0);
                    }
                    ui.checkbox(&mut select, "Select")
                });
                ui.separator();
                let window_size = ui.available_size();
                const PADDING_AROUND_TRACK_SQUENCER: f32 = 6.0;
                const MINIMUM_SIZE: f32 = 600.0;
                const BEAT_INCREMENT_SIZE: f32 = 37.5;
                let range = Rect::from_min_max(
                    Pos2::ZERO,
                    pos2(
                        (((window_size.x - PADDING_AROUND_TRACK_SQUENCER) / BEAT_INCREMENT_SIZE) + extra_beats).max(*beats_to_display),
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
                                    (window_size.x - PADDING_AROUND_TRACK_SQUENCER + extra_beats * BEAT_INCREMENT_SIZE).max(*beats_to_display * BEAT_INCREMENT_SIZE),
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
