use crate::pmodel::AntiAliasingModeProto;
use local_macro::{FromProto, IntoProto};
use strum::{Display, EnumIter, EnumString};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, EnumIter, IntoProto, FromProto, Hash,
)]
pub enum AntiAliasingMode {
    // No anti-aliasing. Uses naive waves without oversampling. Produces artifacts for waves like
    // square and saw, especially at high frequencies.
    Off,

    // Reduces aliasing by oversampling by the given factor, low passing, then downsampling. This uses additional computation
    // power.
    Oversample,

    // Avoids aliasing entirely by constructing the waveform additively. This results in zero
    // aliasing but is the most computationally expensive.
    Additive,
}
