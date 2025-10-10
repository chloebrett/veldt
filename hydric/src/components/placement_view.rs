use crate::LocalState;
use crate::components::drum_placement_view::DrumPlacementView;
use crate::components::sample_placement_view::SamplePlacementView;
use crate::components::track_placement_view::TrackPlacementView;
use crate::components::utils::ToEguiColour;
use crate::local_state::GetSet;
use crate::view::View;
use crate::widget::frame::inner_frame_dark;
use crate::widget::{StateWindow, int_slider};
use crate::window_state::WindowKind;
use egui::color_picker::Alpha;
use egui::{Ui, widgets::color_picker::color_picker_color32};
use shared::model::{PlacementId, PlacementType};
use state::{Action, PlacementSelector, Store, TypeField, UintField};

pub struct PlacementView<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
    track_view: TrackPlacementView<'a>,
    sample_view: SamplePlacementView<'a>,
    drum_view: DrumPlacementView<'a>,
}

impl<'a> PlacementView<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState) -> Self {
        Self {
            store,
            local_state,
            track_view: TrackPlacementView::new(store),
            sample_view: SamplePlacementView::new(store),
            drum_view: DrumPlacementView::new(store),
        }
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
                ui.add_space(5.0);
                match &placement.kind {
                    PlacementType::Track(track_placement) => {
                        self.track_view
                            .ui(ui, placement_id, placement, track_placement, &sel);
                    }
                    PlacementType::Sample(sample_placement) => {
                        self.sample_view
                            .ui(ui, placement_id, placement, sample_placement, &sel);
                    }
                    PlacementType::DrumTrack(drum_placement) => {
                        self.drum_view.ui(ui, placement_id, drum_placement, &sel);
                    }
                }

                inner_frame_dark().show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Visual placement");
                            ui.add_space(95.0);
                        });

                        int_slider(
                            ui,
                            "",
                            placement.visual_placement as f64,
                            |it| {
                                store.dispatch(
                                    &sel,
                                    Action::SetUint(UintField::VisualPlacement, it as u32),
                                )
                            },
                            0..=self.local_state.visual_placement_rows.get() as i32 - 1,
                            on_release,
                        );
                    });
                });

                let initial_colour = placement.colour.to_egui();
                let mut new_colour = initial_colour;

                ui.add_space(5.0);
                inner_frame_dark().show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label("Placement Colour");
                        ui.add_space(5.0);
                        color_picker_color32(ui, &mut new_colour, Alpha::Opaque);
                    });
                });

                if new_colour != initial_colour {
                    store.dispatch(
                        &sel,
                        Action::SetChild(TypeField::Colour(ToEguiColour::from_egui(new_colour))),
                    );
                }

                ui.add_space(5.0);
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
