use crate::pmodel::PolyphonyModeProto;
use local_macro::{FromProto, IntoProto};
use strum::{Display, EnumIter, EnumString};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumString,
    Display,
    EnumIter,
    IntoProto,
    FromProto,
    Hash,
    Default,
)]
pub enum PolyphonyMode {
    Monophonic,

    MonophonicLegato,

    #[default]
    Polyphonic,
}
