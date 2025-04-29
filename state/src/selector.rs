use shared::action_proto::{
    SelectorProto, selector_proto::IndexPair, selector_proto::Kind as SelectorKind,
};

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
        }
    }
}

fn pair(first: usize, second: usize) -> IndexPair {
    IndexPair {
        first: first as u32,
        second: second as u32,
    }
}
