use crate::view::View;
use crate::widget::default_window;
use egui::{Pos2, Ui};
use egui_ltreeview::TreeView;
use state::Store;

pub struct SampleTreeWindow<'a> {
    _store: &'a Store,
    visible: &'a mut bool,
}

impl<'a> SampleTreeWindow<'a> {
    pub fn new(store: &'a Store, visible: &'a mut bool) -> Self {
        SampleTreeWindow {
            _store: store,
            visible,
        }
    }
}

impl View for SampleTreeWindow<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        default_window("Samples")
            .open(self.visible)
            .default_pos(Pos2 { x: 600.0, y: 20.0 })
            .show(ui.ctx(), |ui| {
                let id = ui.make_persistent_id("sample_tree");
                TreeView::new(id).show(ui, |builder| {
                    // Hard coded for now.
                    builder.dir(0, "Drum kit");
                    builder.leaf(1, "Kick");
                    builder.leaf(2, "Snare");
                    builder.leaf(3, "Hi hat");
                    builder.leaf(4, "Tom");
                    builder.leaf(5, "Cymbal");
                    builder.close_dir();
                    builder.dir(6, "Loops");
                    builder.leaf(7, "Bass");
                    builder.leaf(8, "Guitar");
                    builder.leaf(9, "Piano");
                    builder.close_dir();
                });
            });
    }
}
