use crate::{Action, TypeField};

#[derive(PartialEq)]
pub enum BroadcastType {
    Immediate, // immediately broadcast to server.
    OnRelease, // wait until an Action::Release. Used for continuous values. Will also fire if the
    // next action type is (a) different to the previous one and (b) broadcastable.
    Never, // don't broadcast.
}

/// Discrete actions should be broadcasted to the server immediately. Continuous actions should wait for
/// an Action::Release.
/// This must be exhaustive.
/// Also consider whether this could be part of the return value of applying an action instead.
pub fn broadcast_type(action: &Action) -> BroadcastType {
    match action {
        Action::SetFloat(..) => BroadcastType::OnRelease,
        Action::SetUint(..) => BroadcastType::OnRelease,
        Action::SetIndex(..) => BroadcastType::OnRelease,
        Action::MoveChild(..) => BroadcastType::Immediate,
        Action::SetChild(child) => match child {
            TypeField::ProjectList(..) => BroadcastType::Never,
            TypeField::Project(..) => BroadcastType::Never,
            TypeField::LoadProjectName(..) => BroadcastType::Never,
            TypeField::SampleTree(..) => BroadcastType::Never,
            TypeField::SampleTreeConfig(..) => BroadcastType::Never,
            TypeField::Octave(..) => BroadcastType::OnRelease,
            _ => BroadcastType::Immediate,
        },
        Action::SetChildren(..) => BroadcastType::Immediate,
        Action::AddChild(child) => match child {
            // TODO: handle sample load/save better. Currently this could mean clients get out of sync with
            // each other.
            TypeField::Sample(..) => BroadcastType::Never,
            _ => BroadcastType::Immediate,
        },
        Action::AddChildAtIndex(..) => BroadcastType::Immediate,
        Action::AddChildren(..) => BroadcastType::Immediate,
        Action::DeleteChild(..) => BroadcastType::Immediate,
        Action::DeleteChildById(..) => BroadcastType::Immediate,
        Action::DeleteChildren(..) => BroadcastType::Immediate,
        Action::DeleteChildrenById(..) => BroadcastType::Immediate,
        Action::Release => BroadcastType::Never,
        Action::NonReversible => BroadcastType::Never,
        Action::UpdateChildId(..) => BroadcastType::Immediate,
    }
}
