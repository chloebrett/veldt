use crate::{StoreData, receiver::ActionReceiver};
use shared::action_proto::{
    SelectorProto, selector_proto::IndexPair, selector_proto::Kind as SelectorKind,
};
use shared::model::{
    EffectInstance, GeneratorInstance, MatrixCell, MixerChannel, Oscillator, PlacedNote, Placement,
    SubSynthConfig, Track,
};

// TODO: rename to just Selector when Selector enum is gone.
pub trait SelectorTrait {
    type Item: ActionReceiver;

    fn select<'a>(&'a self, store: &'a StoreData) -> &'a Self::Item {
        self.try_select(store).unwrap()
    }

    fn select_mut<'a>(&'a self, store: &'a mut StoreData) -> &'a mut Self::Item {
        self.try_select_mut(store).unwrap()
    }

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item>;

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item>;

    fn as_enum(&self) -> Selector;
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct RootSelector;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct TrackSelector(/* track_index */ pub usize);

impl TrackSelector {
    pub fn downcast_note(&self, note_index: usize) -> NoteSelector {
        NoteSelector(self.0, note_index)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct NoteSelector(
    /* track_index */ pub usize,
    /* note_index */ pub usize,
);

impl NoteSelector {
    pub fn upcast(&self) -> TrackSelector {
        TrackSelector(self.0)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct MixerSelector(/* mixer_index */ pub usize);

impl MixerSelector {
    pub fn downcast_effect(&self, effect_index: usize) -> EffectSelector {
        EffectSelector(self.0, effect_index)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct EffectSelector(
    /* mixer_index */ pub usize,
    /* effect_index */ pub usize,
);

impl EffectSelector {
    pub fn upcast(&self) -> MixerSelector {
        MixerSelector(self.0)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct GeneratorSelector(/* generator_index */ pub usize);

impl GeneratorSelector {
    pub fn downcast_oscillator(&self, oscillator_index: usize) -> OscillatorSelector {
        OscillatorSelector(self.0, oscillator_index)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct PlacementSelector(/* placement_index */ pub usize);

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct OscillatorSelector(
    /* generator_index */ pub usize,
    /* oscillator_index */ pub usize,
);

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct MixerMatrixCellSelector(/* row */ pub usize, /* col */ pub usize);

impl OscillatorSelector {
    pub fn upcast(&self) -> GeneratorSelector {
        GeneratorSelector(self.0)
    }
}

impl SelectorTrait for RootSelector {
    type Item = StoreData;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        Some(store)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        Some(store)
    }

    fn as_enum(&self) -> Selector {
        Selector::Root
    }
}

impl SelectorTrait for TrackSelector {
    type Item = Track;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store.project.tracks.get(self.0)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store.project.tracks.get_mut(self.0)
    }

    fn as_enum(&self) -> Selector {
        Selector::Track(self.0)
    }
}

impl SelectorTrait for NoteSelector {
    type Item = PlacedNote;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store
            .project
            .tracks
            .get(self.0)
            .and_then(|it| it.notes.get(self.1))
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store
            .project
            .tracks
            .get_mut(self.0)
            .and_then(|it| it.notes.get_mut(self.1))
    }

    fn as_enum(&self) -> Selector {
        Selector::Note(self.0, self.1)
    }
}

impl SelectorTrait for MixerSelector {
    type Item = MixerChannel;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store.project.mixer.channels.get(self.0)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store.project.mixer.channels.get_mut(self.0)
    }

    fn as_enum(&self) -> Selector {
        Selector::Mixer(self.0)
    }
}

impl SelectorTrait for EffectSelector {
    type Item = EffectInstance;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store
            .project
            .mixer
            .channels
            .get(self.0)
            .and_then(|it| it.effects.get(self.1))
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store
            .project
            .mixer
            .channels
            .get_mut(self.0)
            .and_then(|it| it.effects.get_mut(self.1))
    }

    fn as_enum(&self) -> Selector {
        Selector::Effect(self.0, self.1)
    }
}

impl SelectorTrait for GeneratorSelector {
    type Item = GeneratorInstance;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store.project.generators.get(self.0)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store.project.generators.get_mut(self.0)
    }

    fn as_enum(&self) -> Selector {
        Selector::Generator(self.0)
    }
}

impl SelectorTrait for PlacementSelector {
    type Item = Placement;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store.project.placements.get(self.0)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store.project.placements.get_mut(self.0)
    }

    fn as_enum(&self) -> Selector {
        Selector::Placement(self.0)
    }
}

impl SelectorTrait for OscillatorSelector {
    type Item = Oscillator;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        let instance = store.project.generators.get(self.0)?;
        let subsynth: &SubSynthConfig = (&instance.it).try_into().ok()?;
        subsynth.oscillators.get(self.1)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        let instance = store.project.generators.get_mut(self.0)?;
        let subsynth: &mut SubSynthConfig = (&mut instance.it).try_into().ok()?;
        subsynth.oscillators.get_mut(self.1)
    }

    fn as_enum(&self) -> Selector {
        Selector::Oscillator(self.0, self.1)
    }
}

impl SelectorTrait for MixerMatrixCellSelector {
    type Item = MatrixCell;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store.project.mixer.matrix.get(self.0, self.1)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store.project.mixer.matrix.get_mut(self.0, self.1)
    }

    fn as_enum(&self) -> Selector {
        Selector::MixerMatrixCell(self.0, self.1)
    }
}

// Enum version of the selector.
// TODO: hide the visibility of this. We will still use it internally to efficiently represent a
// generic selector.
// Maybe rename to SelectorEnum.
#[derive(Clone, Debug, PartialEq)]
pub enum Selector {
    Root,
    Track(/* track_index */ usize),
    Note(/* track_index */ usize, /* note_index */ usize),
    Mixer(/* mixer_index */ usize),
    Effect(/* mixer_index */ usize, /* effect_index */ usize),
    Generator(/* generator_index */ usize),
    Placement(/* placement_index */ usize),
    Oscillator(
        /* generator_index */ usize,
        /* oscillator_index */ usize,
    ),
    MixerMatrixCell(/* row */ usize, /* col */ usize),
}

impl From<Selector> for SelectorProto {
    fn from(other: Selector) -> SelectorProto {
        SelectorProto {
            kind: Some(match other {
                Selector::Root => SelectorKind::Root(0),
                Selector::Track(it) => SelectorKind::Track(it as u32),
                Selector::Note(first, second) => SelectorKind::Note(pair(first, second)),
                Selector::Mixer(it) => SelectorKind::Mixer(it as u32),
                Selector::Effect(first, second) => SelectorKind::Effect(pair(first, second)),
                Selector::Generator(it) => SelectorKind::Generator(it as u32),
                Selector::Placement(it) => SelectorKind::Placement(it as u32),
                Selector::Oscillator(first, second) => {
                    SelectorKind::Oscillator(pair(first, second))
                }
                Selector::MixerMatrixCell(first, second) => {
                    SelectorKind::MixerMatrixCell(pair(first, second))
                }
            }),
        }
    }
}

impl From<SelectorProto> for Selector {
    fn from(other: SelectorProto) -> Selector {
        match other.kind.unwrap() {
            SelectorKind::Root(..) => Selector::Root,
            SelectorKind::Track(it) => Selector::Track(it as usize),
            SelectorKind::Note(IndexPair { first, second }) => {
                Selector::Note(first as usize, second as usize)
            }
            SelectorKind::Mixer(it) => Selector::Mixer(it as usize),
            SelectorKind::Effect(IndexPair { first, second }) => {
                Selector::Effect(first as usize, second as usize)
            }
            SelectorKind::Generator(it) => Selector::Generator(it as usize),
            SelectorKind::Placement(it) => Selector::Placement(it as usize),
            SelectorKind::Oscillator(IndexPair { first, second }) => {
                Selector::Oscillator(first as usize, second as usize)
            }
            SelectorKind::MixerMatrixCell(IndexPair { first, second }) => {
                Selector::MixerMatrixCell(first as usize, second as usize)
            }
        }
    }
}

fn pair(first: usize, second: usize) -> IndexPair {
    IndexPair {
        first: first as u32,
        second: second as u32,
    }
}
