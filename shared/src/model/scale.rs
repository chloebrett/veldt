use strum::{Display, EnumString};
use crate::{pmodel::*, types::PitchValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
pub enum Scale {
    Chromatic = ScaleNotes::new([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]), 
    Major = ScaleNotes::new([0, 2, 4, 5, 7, 9, 11]),
    #[strum(serialize = "Harmonic Minor")]
    HarmonicMinor = ScaleNotes::new([0, 2, 3, 5, 7, 8, 10])
}

pub type ScaleNotes = Vec<PitchValue>;
