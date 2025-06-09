use crate::{LocalState, WindowKind, view::View, widget::StateWindow};
use eframe::{App, CreationContext};
use egui::{Color32, Id, Ui};
use egui_snarl::{
    InPin, InPinId, NodeId, OutPin, OutPinId, Snarl,
    ui::{
        AnyPins, NodeLayout, PinInfo, PinPlacement, SnarlStyle, SnarlViewer, SnarlWidget, WireStyle,
    },
};
use mesic::NodeLabel;
use std::collections::HashMap;

pub struct GraphView<'a> {
    local_state: &'a LocalState,
    snarl: &'a mut Snarl<GraphViewNode>,
    style: SnarlStyle,
}

impl<'a> GraphView<'a> {
    pub fn new(
        local_state: &'a LocalState,
        snarl: &'a mut Snarl<GraphViewNode>,
        style: SnarlStyle,
    ) -> Self {
        Self {
            local_state,
            snarl,
            style,
        }
    }
}

impl View for GraphView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        StateWindow::show_from_window_state(
            ui,
            &self.local_state.window_state,
            WindowKind::GraphDebug,
            "Mixer graph debug",
            |ui| {
                SnarlWidget::new()
                    .id(Id::new("snarl-demo"))
                    .style(self.style)
                    .show(&mut self.snarl, &mut GraphViewer, ui);
            },
        );
    }
}

pub struct GraphViewNode {
    label: NodeLabel,
}

impl GraphViewNode {}

struct GraphViewer;

impl SnarlViewer<GraphViewNode> for GraphViewer {
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<GraphViewNode>) {
        return;
    }

    fn title(&mut self, node: &GraphViewNode) -> String {
        format!("{:?}", node.label)
    }

    fn inputs(&mut self, node: &GraphViewNode) -> usize {
        match node.label {
            NodeLabel::Generator | NodeLabel::Sample | NodeLabel::Buffer => 0,
            NodeLabel::Effect | NodeLabel::Amp => 1,
            NodeLabel::WetDry | NodeLabel::Sum => 2,
        }
    }

    fn outputs(&mut self, node: &GraphViewNode) -> usize {
        1
    }

    #[allow(refining_impl_trait)]
    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphViewNode>,
    ) -> PinInfo {
        PinInfo::circle().with_fill(Color32::GRAY)
    }

    #[allow(refining_impl_trait)]
    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphViewNode>,
    ) -> PinInfo {
        PinInfo::circle().with_fill(Color32::GRAY)
    }
}
