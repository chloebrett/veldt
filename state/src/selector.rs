use crate::action_proto::{SelectorProto, selector_proto::Kind as SelectorKind};

#[derive(Clone, Debug, PartialEq)]
pub enum Selector {
    Root,
    Track(/* track_index */ usize),
    Note(/* track_index */ usize, /* note_index */ usize),
    Mixer(/* mixer_index */ usize),
    Effect(/* mixer_index */ usize, /* effect_index */ usize),
    Generator(/* generator_index */ usize),
    TrackPlacement(/* track_placement_index */ usize),
}

impl From<Selector> for SelectorProto {
    fn from(other: Selector) -> SelectorProto {
        SelectorProto {
            kind: Some(match other {
                Selector::Root => SelectorKind::Root(0),
                Selector::Generator(it) => SelectorKind::Generator(it as u32),
                _ => todo!(),
            }),
        }
    }
}

impl From<SelectorProto> for Selector {
    fn from(other: SelectorProto) -> Selector {
        match other.kind.unwrap() {
            SelectorKind::Root(..) => Selector::Root,
            SelectorKind::Generator(it) => Selector::Generator(it as usize),
        }
    }
}
