use crate::pmodel::{
    DirectoryProto, FileTreeConfigProto, FilenameTreeProto,
    filename_tree_proto::Kind as FilenameTreeProtoKind,
};
use crate::serialize::map_vec;
use local_macro::{FromProto, IntoProto};

pub type FilenameTree = FileTree<String, String>;

#[derive(Debug, PartialEq, Clone)]
pub enum FileTree</* FileData= */ T, /* DirectoryData= */ U> {
    File(T),
    Directory(U, Vec<FileTree<T, U>>),
}

#[derive(Debug, PartialEq, Clone, FromProto, IntoProto)]
pub struct FileTreeConfig {
    // TODO: consider just making this a regular String and special-casing the empty string.
    pub search: Option<String>,
    pub skip_non_audio: bool,
    pub skip_hidden: bool,
}

impl From<FilenameTreeProto> for FilenameTree {
    fn from(other: FilenameTreeProto) -> Self {
        match other.kind.unwrap() {
            FilenameTreeProtoKind::Filename(name) => FilenameTree::File(name),
            FilenameTreeProtoKind::Directory(directory) => {
                FilenameTree::Directory(directory.name, map_vec(directory.contents))
            }
        }
    }
}

impl From<FilenameTree> for FilenameTreeProto {
    fn from(other: FilenameTree) -> Self {
        FilenameTreeProto {
            kind: Some(match other {
                FilenameTree::File(name) => FilenameTreeProtoKind::Filename(name),
                FilenameTree::Directory(name, contents) => {
                    FilenameTreeProtoKind::Directory(DirectoryProto {
                        name,
                        contents: map_vec(contents),
                    })
                }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::proto::proto_testing::assert_proto_round_trip;

    use super::*;

    #[test]
    fn file_tree_proto_round_trip() {
        let tree = FilenameTree::Directory(
            "Samples".to_string(),
            vec![
                FilenameTree::Directory(
                    "Drum kit".to_string(),
                    vec![
                        FilenameTree::File("Kick".to_string()),
                        FilenameTree::Directory(
                            "Snares".to_string(),
                            vec![
                                FilenameTree::File("Snare 1".to_string()),
                                FilenameTree::File("Snare 2".to_string()),
                            ],
                        ),
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

        assert_proto_round_trip::<FilenameTree, FilenameTreeProto>(tree);
    }
}
