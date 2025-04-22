use crate::{FloatField, IndexField, Selector, TypeField, UintField};
use shared::action_proto::{
    ActionProto, SetChildIndexedProto, SetFloatIndexedProto, SetFloatProto, SetUintProto,
    action_proto::Kind as ActionKind,
};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // Generic actions which act generically on entities.
    SetFloat(FloatField, f32),
    SetUint(UintField, u32),
    DeleteChild(IndexField),
    SetChild(TypeField),
    AddChild(TypeField),
    SetChildIndexed(Selector, TypeField),
    SetFloatIndexed(Selector, FloatField, f32),

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
            ActionKind::DeleteChild(index) => Action::DeleteChild(index.into()),
            ActionKind::AddChild(child) => Action::AddChild(child.into()),
            ActionKind::SetChild(child) => Action::SetChild(child.into()),
            ActionKind::SetChildIndexed(it) => {
                let selector = it.selector.expect("Selector expected");
                let child = it.child.expect("Field expected");
                Action::SetChildIndexed(selector.into(), child.into())
            }
            ActionKind::SetFloatIndexed(it) => {
                let selector = it.selector.expect("Selector expected");

                let key = FloatField::from_str(&it.key)
                    .unwrap_or_else(|_| panic!("Expected float field name: {}", it.key));

                Action::SetFloatIndexed(selector.into(), key, it.value)
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
                Action::SetChild(child) => ActionKind::SetChild(child.into()),
                Action::AddChild(child) => ActionKind::AddChild(child.into()),
                Action::DeleteChild(index) => ActionKind::DeleteChild(index.into()),
                Action::MoveEffectUp(index) => ActionKind::MoveEffectUp(index as u32),
                Action::MoveEffectDown(index) => ActionKind::MoveEffectDown(index as u32),
                Action::SetChildIndexed(selector, child) => {
                    ActionKind::SetChildIndexed(SetChildIndexedProto {
                        selector: Some(selector.into()),
                        child: Some(child.into()),
                    })
                }
                Action::SetFloatIndexed(selector, key, value) => {
                    ActionKind::SetFloatIndexed(SetFloatIndexedProto {
                        selector: Some(selector.into()),
                        key: key.to_string(),
                        value,
                    })
                }

                // Non-serializable actions
                Action::Release => panic!(),
                Action::NonReversible => panic!(),
            }),
        }
    }
}
