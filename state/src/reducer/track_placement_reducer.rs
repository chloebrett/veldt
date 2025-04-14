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
            let prev = track_placement.clipped_duration.map(|value| *value);
            track_placement.clipped_duration = duration.map(OrderedFloat);
            Action::SetTrackPlacementClippedDuration(prev)
        }

        _ => Action::NonReversible,
    }
}
