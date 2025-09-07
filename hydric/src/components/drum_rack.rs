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
    DrumTrackPlacement, GeneratorId, Placement, PlacementId, PlacementType, SamplePlacement, Track, TrackPlacement
};
use shared::types::Beats;
use state::{
    Action, PlacementSelector, SampleSelector, Store, TrackSelector, TypeField, UintField,
};

pub struct DrumRackView<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
}

impl<'a> DrumRackView<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState) -> Self {
        Self { store, local_state }
    }

    fn drum_rack_ui(
        ui: &mut Ui,
        placement_id: PlacementId,
        placement: &Placement,
        drum_placement: &DrumTrackPlacement,
        sel: &PlacementSelector,
        store: &Store,
    ) {

    }
}

impl View for DrumRackView<'_> {
    fn ui(&mut self, ui: &mut Ui) {

    }
}
