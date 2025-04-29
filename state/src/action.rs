use crate::{FloatField, IndexField, TypeField, UintField};
use shared::action_proto::{
    ActionProto, SetFloatProto, SetUintProto, action_proto::Kind as ActionKind,
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
    // Delete a child object *by* index.
    DeleteChild(IndexField),
    // Set a child object by type.
    SetChild(TypeField),
    // Add a child object by type.
    AddChild(TypeField),

    // Note: avoid creating new ad hoc action types.
    // Try to encapsulate them within a generic action type like the ones above.
    // Perhaps a generic "MoveChild" action could work.
    MoveEffectUp(usize),
    MoveEffectDown(usize),

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
            ActionKind::MoveEffectUp(it) => Action::MoveEffectUp(it as usize),
            ActionKind::MoveEffectDown(it) => Action::MoveEffectDown(it as usize),
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
            ActionKind::AddChild(child) => Action::AddChild(child.into()),
            ActionKind::SetChild(child) => Action::SetChild(child.into()),
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
                Action::DeleteChild(index) => ActionKind::DeleteChild(index.into()),
                Action::MoveEffectUp(index) => ActionKind::MoveEffectUp(index as u32),
                Action::MoveEffectDown(index) => ActionKind::MoveEffectDown(index as u32),

                // Non-serializable actions
                Action::Release => panic!(),
                Action::NonReversible => panic!(),
            }),
        }
    }
}
