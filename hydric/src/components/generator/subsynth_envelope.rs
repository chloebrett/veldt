use crate::view::View;
use crate::widget::{TabDisplay, TabOrientation, knob};
use crate::transform::Transform;
use crate::DataState;
use egui::cache::{ComputerMut, FrameCache};
use egui::{
    Color32, Stroke, Vec2, Pos2, Rect, Ui,
    containers::Frame,
    emath::RectTransform,
    epaint::{PathStroke, Shape},
    pos2, vec2,
};
use log::info;
use ordered_float::OrderedFloat;
use shared::model::{AdsrEnvelope, SubSynthConfig};
use state::{Action, TypeField};

pub struct SubSynthEnvelopeView<'a, F: Fn(Action), G: Fn()> {
    config: &'a SubSynthConfig,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> SubSynthEnvelopeView<'a, F, G> {
    pub fn new(config: &'a SubSynthConfig, dispatch: F, on_release: G) -> Self {
        Self {
            config,
            dispatch,
            on_release,
        }
    }
}

impl <F: Fn(Action), G: Fn()> View for SubSynthEnvelopeView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let config = self.config.clone();
        let dispatch = &self.dispatch;
        let on_release = &self.on_release;

        let handle_env_tab_click: Box<dyn Fn(&mut egui::Ui, usize) + Send + Sync + 'static> =
                Box::new(move |ui, index| {
                    DataState::SubSynthEnvTab.set_value(ui, index);
                });

        let active_env_tab = DataState::SubSynthEnvTab
            .get_value::<usize>(ui)
            .unwrap_or_default();

        let outer_frame = Frame::new()
            .fill(Color32::from_rgb(50, 50, 50))
            .stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 50)))
            .corner_radius(8.0)
            .inner_margin(6.0);

        outer_frame.show(ui, |ui| {
            let original_spacing = ui.spacing().item_spacing; // store original spacing
            ui.spacing_mut().item_spacing = Vec2::ZERO; // set spacing to zero so that the tabs and associated content actually touch each other

            ui.horizontal(|ui| {
                TabDisplay::new(
                    active_env_tab,
                    vec!["ENV 1", "ENV 2", "ENV 3"],
                    TabOrientation::Left,
                    handle_env_tab_click,
                )
                .ui(ui);
                let inner_frame = Frame::new()
                    .fill(Color32::from_rgb(30, 30, 30))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(30, 30, 30)))
                    .corner_radius(8.0)
                    .inner_margin(15.0);
                inner_frame.show(ui, |ui| {
                    // TODO replace this section with an actual envelope visualisation
                    let size = egui::Vec2::new(200.0, 180.0);
                    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
                    let painter = ui.painter_at(rect);
                    let rect_shape = egui::Shape::rect_filled(rect, 5.0, Color32::RED);
                    painter.add(rect_shape);
                })
            });
            ui.spacing_mut().item_spacing = original_spacing; // reset ui spacing back to original
        });
    }
}
