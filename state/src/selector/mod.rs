use crate::{StoreData, receiver::ActionReceiver};
use shared::action_proto::selector_proto::IndexTriple;
use shared::action_proto::{
    SelectorProto, selector_proto::IndexPair, selector_proto::Kind as SelectorKind,
};
use shared::model::GeneratorId;

mod effect;
mod envelope;
mod generator;
mod generator_effect;
mod lfo;
mod mixer;
mod mixer_matrix_cell;
mod note;
mod oscillator;
mod placement;
mod root;
mod sample;
mod track;

pub use effect::*;
pub use envelope::*;
pub use generator::*;
pub use generator_effect::*;
pub use lfo::*;
pub use mixer::*;
pub use mixer_matrix_cell::*;
pub use note::*;
pub use oscillator::*;
pub use placement::*;
pub use root::*;
pub use sample::*;
pub use track::*;

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

// Enum version of the selector.
// TODO: hide the visibility of this. We will still use it internally to efficiently represent a
// generic selector.
// Maybe rename to SelectorEnum, then rename SelectorTrait to Selector.
#[derive(Clone, Debug, PartialEq)]
pub enum Selector {
    Root,
    Track(/* track_index */ usize),
    Note(/* track_index */ usize, /* note_index */ usize),
    Mixer(/* mixer_index */ usize),
    Effect(/* mixer_index */ usize, /* effect_index */ usize),
    Generator(GeneratorId),
    Placement(PlacementId),
    Sample(/* sample_index */ usize),
    Oscillator(GeneratorId, /* oscillator_index */ usize),
    MixerMatrixCell(/* row */ usize, /* col */ usize),
    Lfo(GeneratorId, /* lfo_index */ usize),
    Envelope(GeneratorId, /* envelope_index */ usize),
    GeneratorEffect(GeneratorId, /* oscillator_index */ usize),
    ModMatrixCell(GeneratorId, /* row */ usize, /* col */ usize),
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
                Selector::Generator(it) => SelectorKind::Generator(it.into()),
                Selector::Placement(it) => SelectorKind::Placement(it as u32),
                Selector::Sample(it) => SelectorKind::Sample(it as u32),
                Selector::Oscillator(first, second) => {
                    SelectorKind::Oscillator(pair(*first, second))
                }
                Selector::MixerMatrixCell(first, second) => {
                    SelectorKind::MixerMatrixCell(pair(first, second))
                }
                Selector::Lfo(first, second) => SelectorKind::Lfo(pair(*first, second)),
                Selector::GeneratorEffect(first, second) => {
                    SelectorKind::GeneratorEffect(pair(*first, second))
                }
                Selector::Envelope(first, second) => SelectorKind::Envelope(pair(*first, second)),
                Selector::ModMatrixCell(first, second, third) => {
                    SelectorKind::ModMatrixCell(triple(*first, second, third))
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
            SelectorKind::Generator(it) => Selector::Generator(it.into()),
            SelectorKind::Placement(it) => Selector::Placement(it as usize),
            SelectorKind::Sample(it) => Selector::Sample(it as usize),
            SelectorKind::Oscillator(IndexPair { first, second }) => {
                Selector::Oscillator(first.into(), second as usize)
            }
            SelectorKind::MixerMatrixCell(IndexPair { first, second }) => {
                Selector::MixerMatrixCell(first as usize, second as usize)
            }
            SelectorKind::Lfo(IndexPair { first, second }) => {
                Selector::Lfo(first.into(), second as usize)
            }
            SelectorKind::GeneratorEffect(IndexPair { first, second }) => {
                Selector::GeneratorEffect(first.into(), second as usize)
            }
            SelectorKind::Envelope(IndexPair { first, second }) => {
                Selector::Envelope(first.into(), second as usize)
            }
            SelectorKind::ModMatrixCell(IndexTriple {
                first,
                second,
                third,
            }) => Selector::ModMatrixCell(first.into(), second as usize, third as usize),
        }
    }
}

fn pair(first: usize, second: usize) -> IndexPair {
    IndexPair {
        first: first as u32,
        second: second as u32,
    }
}

fn triple(first: usize, second: usize, third: usize) -> IndexTriple {
    IndexTriple {
        first: first as u32,
        second: second as u32,
        third: third as u32,
    }
}
