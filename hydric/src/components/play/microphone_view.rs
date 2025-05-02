use crate::AsyncState;
use crate::promise::{poll, spawn};
use crate::rpc::load_sample_tree;
use crate::view::View;
use crate::widget::{checkbox, default_window, get_set, string_observer};
use egui::{Pos2, ScrollArea, Ui};
use egui_ltreeview::{TreeView, TreeViewBuilder};
use shared::model::{FileTreeConfig, FilenameTree};
use state::{Action, Store, TypeField};
use log::info;

pub struct MicrophoneView<'a> {
    store: &'a Store,
    async_state: &'a mut AsyncState,
    visible: &'a mut bool,
}

impl<'a> MicrophoneView<'a> {
    pub fn new(store: &'a Store, async_state: &'a mut AsyncState, visible: &'a mut bool) -> Self {
        MicrophoneView {
            store,
            async_state,
            visible,
        }
    }
}

impl View for MicrophoneView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        default_window("Microphone")
            .resizable(true)
            .open(self.visible)
            .default_pos(Pos2 { x: 600.0, y: 20.0 })
            .show(ui.ctx(), |ui| {
                if ui.button("test").clicked(){
                    info!("hi");
                }
            });
    }
}
