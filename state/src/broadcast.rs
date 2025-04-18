use crate::Action;

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
        Action::SetKey(..) => BroadcastType::Immediate,
        Action::SetScale(..) => BroadcastType::Immediate,
        Action::SetProjectName(..) => BroadcastType::Immediate,
        Action::SetFloat(..) => BroadcastType::OnRelease,
        Action::AddTrackPlacement(..) => BroadcastType::Immediate,
        Action::DeleteTrackPlacement(..) => BroadcastType::Immediate,
        Action::SetProjectList(..) => BroadcastType::Never,
        Action::SetProject(..) => BroadcastType::Never,
        Action::SetLoadProjectName(..) => BroadcastType::Never,
        // TODO: handle sample load/save better. Currently this could mean clients get out of sync with
        // each other.
        Action::AddSample(..) => BroadcastType::Never,
        Action::AddTrack(..) => BroadcastType::Immediate,
        Action::DeleteTrack(..) => BroadcastType::Immediate,
        Action::DeleteNote { .. } => BroadcastType::Immediate,
        Action::AddNote(..) => BroadcastType::Immediate,
        Action::SetNoteScaleValue(..) => BroadcastType::OnRelease,
        Action::SetNoteOctave(..) => BroadcastType::OnRelease,
        Action::SetNotePitchName(..) => BroadcastType::OnRelease,
        Action::SetMute(..) => BroadcastType::Immediate,
        Action::SetWave(..) => BroadcastType::Immediate,
        Action::SetOscCount(..) => BroadcastType::OnRelease,
        Action::SetEnvelope(..) => BroadcastType::OnRelease,
        Action::SetAntiAliasingMode(..) => BroadcastType::Immediate,
        Action::SetOversampleFactor(..) => BroadcastType::OnRelease,
        Action::MoveEffectUp(..) => BroadcastType::Immediate,
        Action::MoveEffectDown(..) => BroadcastType::Immediate,
        Action::DeleteEffect(..) => BroadcastType::Immediate,
        Action::AddEffect(..) => BroadcastType::Immediate,
        Action::SetEqKind(..) => BroadcastType::Immediate,
        Action::SetModDelayMinDepth(..) => BroadcastType::OnRelease,
        Action::SetModDelayMaxDepth(..) => BroadcastType::OnRelease,
        Action::SetModDelayLfoType(..) => BroadcastType::OnRelease,
        Action::SetTrackPlacementTrackId(..) => BroadcastType::Immediate,
        Action::SetTrackPlacementClippedDuration(..) => BroadcastType::OnRelease,
        Action::Release => BroadcastType::Never,
        Action::NonReversible => BroadcastType::Never,
    }
}
