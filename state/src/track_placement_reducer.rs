use crate::Action;
use ordered_float::OrderedFloat;
use shared::logger::log;
use shared::model::TrackPlacement;

pub fn track_placement_reducer(track_placement: &mut TrackPlacement, action: &Action) -> Action {
    log(&format!(
        "track_placement_reducer processing: {:?}",
        action.clone()
    ));

    match action {
        Action::SetTrackPlacementTrackId(track_id) => {
            let prev = track_placement.track_id;
            track_placement.track_id = *track_id;
            Action::SetTrackPlacementTrackId(prev)
        }
        Action::SetTrackPlacementOffset(offset) => {
            let prev = track_placement.offset;
            track_placement.offset = OrderedFloat(*offset);
            Action::SetTrackPlacementOffset(*prev)
        }
        Action::SetTrackPlacementClippedDuration(duration) => {
            if let Some(prev) = track_placement.clipped_duration {
                track_placement.clipped_duration = Some(OrderedFloat(*duration));
                Action::SetTrackPlacementClippedDuration(*prev)
            } else {
                track_placement.clipped_duration = Some(OrderedFloat(*duration));
                Action::RemoveTrackPlacementClippedDuration()
            }
        }
        Action::RemoveTrackPlacementClippedDuration() => {
            if let Some(prev) = track_placement.clipped_duration {
                track_placement.clipped_duration = None;
                Action::SetTrackPlacementClippedDuration(*prev)
            } else {
                track_placement.clipped_duration = None;
                Action::RemoveTrackPlacementClippedDuration()
            }
        }
        _ => Action::NonReversible,
    }
}
