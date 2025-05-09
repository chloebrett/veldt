use crate::{
    Action, EffectSelector, GeneratorSelector, MixerMatrixCellSelector, MixerSelector,
    NoteSelector, OscillatorSelector, PlacementSelector, RootSelector, Selector, SelectorTrait,
    StoreData, TrackSelector, receiver::ActionReceiver, LfoSelector
};

fn reducer_internal<T: SelectorTrait>(
    selector: T,
    data: &mut StoreData,
    action: &Action,
) -> Option<Action> {
    selector.select_mut(data).apply(action)
}

pub fn reducer(data: &mut StoreData, selector: &Selector, action: &Action) -> Option<Action> {
    match *selector {
        Selector::Root => reducer_internal(RootSelector, data, action),
        Selector::Track(a) => reducer_internal(TrackSelector(a), data, action),
        Selector::Note(a, b) => reducer_internal(NoteSelector(a, b), data, action),
        Selector::Mixer(a) => reducer_internal(MixerSelector(a), data, action),
        Selector::Effect(a, b) => reducer_internal(EffectSelector(a, b), data, action),
        Selector::Placement(a) => reducer_internal(PlacementSelector(a), data, action),
        Selector::Generator(a) => reducer_internal(GeneratorSelector(a), data, action),
        Selector::Oscillator(a, b) => reducer_internal(OscillatorSelector(a, b), data, action),
        Selector::Lfo(a, b)=> reducer_internal(LfoSelector(a, b), data, action),
        Selector::MixerMatrixCell(a, b) => {
            reducer_internal(MixerMatrixCellSelector(a, b), data, action)
        }
    }
}
