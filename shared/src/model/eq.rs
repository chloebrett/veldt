use crate::types::{Freq, KnobPosition};
use strum::{Display, EnumIter, EnumString};

#[derive(Clone, Debug, EnumIter, EnumString, Display, PartialEq)]
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct EqConfig {
    pub kind: EqType,

    pub fc: Freq,

    pub q: KnobPosition,
}
