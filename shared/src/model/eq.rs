use strum::{Display, EnumIter, EnumString};
use crate::types::{Freq, KnobPosition};

#[derive(Clone, EnumIter, EnumString, Display, PartialEq)]
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

pub struct EqConfig {
    pub kind: EqType,

    pub fc: Freq,

    pub q: KnobPosition,
}

