use crate::view::View;
use crate::widget::default_window;
use egui::{Pos2, Ui};
use egui_ltreeview::{TreeView, TreeViewBuilder};
use shared::model::FilenameTree;
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

/// Adds a FilenameTree to a TreeViewBuilder. Returns the next unused ID.
fn add_node(
    builder: &mut TreeViewBuilder<usize>,
    node: &FilenameTree,
    start_id: usize,
) -> usize {
    match node {
        FilenameTree::File(name) => {
            builder.leaf(start_id, name);
            return start_id + 1;
        }
        FilenameTree::Directory(name, contents) => {
            builder.dir(start_id, name);
            let mut next_id = start_id + 1;
            for child in contents {
                next_id = add_node(builder, child, next_id);
            }
            builder.close_dir();
            return next_id;
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
                // Hard coded for now.
                let tree = FilenameTree::Directory(
                    "Samples".to_string(),
                    vec![
                        FilenameTree::Directory(
                            "Drum kit".to_string(),
                            vec![
                                FilenameTree::File("Kick".to_string()),
                                FilenameTree::File("Snare".to_string()),
                                FilenameTree::File("Hi hat".to_string()),
                                FilenameTree::File("Tom".to_string()),
                                FilenameTree::File("Cymbal".to_string()),
                            ],
                        ),
                        FilenameTree::Directory(
                            "Loops".to_string(),
                            vec![
                                FilenameTree::File("Bass".to_string()),
                                FilenameTree::File("Guitar".to_string()),
                                FilenameTree::File("Piano".to_string()),
                            ],
                        ),
                    ],
                );
                TreeView::new(id).show(ui, |builder| {
                    add_node(builder, &tree, 0);
                });
            });
    }
}
