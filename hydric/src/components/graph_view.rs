use crate::view::View;
use std::collections::HashMap;
use eframe::{App, CreationContext};
use egui::{Color32, Id, Ui};
use egui_snarl::{
    InPin, InPinId, NodeId, OutPin, OutPinId, Snarl,
    ui::{
        AnyPins, NodeLayout, PinInfo, PinPlacement, SnarlStyle, SnarlViewer, SnarlWidget,
        WireStyle,
    },
};
use mesic::NodeLabel;

pub struct GraphView {}

impl GraphView {
    pub fn new() -> Self {
        Self {}
    }
}

impl View for GraphView {
    fn ui(&mut self, ui: &mut Ui) {}
}

struct GraphViewNode {
    label: NodeLabel,
}
