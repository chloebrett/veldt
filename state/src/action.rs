use crate::{
    FloatField, IndexField, MoveField, MultiIndexField, MultiTypeField, TypeField, UintField,
};
use shared::action_proto::{
    ActionProto, ChildIdPairProto, ChildIndexPairProto, SetFloatProto, SetUintProto,
    action_proto::Kind as ActionKind,
};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // Set a child field that happens to be a float.
    SetFloat(FloatField, f32),
    // Set a child field that happens to be a uint.
    SetUint(UintField, u32),
    // Set a child field that happens to be an index.
    SetIndex(IndexField),
    // Delete a child object by index.
    DeleteChild(IndexField),
    // Delete a child object by ID.
    // TODO: ID types live on TypeField, maybe they should have their own enum?
    DeleteChildById(TypeField),
    // Delete multiple children by index.
    DeleteChildren(MultiIndexField),
    // Delete multiple children by ID.
    // TODO: ID types live on TypeField, maybe they should have their own enum?
    DeleteChildrenById(MultiTypeField),
    // Set a child object by type.
    SetChild(TypeField),
    // Add a child object by type.
    AddChild(TypeField),
    // Add a child with a specific ID. (e.g. adding a new effect).
    // This allows IDs to be created in a single place annd then communicated to the receivers.
    AddChildWithId(TypeField, TypeField),
    AddChildAtIndex(TypeField, IndexField),
    // Set children of an object by type.
    AddChildren(MultiTypeField),
    SetChildren(MultiTypeField),
    MoveChild(MoveField),

    /// Denotes that the mouse has been released from a UI element, finalizing its value.
    /// This is how we know to flatten (in the undo stack) actions that modify floats.
    /// This event is just a marker, it doesn't get passed on to the reducers.
    Release,
    /// Denotes the reverse-action for an action that isn't reversible.
    /// Applying this is a no-op.
    /// There might be a better way of describing this concept, keep a look out.
    NonReversible,
}

impl From<ActionProto> for Action {
    fn from(other: ActionProto) -> Action {
        match other.kind.unwrap() {
            ActionKind::MoveChild(it) => Action::MoveChild(it.into()),
            ActionKind::SetFloat(it) => Action::SetFloat(
                FloatField::from_str(&it.key)
                    .unwrap_or_else(|_| panic!("Expected float field name: {}", it.key)),
                it.value,
            ),
            ActionKind::SetUint(it) => Action::SetUint(
                UintField::from_str(&it.key)
                    .unwrap_or_else(|_| panic!("Expected uint field name: {}", it.key)),
                it.value,
            ),
            ActionKind::SetIndex(index) => Action::SetIndex(index.into()),
            ActionKind::DeleteChild(index) => Action::DeleteChild(index.into()),
            ActionKind::DeleteChildById(id) => Action::DeleteChildById(id.into()),
            ActionKind::DeleteChildren(indexes) => Action::DeleteChildren(indexes.into()),
            ActionKind::DeleteChildrenById(ids) => Action::DeleteChildrenById(ids.into()),
            ActionKind::AddChild(child) => Action::AddChild(child.into()),
            ActionKind::AddChildAtIndex(ChildIndexPairProto { child, index }) => {
                Action::AddChildAtIndex(child.unwrap().into(), index.unwrap().into())
            }
            ActionKind::SetChild(child) => Action::SetChild(child.into()),
            ActionKind::SetChildren(children) => Action::SetChildren(children.into()),
            ActionKind::AddChildren(children) => Action::AddChildren(children.into()),
            ActionKind::AddChildWithId(ChildIdPairProto { child, id }) => {
                Action::AddChildWithId(child.unwrap().into(), id.unwrap().into())
            }
        }
    }
}

impl From<Action> for ActionProto {
    fn from(other: Action) -> ActionProto {
        ActionProto {
            kind: Some(match other {
                Action::SetFloat(key, value) => ActionKind::SetFloat(SetFloatProto {
                    key: key.to_string(),
                    value,
                }),
                Action::SetUint(key, value) => ActionKind::SetUint(SetUintProto {
                    key: key.to_string(),
                    value,
                }),
                Action::SetIndex(index) => ActionKind::SetIndex(index.into()),
                Action::SetChild(child) => ActionKind::SetChild(child.into()),
                Action::AddChild(child) => ActionKind::AddChild(child.into()),
                Action::AddChildAtIndex(child, index) => {
                    ActionKind::AddChildAtIndex(ChildIndexPairProto {
                        child: Some(child.into()),
                        index: Some(index.into()),
                    })
                }
                Action::DeleteChild(index) => ActionKind::DeleteChild(index.into()),
                Action::DeleteChildById(id) => ActionKind::DeleteChildById(id.into()),
                Action::DeleteChildren(indexes) => ActionKind::DeleteChildren(indexes.into()),
                Action::DeleteChildrenById(ids) => ActionKind::DeleteChildrenById(ids.into()),
                Action::MoveChild(it) => ActionKind::MoveChild(it.into()),
                Action::SetChildren(it) => ActionKind::SetChildren(it.into()),
                Action::AddChildren(it) => ActionKind::AddChildren(it.into()),
                Action::AddChildWithId(child, id) => ActionKind::AddChildWithId(ChildIdPairProto {
                    child: Some(child.into()),
                    id: Some(id.into()),
                }),
                // Non-serializable actions
                Action::Release => panic!(),
                Action::NonReversible => panic!(),
            }),
        }
    }
}
