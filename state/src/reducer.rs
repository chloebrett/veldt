use crate::{Action, Selector, StoreData, receiver::ActionReceiver};
use shared::model::GeneratorType;

pub fn reducer(data: &mut StoreData, selector: &Selector, action: &Action) -> Option<Action> {
    match selector {
        Selector::Track(track_index) => {
            let track = &mut data.project.tracks[*track_index];
            track.apply(action)
        }
        Selector::Generator(generator_index) => {
            let generator = &mut data.project.generators[*generator_index];
            generator.apply(action)
        }
        Selector::Effect(mixer_index, effect_index) => {
            let effect = &mut data.project.mixer[*mixer_index].effects[*effect_index];
            effect.apply(action)
        }
        Selector::Mixer(mixer_index) => {
            let mixer_channel = &mut data.project.mixer[*mixer_index];
            mixer_channel.apply(action)
        }
        Selector::Note(track_index, note_index) => {
            let note = &mut data.project.tracks[*track_index].notes[*note_index];
            note.apply(action)
        }
        Selector::Placement(placement_index) => {
            let placement = &mut data.project.placements[*placement_index];
            placement.apply(action)
        }
        Selector::Oscillator(generator_index, oscillator_index) => {
            let generator = &mut data.project.generators[*generator_index];
            match &mut generator.kind {
                GeneratorType::SubSynth(config) => {
                    config.oscillators[*oscillator_index].apply(action)
                }
                _ => None,
            }
        }
        Selector::Root => data.apply(action),
    }
}
