use crate::{LocalState, WindowKind, playback::AudioPlayer, view::View, widget::StateWindow};
use egui::{Color32, Id, Ui, pos2};
use egui_snarl::{
    InPin, InPinId, OutPin, OutPinId, Snarl,
    ui::{NodeLayout, PinInfo, PinPlacement, SnarlStyle, SnarlViewer, SnarlWidget},
};
use mesic::NodeLabel;
use std::collections::HashMap;

pub struct GraphView<'a> {
    local_state: &'a LocalState,
    snarl: &'a mut Snarl<GraphViewNode>,
    player: &'a AudioPlayer,
}

impl<'a> GraphView<'a> {
    pub fn new(
        local_state: &'a LocalState,
        snarl: &'a mut Snarl<GraphViewNode>,
        player: &'a AudioPlayer,
    ) -> Self {
        Self {
            local_state,
            snarl,
            player,
        }
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
                                .insert_node(pos2(0.0, 0.0), GraphViewNode { label });
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
                    .style(SnarlStyle::new())
                    .show(self.snarl, &mut GraphViewer, ui);
            },
        );
    }
}

#[derive(Debug)]
pub struct GraphViewNode {
    label: NodeLabel,
}

struct GraphViewer;

impl SnarlViewer<GraphViewNode> for GraphViewer {
    fn connect(&mut self, _from: &OutPin, _to: &InPin, _snarl: &mut Snarl<GraphViewNode>) {}

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

    fn outputs(&mut self, _node: &GraphViewNode) -> usize {
        1
    }

    #[allow(refining_impl_trait)]
    fn show_input(
        &mut self,
        _pin: &InPin,
        _ui: &mut Ui,
        _snarl: &mut Snarl<GraphViewNode>,
    ) -> PinInfo {
        PinInfo::circle().with_fill(Color32::GRAY)
    }

    #[allow(refining_impl_trait)]
    fn show_output(
        &mut self,
        _pin: &OutPin,
        _ui: &mut Ui,
        _snarl: &mut Snarl<GraphViewNode>,
    ) -> PinInfo {
        PinInfo::circle().with_fill(Color32::GRAY)
    }
}
