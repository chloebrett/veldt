use crate::{
    pmodel::{EqConfigProto, EqTypeProto},
    types::{Freq, GainDB, KnobPosition},
};
use local_macro::{FromProto, IntoProto};
use strum::{Display, EnumIter, EnumString};

#[derive(Clone, Debug, EnumIter, EnumString, Display, PartialEq, FromProto, IntoProto)]
pub enum EqType {
    SimpleResonator,
    SmithAngellResonator,
    SimpleFirstOrderLowPass,
    SimpleFirstOrderHighPass,
    SimpleSecondOrderLowPass,
    SimpleSecondOrderHighPass,
    SimpleSecondOrderResonator,
    SimpleSecondOrderBandStop,
    ButterworthLowPass,
    ButterworthHighPass,
    ButterworthResonator,
    ButterworthBandStop,
    FirstOrderAllPass,
    SecondOrderAllPass,
    LinkwitzRileySecondOrderLowPass,
    LinkwitzRileySecondOrderHighPass,
    LowShelvingFirstOrder,
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct EqConfig {
    #[proto_enum]
    pub kind: EqType,

    pub fc: Freq,

    pub gain: GainDB,

    pub q: KnobPosition,
}
