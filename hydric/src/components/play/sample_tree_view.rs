use crate::WindowKind;
use crate::local_state::LocalState;
use crate::promise::{poll, spawn};
use crate::rpc::{load_sample, load_sample_tree};
use crate::view::View;
use crate::widget::{StateWindow, checkbox};
use crate::{AsyncState, playback::AudioPlayer};
use egui::{Checkbox, ScrollArea, Ui};
use egui_ltreeview::{Action as TreeAction, TreeView, TreeViewBuilder};
use mesic::interleave_stereo;
use shared::model::{FileTree, FileTreeConfig, FilenameTree};
use state::{Action, Store, TypeField};

pub struct SampleTreeView<'a> {
    store: &'a Store,
    async_state: &'a mut AsyncState,
    player: &'a mut AudioPlayer,
    local_state: &'a LocalState,
}

impl<'a> SampleTreeView<'a> {
    pub fn new(
        store: &'a Store,
        async_state: &'a mut AsyncState,
        player: &'a mut AudioPlayer,
        local_state: &'a LocalState,
    ) -> Self {
        SampleTreeView {
            store,
            async_state,
            player,
            local_state,
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

pub fn interacted_sample_file_name(
    actions: Vec<TreeAction<usize>>,
    tree: &FileTree<String, String>,
) -> Option<String> {
    let FileTree::Directory(_, sample_files) = tree else {
        return None;
    };

    let action = actions.iter().find_map(|it| {
        if let TreeAction::Activate(activate) = it {
            Some(activate)
        } else {
            None
        }
    })?;

    let node_id = action.selected.iter().next()?;
    // TODO: support samples nested in directories
    let sample_node = sample_files.get(*node_id - 1)?;
    let FileTree::File(file_name) = sample_node else {
        return None;
    };

    Some(file_name.to_string())
}

impl View for SampleTreeView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        StateWindow::show_from_window_state(
            ui,
            &self.local_state.window_state,
            WindowKind::SampleTree,
            "Samples",
            |ui| {
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

                poll(&mut self.async_state.load_sample_tree, |tree| {
                    self.store
                        .dispatchr(Action::SetChild(TypeField::SampleTree(tree.clone())))
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
                            let selected_sample = interacted_sample_file_name(actions, tree);
                            if let Some(selected_sample) = selected_sample {
                                let cached_sample = self
                                    .local_state
                                    .sample_cache
                                    .borrow()
                                    .get(&selected_sample)
                                    .cloned();
                                if let Some(sample) = cached_sample {
                                    // if sample is already loaded in local state then play it
                                    let sample_audio = interleave_stereo(
                                        sample.left.clone(),
                                        sample.right.clone(),
                                    );
                                    self.player.set_audio(sample_audio);
                                    self.player.play();
                                } else {
                                    spawn(&mut self.async_state.load_sample, async move {
                                        load_sample(selected_sample).await
                                    });
                                }
                            }

                            poll(&mut self.async_state.load_sample, |sample| {
                                let sample_audio =
                                    interleave_stereo(sample.left.clone(), sample.right.clone());
                                self.player.set_audio(sample_audio);
                                self.player.play();
                                // cache into local state
                                self.local_state
                                    .sample_cache
                                    .borrow_mut()
                                    .insert(sample.sample_name.clone(), sample.clone());
                            });
                        });
                }
            },
        );
    }
}
