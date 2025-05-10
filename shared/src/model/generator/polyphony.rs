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

    #[strum(to_string = "Monophonic Legato")]
    MonophonicLegato,

    #[default]
    Polyphonic,
}
