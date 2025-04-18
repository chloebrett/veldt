use crate::{Action, Selector, StoreData, receiver::ActionReceiver};

pub fn reducer(data: &mut StoreData, selector: &Selector, action: &Action) -> Action {
    match selector {
        Selector::Track(track_index) => {
            let track = &mut data.project.tracks[*track_index];
            track.apply(action).unwrap_or(Action::NonReversible)
        }
        Selector::Generator(generator_index) => {
            let generator = &mut data.project.generators[*generator_index];
            generator.apply(action).unwrap_or(Action::NonReversible)
        }
        Selector::Effect(mixer_index, effect_index) => {
            let effect = &mut data.project.mixer[*mixer_index].effects[*effect_index];
            effect.apply(action).unwrap_or(Action::NonReversible)
        }
        Selector::Mixer(mixer_index) => {
            let mixer_channel = &mut data.project.mixer[*mixer_index];
            mixer_channel.apply(action).unwrap_or(Action::NonReversible)
        }
        Selector::Note(track_index, note_index) => {
            let note = &mut data.project.tracks[*track_index].notes[*note_index];
            note.apply(action).unwrap_or(Action::NonReversible)
        }
        Selector::TrackPlacement(track_placement_index) => {
            let track_placement = &mut data.project.track_placements[*track_placement_index];
            track_placement
                .apply(action)
                .unwrap_or(Action::NonReversible)
        }
        Selector::Root => data.apply(action).unwrap_or(Action::NonReversible),
    }
}
