use crate::view::View;
use crate::widget::{StateWindow, get_set, int_slider, selectable_value, slider};
use crate::window_state::WindowKind;
use crate::{GetSet, LocalState};
use egui::Color32;
use egui::color_picker::Alpha;
use egui::{Ui, widgets::color_picker::color_picker_color32};
use mesic::samples_to_beats;
use ordered_float::OrderedFloat;
use shared::model::{
    Placement, PlacementId, PlacementType, SamplePlacement, Track, TrackPlacement, DrumPlacement,
};
use shared::types::Beats;
use state::{
    Action, PlacementSelector, SampleSelector, Store, TrackSelector, TypeField, UintField,
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
        ui: &mut Ui,
        placement_id: PlacementId,
        placement: &Placement,
        track_placement: &TrackPlacement,
        sel: &PlacementSelector,
        store: &Store,
    ) {
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
            .selected_text(format!("Generator ID {}", *track_placement.generator_id))
            .show_ui(ui, |ui| {
                let mut generators: Vec<_> = store.get().project.generators.keys().collect();
                generators.sort();

                for generator_id in generators {
                    selectable_value(
                        ui,
                        get_set(&track_placement.generator_id, |it| {
                            store.dispatch(sel, Action::SetChild(TypeField::GeneratorId(*it)))
                        }),
                        generator_id,
                        generator_id.to_string(),
                    );
                }
            });

        let track_sel = TrackSelector(track_placement.track_id);
        let track: &Track = store.select(&track_sel);
        let max_duration = *(track.unclipped_duration());
        let duration = *placement
            .clipped_duration
            .unwrap_or(OrderedFloat(max_duration));
        Self::duration_ui(ui, sel, duration, max_duration, store);
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
            ui.label("No samples loaded yet.");
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
    }

    fn drum_placement_ui(
        ui: &mut Ui,
        placement_id: PlacementId,
        placement: &Placement, 
        drum_placement: &DrumPlacement,
        sel: &PlacementSelector,
        store: &Store,
    ) {
        ui.label("Drum Placement");
        
        egui::ComboBox::from_id_salt(format!("placement_{:?}_drum_track", placement_id))
        .selected_text(format!("Track ID {}", *drum_placement.track_id))
        .show_ui(ui, |ui| {
            for track_id in store.get().project.tracks.keys() {
                selectable_value(
                    ui,
                    get_set(&drum_placement.track_id, |it| {
                        store.dispatch(sel, Action::SetChild(TypeField::TrackId(*it)))
                    }),
                    track_id,
                    track_id.to_string(),
                );
            }
        });

    }

    fn duration_ui(
        ui: &mut Ui,
        sel: &PlacementSelector,
        duration: f32,
        max_duration: f32,
        store: &Store,
    ) {
        let on_release = || store.dispatchr(Action::Release);

        ui.horizontal(|ui| {
            slider(
                ui,
                "Clipped duration",
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
        let title = format!("Placement {:?}", placement_id);
        StateWindow::show_from_window_state(
            ui,
            &self.local_state.window_state,
            WindowKind::Placement,
            &title,
            |ui| {
                match &placement.kind {
                    PlacementType::Track(track_placement) => {
                        Self::track_placement_ui(
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
                    PlacementType::Drum(drum_placement) => {
                        Self::drum_placement_ui(
                            ui,
                            placement_id,
                            placement,
                            drum_placement,
                            &sel,
                            store,
                        );
                    }
                }

                int_slider(
                    ui,
                    "Visual placement",
                    placement.visual_placement as f64,
                    |it| {
                        store.dispatch(&sel, Action::SetUint(UintField::VisualPlacement, it as u32))
                    },
                    0..=3,
                    on_release,
                );

                let rgb_initial = placement.colour.clone();
                let mut initial_colour =
                    Color32::from_rgb(rgb_initial[0], rgb_initial[1], rgb_initial[2]);

                ui.label("Placement Colour");
                color_picker_color32(ui, &mut initial_colour, Alpha::Opaque);

                let new_colour = [initial_colour.r(), initial_colour.g(), initial_colour.b()]; // initial colour gets modified by color picker
                if rgb_initial != new_colour {
                    let mut new_rgb = [0; 3];
                    for i in 0..3 {
                        new_rgb[i] = new_colour[i] as u32;
                    }
                    store.dispatch(
                        &sel,
                        Action::SetChildren(state::MultiTypeField::Colour(new_rgb)),
                    );
                }

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
            },
        );
    }
}
