use crate::pmodel::{
    DirectoryProto, FilenameTreeProto, filename_tree_proto::Kind as FilenameTreeProtoKind,
};
use crate::serialize::map_vec;

pub type FilenameTreeNode = FileTreeNode<String, String>;

#[derive(Debug, PartialEq, Clone)]
pub enum FileTreeNode</* FileData= */ T, /* DirectoryData= */ U> {
    File(T),
    Directory(U, Vec<FileTreeNode<T, U>>),
}

impl From<FilenameTreeProto> for FilenameTreeNode {
    fn from(other: FilenameTreeProto) -> Self {
        match other.kind.unwrap() {
            FilenameTreeProtoKind::Filename(name) => FilenameTreeNode::File(name),
            FilenameTreeProtoKind::Directory(directory) => {
                FilenameTreeNode::Directory(directory.name, map_vec(directory.contents))
            }
        }
    }
}

impl From<FilenameTreeNode> for FilenameTreeProto {
    fn from(other: FilenameTreeNode) -> Self {
        FilenameTreeProto {
            kind: Some(match other {
                FilenameTreeNode::File(name) => FilenameTreeProtoKind::Filename(name),
                FilenameTreeNode::Directory(name, contents) => {
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
        let tree = FilenameTreeNode::Directory(
            "Samples".to_string(),
            vec![
                FilenameTreeNode::Directory(
                    "Drum kit".to_string(),
                    vec![
                        FilenameTreeNode::File("Kick".to_string()),
                        FilenameTreeNode::Directory(
                            "Snares".to_string(),
                            vec![
                                FilenameTreeNode::File("Snare 1".to_string()),
                                FilenameTreeNode::File("Snare 2".to_string()),
                            ],
                        ),
                        FilenameTreeNode::File("Hi hat".to_string()),
                        FilenameTreeNode::File("Tom".to_string()),
                        FilenameTreeNode::File("Cymbal".to_string()),
                    ],
                ),
                FilenameTreeNode::Directory(
                    "Loops".to_string(),
                    vec![
                        FilenameTreeNode::File("Bass".to_string()),
                        FilenameTreeNode::File("Guitar".to_string()),
                        FilenameTreeNode::File("Piano".to_string()),
                    ],
                ),
            ],
        );

        assert_proto_round_trip::<FilenameTreeNode, FilenameTreeProto>(tree);
    }
}
