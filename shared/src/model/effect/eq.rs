use crate::{
    pmodel::{EqConfigProto, EqTypeProto},
    types::{Freq, GainDB, KnobPosition},
};
use local_macro::{FromProto, IntoProto};
use strum::{Display, EnumIter, EnumString};

#[derive(Clone, Debug, EnumIter, EnumString, Display, PartialEq, FromProto, IntoProto)]
pub enum EqType {
    #[strum(to_string = "Simple Resonator")]
    SimpleResonator,
    #[strum(to_string = "Smith-Angell Resonator")]
    SmithAngellResonator,
    #[strum(to_string = "Simple First-Order Low Pass")]
    SimpleFirstOrderLowPass,
    #[strum(to_string = "Simple First-Order High Pass")]
    SimpleFirstOrderHighPass,
    #[strum(to_string = "Simple Second-Order Low Pass")]
    SimpleSecondOrderLowPass,
    #[strum(to_string = "Simple Second-Order High Pass")]
    SimpleSecondOrderHighPass,
    #[strum(to_string = "Simple Second-Order Resonator")]
    SimpleSecondOrderResonator,
    #[strum(to_string = "Simple Second-Order Band Stop")]
    SimpleSecondOrderBandStop,
    #[strum(to_string = "First-Order All-Pass")]
    FirstOrderAllPass,
    #[strum(to_string = "Second-Order All-Pass")]
    SecondOrderAllPass,
    #[strum(to_string = "Linkwitz-Riley Second-Order Low Pass")]
    LinkwitzRileySecondOrderLowPass,
    #[strum(to_string = "Linkwitz-Riley Second-Order High Pass")]
    LinkwitzRileySecondOrderHighPass,
    #[strum(to_string = "Parametric Second-Order (non-const Q)")]
    ParametricSecondOrderNonConstantQ,
    #[strum(to_string = "Parametric Second-Order (const Q)")]
    ParametricSecondOrderConstantQ,
    #[strum(to_string = "First-Order All-Pole")]
    FirstOrderAllPole,
    #[strum(to_string = "First-Order Low Shelf")]
    LowShelvingFirstOrder,
    #[strum(to_string = "First-Order High Shelf")]
    HighShelvingFirstOrder,
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct EqConfig {
    #[proto_enum]
    pub kind: EqType,

    pub fc: Freq,

    pub gain: GainDB,

    pub q: KnobPosition,
}

impl Default for EqConfig {
    fn default() -> Self {
        Self {
            kind: EqType::ParametricSecondOrderNonConstantQ,
            fc: 1000.0,
            gain: 0.0,
            q: 1.0,
        }
    }
}
