use state::TrackSelector;

#[derive(Default)]
pub struct LocalState {
    pub active_track: Option<TrackSelector>,
}
