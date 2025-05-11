use crate::view::View;
use crate::widget::{StateWindow, default_window, get_set, int_slider, selectable_value, slider};
use crate::{GetSet, LocalState};
use egui::{Ui, pos2};
use shared::model::SamplePlacement;
use shared::types::Beats;
use state::{Action, FloatField, IndexField, PlacementSelector, Store, TypeField, UintField};

pub struct SamplePlacementView<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
}

impl<'a> SamplePlacementView<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState) -> Self {
        Self { store, local_state }
    }
}

// TODO: consider combining sample_placement_view and track_placement_view?
// They have many similar options.
impl View for SamplePlacementView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let store = &mut self.store;
        // TODO: rename active_track_placement to active_placement.
        let Some(placement_index): Option<usize> = self.local_state.active_track_placement.get()
        else {
            return;
        };
        let on_release = || store.dispatchr(Action::Release);
        let placement = &store.get().project.placements[placement_index];
        let Some(sample_placement): Option<&SamplePlacement> = placement.try_into().ok() else {
            return;
        };
        let samples_length = store.get().project.samples.len();
        let sel = PlacementSelector(placement_index);
        let title = format!("Placement {placement_index}");

        let window = StateWindow(
            default_window(&title)
                .default_pos(pos2(100.0, 20.0))
                .resizable(true),
        );
        window.show_with_closure(
            ui,
            self.local_state.track_placement_window.get(),
            |_| self.local_state.track_placement_window.set(false),
            |ui| {
                egui::ComboBox::from_id_salt(format!("placement_{placement_index}"))
                    .selected_text(format!("Sample {}", sample_placement.sample_index))
                    .show_ui(ui, |ui| {
                        for sample_index in 0..samples_length {
                            selectable_value(
                                ui,
                                get_set(&sample_placement.sample_index, |it| {
                                    store.dispatch(&sel, Action::SetIndex(IndexField::Sample(*it)))
                                }),
                                &sample_index,
                                sample_index.to_string(),
                            );
                        }
                    });

                let offset = *placement.offset as f64;
                slider(
                    ui,
                    "Start position",
                    offset,
                    |it| store.dispatch(&sel, Action::SetFloat(FloatField::Offset, it as Beats)),
                    0.0..=16.0,
                    on_release,
                );

                // TODO: use real sample duration.
                let unclipped_duration = 1.0;
                ui.horizontal(|ui| {
                    slider(
                        ui,
                        "Clipped duration",
                        unclipped_duration,
                        |it| {
                            store.dispatch(&sel, {
                                let clipped_duration = if it < unclipped_duration as f64 {
                                    Some(it as Beats)
                                } else {
                                    None
                                };
                                Action::SetChild(TypeField::ClippedDuration(clipped_duration))
                            })
                        },
                        0.0..=unclipped_duration as f64,
                        on_release,
                    );
                });

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

                if ui.button("Delete").clicked() {
                    store.dispatchr(Action::DeleteChild(IndexField::Placement(placement_index)));
                    self.local_state.track_placement_window.set(false);
                    self.local_state.active_track_placement.set(None);
                    self.local_state.selected_track_placements.update(|mut it| {
                        it.remove(&placement_index);
                        it
                    });
                }
            },
        );
    }
}
