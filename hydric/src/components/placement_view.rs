use crate::components::utils::ToEguiColour;
use crate::view::View;
use crate::widget::{StateWindow, get_set, int_slider, selectable_value, slider};
use crate::window_state::WindowKind;
use crate::{GetSet, LocalState};
use egui::color_picker::Alpha;
use egui::{Ui, widgets::color_picker::color_picker_color32, Color32};
use mesic::samples_to_beats;
use ordered_float::OrderedFloat;
use shared::model::{
    DrumTrackId, DrumTrackPlacement, GeneratorId, Placement, PlacementId, PlacementType, SampleId,
    SamplePlacement, Track, TrackPlacement,
};
use shared::types::Beats;
use state::{
    Action, DrumTrackSelector, PlacementSelector, SampleSelector, Store, TrackSelector, TypeField,
    UintField,
};
use std::cmp::max;

pub struct PlacementView<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
}

impl<'a> PlacementView<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState) -> Self {
        Self { store, local_state }
    }

    fn track_placement_ui(
        &self,
        ui: &mut Ui,
        placement_id: PlacementId,
        placement: &Placement,
        track_placement: &TrackPlacement,
        sel: &PlacementSelector,
        store: &Store,
    ) {
        ui.horizontal(|ui| {
            ui.set_width(180.0);
            egui::ComboBox::from_id_salt(format!("placement_{:?}_track", placement_id))
            .selected_text(format!("Track ID {}", *track_placement.track_id))
            .show_ui(ui, |ui| {
                for track_id in store.get().project.tracks.keys() {
                    selectable_value(
                        ui,
                        get_set(&track_placement.track_id, |it| {
                            store.dispatch(sel, Action::SetChild(TypeField::TrackId(*it)))
                        }),
                        track_id,
                        track_id.to_string(),
                    );
                }
            });

        egui::ComboBox::from_id_salt(format!("placement_{:?}_generator", placement_id))
            .selected_text(self.get_generator_name(store, &track_placement.generator_id))
            .show_ui(ui, |ui| {
                let mut generators: Vec<_> = store.get().project.generators.keys().collect();
                if !generators.is_empty() {
                    generators.sort();
                    for generator_id in generators {
                        selectable_value(
                            ui,
                            get_set(&track_placement.generator_id, |it| {
                                store.dispatch(sel, Action::SetChild(TypeField::GeneratorId(*it)))
                            }),
                            generator_id,
                            self.get_generator_name(store, generator_id),
                        );
                    }
                }
            });
        });
        
        let track_sel = TrackSelector(track_placement.track_id);
        let track: &Track = store.select(&track_sel);
        let max_duration = *(track.unclipped_duration());
        let duration = *placement
            .clipped_duration
            .unwrap_or(OrderedFloat(max_duration));
        ui.add_space(5.0);
        Self::duration_ui(ui, sel, duration, max_duration, store);
    }

    fn get_generator_name(&self, store: &Store, generator_id: &GeneratorId) -> String {
        let mut generator_name = format!("Generator ID {}", **generator_id);
        if let Some(generator_instance) = store.get().project.generators.get(generator_id) {
            if !generator_instance.meta.name.is_empty() {
                generator_name = generator_instance.meta.name.clone();
            }
        }
        if self.store.get().project.generators.is_empty() {
            generator_name = "There are currently no generators".to_string();
        }
        generator_name
    }
    fn sample_placement_ui(
        ui: &mut Ui,
        placement_id: PlacementId,
        placement: &Placement,
        sample_placement: &SamplePlacement,
        sel: &PlacementSelector,
        store: &Store,
    ) {
        egui::ComboBox::from_id_salt(format!("placement_{:?}", placement_id))
            .selected_text(format!("Sample {:?}", sample_placement.sample_id))
            .show_ui(ui, |ui| {
                for sample_id in store.get().project.samples.keys() {
                    selectable_value(
                        ui,
                        get_set(&sample_placement.sample_id, |it| {
                            store.dispatch(sel, Action::SetChild(TypeField::SampleId(*it)))
                        }),
                        sample_id,
                        sample_id.to_string(),
                    );
                }
            });

        let sample_sel = SampleSelector(sample_placement.sample_id);
        let Some(sample) = store.try_select(&sample_sel) else {
            return;
        };
        let max_duration = samples_to_beats(
            max(sample.left.len(), sample.right.len()),
            store.get().project.bpm,
        );
        let duration = *placement
            .clipped_duration
            .unwrap_or(OrderedFloat(max_duration));
        Self::duration_ui(ui, sel, duration, max_duration, store);
    });
    
    }

    fn get_sample_name(store: &Store, sample_id: &SampleId) -> String {
        let mut sample_name = format!("Sample ID {}", **sample_id);
        if let Some(sample) = store.get().project.samples.get(sample_id) {
            sample_name = sample.sample_name.clone();
        }
        if store.get().project.samples.is_empty() {
            sample_name = "".to_string();
        }   sample_name
    }
 

    fn duration_ui(
        ui: &mut Ui,
        sel: &PlacementSelector,
        duration: f32,
        max_duration: f32,
        store: &Store,
    ) {
        let on_release = || store.dispatchr(Action::Release);
        let frame = egui::Frame::NONE
            .fill(Color32::from_gray(20))
            .corner_radius(5.0)
            .inner_margin(egui::Vec2::new(10.0, 5.0));

        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Clipped duration");
                    ui.add_space(90.0);
                });
                slider(
                    ui,
                    "",
                    duration as f64,
                    |it| {
                        store.dispatch(sel, {
                            let clipped_duration = if it < max_duration as f64 {
                                Some(it as Beats)
                            } else {
                                None
                            };
                            Action::SetChild(TypeField::ClippedDuration(clipped_duration))
                        })
                    },
                    0.0..=max_duration as f64,
                    on_release,
                );
            });
        });
    }

    fn drum_placement_ui(
        ui: &mut Ui,
        placement_id: PlacementId,
        _placement: &Placement,
        drum_placement: &DrumTrackPlacement,
        sel: &PlacementSelector,
        store: &Store,
        local_state: &'a LocalState,
    ) {
        let drum_sel = DrumTrackSelector(drum_placement.drum_track_id);
        let Some(_drum_track) = store.try_select(&drum_sel) else {
            ui.label("No drum tracks added yet");
            return;
        };
        egui::ComboBox::from_id_salt(format!("placement_{:?}", placement_id))
            .selected_text(format!(
                "{}",
                Self::get_drum_track_name(store, &drum_placement.drum_track_id)
            ))
            .show_ui(ui, |ui| {
                for drum_track_id in store.get().project.drum_tracks.keys() {
                    selectable_value(
                        ui,
                        get_set(&drum_placement.drum_track_id, |it| {
                            local_state
                                .active_drum_track
                                .set(Some(DrumTrackSelector(*it)));
                            store.dispatch(sel, Action::SetChild(TypeField::DrumTrackId(*it)))
                        }),
                        drum_track_id,
                        Self::get_drum_track_name(store, drum_track_id),
                    );
                }
            });
            ui.add_space(3.0);
    }

    fn get_drum_track_name(store: &Store, drum_track_id: &DrumTrackId) -> String {
        let mut drum_track_name = format!("Drum Track {}", **drum_track_id);
        if store.get().project.drum_tracks.is_empty() {
            drum_track_name = "".to_string();
        }
        drum_track_name
    }
}

