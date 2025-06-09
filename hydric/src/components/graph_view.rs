use crate::{LocalState, WindowKind, playback::AudioPlayer, view::View, widget::StateWindow};
use eframe::{App, CreationContext};
use egui::{Color32, Id, Ui, pos2};
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
    player: &'a AudioPlayer,
}

impl<'a> GraphView<'a> {
    pub fn new(
        local_state: &'a LocalState,
        snarl: &'a mut Snarl<GraphViewNode>,
        style: SnarlStyle,
        player: &'a AudioPlayer,
    ) -> Self {
        Self {
            local_state,
            snarl,
            style,
            player,
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
                if ui.button("Refresh").clicked() {
                    *self.snarl = Snarl::new();

                    if let Some(info) = self.player.get_graph_debug_info() {
                        for (node_index, label) in info.node_labels {
                            //log::info!("Label: {:?}", label);
                            self.snarl
                                .insert_node(pos2(0.0, 0.0), GraphViewNode { label, node_index });
                        }
                        for edge in info.edges {
                            //log::info!("Edge: {:?}", edge);
                        }
                    }
                }

                log::info!("{:?}", self.snarl);

                SnarlWidget::new()
                    .id(Id::new("graph-debug"))
                    .style(self.style)
                    .show(&mut self.snarl, &mut GraphViewer, ui);
            },
        );
    }
}

#[derive(Debug)]
pub struct GraphViewNode {
    node_index: usize,
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
