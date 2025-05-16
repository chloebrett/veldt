use crate::AsyncState;
use crate::promise::{poll, spawn};
use crate::rpc::load_sample_tree;
use crate::view::View;
use crate::widget::{checkbox, default_window, get_set, string_observer};
use egui::{Pos2, ScrollArea, Ui};
use egui_ltreeview::{TreeView, TreeViewBuilder};
use shared::model::{FileTreeConfig, FilenameTree};
use state::{Action, Store, TypeField};

pub struct SampleTreeView<'a> {
    store: &'a Store,
    async_state: &'a mut AsyncState,
    visible: &'a mut bool,
}

impl<'a> SampleTreeView<'a> {
    pub fn new(store: &'a Store, async_state: &'a mut AsyncState, visible: &'a mut bool) -> Self {
        SampleTreeView {
            store,
            async_state,
            visible,
        }
    }
}

/// Adds a FilenameTree to a TreeViewBuilder. Returns the next unused ID.
/// If ignore_top = true, does not push the top-level directory.
/// This is useful for samples, as we don't care about rendering the top-level
/// "samples" directory name.
fn add_node(
    builder: &mut TreeViewBuilder<usize>,
    node: &FilenameTree,
    start_id: usize,
    ignore_top: bool,
) -> usize {
    match node {
        FilenameTree::File(name) => {
            builder.leaf(start_id, name);
            start_id + 1
        }
        FilenameTree::Directory(name, contents) => {
            if !ignore_top {
                builder.dir(start_id, name);
            }
            let mut next_id = start_id + 1;
            for node in contents {
                next_id = add_node(builder, node, next_id, /* ignore_top= */ false);
            }
            if !ignore_top {
                builder.close_dir();
            }
            next_id
        }
    }
}

impl View for SampleTreeView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        default_window("Samples")
            .resizable(true)
            .open(self.visible)
            .default_pos(Pos2 { x: 600.0, y: 20.0 })
            .show(ui.ctx(), |ui| {
                let config = &self.store.get().sample_tree_config;
                let search = config.search.clone();

                let mut search = config.search.clone();
                let response = ui.text_edit_singleline(&mut search);
                if response.changed() {
                    self.store
                        .dispatchr(Action::SetChild(TypeField::SampleTreeConfig(
                            FileTreeConfig {
                                search,
                                ..config.clone()
                            },
                        )));
                }

                let mut reload = false;
                if ui.button("Search").clicked() {
                    reload = true;
                }

                checkbox(
                    ui,
                    config.show_non_audio,
                    |show_non_audio| {
                        self.store
                            .dispatchr(Action::SetChild(TypeField::SampleTreeConfig(
                                FileTreeConfig {
                                    show_non_audio,
                                    ..config.clone()
                                },
                            )));
                        reload = true;
                    },
                    "Show non-audio files",
                );

                checkbox(
                    ui,
                    config.show_hidden,
                    |show_hidden| {
                        self.store
                            .dispatchr(Action::SetChild(TypeField::SampleTreeConfig(
                                FileTreeConfig {
                                    show_hidden,
                                    ..config.clone()
                                },
                            )));
                        reload = true;
                    },
                    "Show hidden files",
                );

                // TODO: show a loading spinner.
                // TODO: investigate and resolve possible race conditions.
                if reload {
                    let config = config.clone();
                    spawn(&mut self.async_state.load_sample_tree, async move {
                        load_sample_tree(config).await
                    })
                }

                poll(&mut self.async_state.load_sample_tree, |tree| {
                    self.store
                        .dispatchr(Action::SetChild(TypeField::SampleTree(tree.clone())))
                });

                if let Some(tree) = &self.store.get().sample_tree {
                    ScrollArea::vertical()
                        .min_scrolled_height(200.0)
                        .show(ui, |ui| {
                            let id = ui.make_persistent_id("sample_tree");
                            TreeView::new(id).show(ui, |builder| {
                                add_node(
                                    builder, tree, /* start_id= */ 0,
                                    /* ignore_top= */ true,
                                );
                            });
                        });
                }
            });
    }
}