impl View for PlacementView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let store = &self.store;
        let Some(placement_id): Option<PlacementId> = self.local_state.active_placement.get()
        else {
            return;
        };
        let on_release = || store.dispatchr(Action::Release);
        let placement = &store.get().project.placements[&placement_id];
        let sel = PlacementSelector(placement_id);
        let title = format!("Placement {:?}", *placement_id);
        StateWindow::show_from_window_state(
            ui,
            &self.local_state.window_state,
            WindowKind::Placement,
            &title,
            |ui| {
                ui.add_space(3.0);
                match &placement.kind {
                    PlacementType::Track(track_placement) => {
                        Self::track_placement_ui(
                            self,
                            ui,
                            placement_id,
                            placement,
                            track_placement,
                            &sel,
                            store,
                        );
                    }
                    PlacementType::Sample(sample_placement) => {
                        Self::sample_placement_ui(
                            ui,
                            placement_id,
                            placement,
                            sample_placement,
                            &sel,
                            store,
                        );
                    }
                    PlacementType::DrumTrack(drum_placement) => {
                        Self::drum_placement_ui(
                            ui,
                            placement_id,
                            placement,
                            drum_placement,
                            &sel,
                            store,
                            &self.local_state,
                        );
                    }
                }
                let frame = egui::Frame::NONE
                    .fill(Color32::from_gray(20))
                    .corner_radius(5.0)
                    .inner_margin(egui::Vec2::new(10.0, 5.0));
                 
                 frame.show(ui, |ui| {
                    ui.vertical(|ui| {
                        
                        ui.horizontal(|ui|{
                            ui.label("Visual placement");
                            ui.add_space(90.0);
                        });
                        int_slider(
                            ui,
                            "",
                            placement.visual_placement as f64,
                            |it| {
                                store.dispatch(&sel, Action::SetUint(UintField::VisualPlacement, it as u32))
                            },
                            0..=3,
                            on_release,
                        );
                    });
                });

                let initial_colour = placement.colour.to_egui();
                let mut new_colour = initial_colour;

                ui.add_space(15.0);
                ui.label("Placement Colour"); 
                ui.add_space(3.0);
                color_picker_color32(ui, &mut new_colour, Alpha::Opaque);

                if new_colour != initial_colour {
                    store.dispatch(
                        &sel,
                        Action::SetChild(state::TypeField::Colour(ToEguiColour::from_egui(
                            new_colour,
                        ))),
                    );
                }
            
                ui.add_space(2.0);
                if ui.button("Delete").clicked() {
                    store.dispatchr(Action::DeleteChildById(TypeField::PlacementId(
                        placement_id,
                    )));
                    self.local_state
                        .window_state
                        .set_visible(WindowKind::Placement, false);
                    self.local_state.active_placement.set(None);
                    self.local_state.selected_placements.update(|mut it| {
                        it.remove(&placement_id);
                        it
                    });
                }

                ui.add_space(2.0);
            },
        );
    }
}
