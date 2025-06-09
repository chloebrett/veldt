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

const fn default_style() -> SnarlStyle {
    SnarlStyle {
        node_layout: Some(NodeLayout::coil()),
        pin_placement: Some(PinPlacement::Edge),
        pin_size: Some(7.0),
        node_frame: Some(egui::Frame {
            inner_margin: egui::Margin::same(8),
            outer_margin: egui::Margin {
                left: 0,
                right: 0,
                top: 0,
                bottom: 4,
            },
            corner_radius: egui::CornerRadius::same(8),
            fill: egui::Color32::from_gray(30),
            stroke: egui::Stroke::NONE,
            shadow: egui::Shadow::NONE,
        }),
        bg_frame: Some(egui::Frame {
            inner_margin: egui::Margin::ZERO,
            outer_margin: egui::Margin::same(2),
            corner_radius: egui::CornerRadius::ZERO,
            fill: egui::Color32::from_gray(40),
            stroke: egui::Stroke::NONE,
            shadow: egui::Shadow::NONE,
        }),
        ..SnarlStyle::new()
    }
}

impl View for GraphView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        StateWindow::show_from_window_state_resizable(
            ui,
            &self.local_state.window_state,
            WindowKind::GraphDebug,
            "Mixer graph debug",
            |ui| {
                if ui.button("Refresh").clicked() {
                    *self.snarl = Snarl::new();

                    if let Some(info) = self.player.get_graph_debug_info() {
                        // Maps mesic's node IDs to snarl's.
                        let mut id_map = HashMap::new();

                        for (node_index, label) in info.node_labels {
                            let snarl_id = self
                                .snarl
                                .insert_node(pos2(0.0, 0.0), GraphViewNode { label, node_index });
                            id_map.insert(node_index, snarl_id);
                        }
                        for (first, second) in info.edges {
                            let from = OutPinId {
                                node: id_map[&first],
                                output: 0,
                            };
                            let to = InPinId {
                                node: id_map[&second],
                                input: 0,
                            };
                            self.snarl.connect(from, to);
                        }
                    }
                }

                SnarlWidget::new()
                    .id(Id::new("graph-debug"))
                    .style(default_style())
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
            _ => 1, // just use a single input, even if we technically allow multiple (e.g. sum
                    // node).
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
