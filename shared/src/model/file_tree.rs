use crate::pmodel::{
    DirectoryProto, FilenameTreeProto, filename_tree_proto::Kind as FilenameTreeProtoKind,
};
use crate::serialize::map_vec;

pub type FilenameTreeNode = FileTreeNode<String, String>;

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
