use crate::{AsyncState, playback::AudioPlayer};
use crate::promise::{poll, spawn};
use crate::rpc::{load_sample, load_sample_tree};
use crate::view::View;
use crate::widget::{checkbox, default_window};
use egui::{Checkbox, Pos2, ScrollArea, Ui};
use egui_ltreeview::{TreeView, TreeViewBuilder, Action as TreeAction};
use shared::model::{FileTreeConfig, FilenameTree};
use state::{Action, Store, TypeField};
use mesic::interleave_stereo;
use log::info;
use shared::model::FileTree;

pub struct SampleTreeView<'a> {
    store: &'a Store,
    async_state: &'a mut AsyncState,
    visible: &'a mut bool,
    player: &'a mut AudioPlayer,
}

impl<'a> SampleTreeView<'a> {
    pub fn new(store: &'a Store, async_state: &'a mut AsyncState, visible: &'a mut bool, player: &'a mut AudioPlayer,) -> Self {
        SampleTreeView {
            store,
            async_state,
            visible,
            player
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

pub fn get_filename(actions: Vec<TreeAction<usize>>, tree: &FileTree<String, String>) -> Option<String> {
    for action in actions.iter() {
        match action {
            TreeAction::Activate(activate) => {
                if let Some(node_id) = activate.selected.iter().next() {
                    let filename = match tree {
                        FileTree::File(file) => file.clone(),
                        FileTree::Directory(_directory_name, subtree) => {
                            if let Some(sample_node) = subtree.get(*node_id - 1) {
                                match sample_node {
                                    FileTree::File(sample_file_name) => sample_file_name.clone(),
                                    FileTree::Directory(sub_directory_name, _sub_directory) => sub_directory_name.clone(),
                                }
                            } else {
                                continue;
                            }
                        }
                    };
                    return Some(filename);
                }
            }
            _ => {} // Ignore other TreeAction variants
        }
    }
    None
}

impl View for SampleTreeView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        default_window("Samples")
            .resizable(true)
            .open(self.visible)
            .default_pos(Pos2 { x: 600.0, y: 20.0 })
            .show(ui.ctx(), |ui| {
                let config = &self.store.get().sample_tree_config;

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
                    |value| Checkbox::new(value, "Show non-audio files"),
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
                );

                checkbox(
                    ui,
                    |value| Checkbox::new(value, "Show hidden files"),
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
                );

                // TODO: show a loading spinner.
                // TODO: investigate and resolve possible race conditions.
                if reload {
                    let config = config.clone();
                    spawn(&mut self.async_state.load_sample_tree, async move {
                        load_sample_tree(config).await
                    })
                }

                poll(&mut self.async_state.load_sample_tree, |_tree| {
                    
                });

                if let Some(tree) = &self.store.get().sample_tree {
                    ScrollArea::vertical()
                        .min_scrolled_height(200.0)
                        .show(ui, |ui| {
                            let id = ui.make_persistent_id("sample_tree");
                            let (_response, actions) = TreeView::new(id).show(ui, |builder| {
                                add_node(
                                    builder, tree, /* start_id= */ 0,
                                    /* ignore_top= */ true,
                                );
                            });
                            let filename = get_filename(actions, tree);
                            if let Some(filename) = filename {
                                spawn(&mut self.async_state.load_sample, async move {
                                    load_sample(filename.to_string()).await
                                });
                                ui.ctx().request_repaint();
                            }
                            });
                        }
                });
                
                poll(&mut self.async_state.load_sample, |sample| {
                    self.store.dispatchr(Action::AddChild(TypeField::Sample(sample.clone())));
                    let all_samples = &self.store.get().project.samples;
                    if all_samples.is_empty() {
                        info!("NOOOOOO");
                        return
                    }
                    info!("is this even happening");
                    let sample = all_samples[0].clone(); // get most recent one
                    let sample = interleave_stereo(sample.left, sample.right);
                    self.player.set_audio(sample);
                    self.player.play();
                }); 


                }
}
