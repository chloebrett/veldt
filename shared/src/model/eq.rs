use crate::{
    pmodel::{EqConfigProto, EqTypeProto},
    types::{Freq, KnobPosition},
};
use local_macro::{FromProto, IntoProto};
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
    FirstOrderAllPass,
    SecondOrderAllPass,
}

impl From<EqTypeProto> for EqType {
    fn from(item: EqTypeProto) -> Self {
        match item {
            EqTypeProto::UnknownEqType => panic!(""),
            EqTypeProto::SimpleResonatorEqType => EqType::SimpleResonator,
            EqTypeProto::SmithAngellResonatorEqType => EqType::SmithAngellResonator,
            EqTypeProto::SimpleFirstOrderLowPassEqType => EqType::SimpleFirstOrderLowPass,
            EqTypeProto::SimpleFirstOrderHighPassEqType => EqType::SimpleFirstOrderHighPass,
            EqTypeProto::SimpleSecondOrderLowPassEqType => EqType::SimpleSecondOrderLowPass,
            EqTypeProto::SimpleSecondOrderHighPassEqType => EqType::SimpleSecondOrderHighPass,
            EqTypeProto::SimpleSecondOrderResonatorEqType => EqType::SimpleSecondOrderResonator,
            EqTypeProto::SimpleSecondOrderBandStopEqType => EqType::SimpleSecondOrderBandStop,
            EqTypeProto::ButterworthLowPassEqType => EqType::ButterworthLowPass,
            EqTypeProto::ButterworthHighPassEqType => EqType::ButterworthHighPass,
            EqTypeProto::ButterworthResonatorEqType => EqType::ButterworthResonator,
            EqTypeProto::ButterworthBandStopEqType => EqType::ButterworthBandStop,
            EqTypeProto::FirstOrderAllPassEqType => EqType::FirstOrderAllPass,
            EqTypeProto::SecondOrderAllPassEqType => EqType::SecondOrderAllPass,
        }
    }
}

impl From<EqType> for EqTypeProto {
    fn from(item: EqType) -> Self {
        match item {
            EqType::SimpleResonator => EqTypeProto::SimpleResonatorEqType,
            EqType::SmithAngellResonator => EqTypeProto::SmithAngellResonatorEqType,
            EqType::SimpleFirstOrderLowPass => EqTypeProto::SimpleFirstOrderLowPassEqType,
            EqType::SimpleFirstOrderHighPass => EqTypeProto::SimpleFirstOrderHighPassEqType,
            EqType::SimpleSecondOrderLowPass => EqTypeProto::SimpleSecondOrderLowPassEqType,
            EqType::SimpleSecondOrderHighPass => EqTypeProto::SimpleSecondOrderHighPassEqType,
            EqType::SimpleSecondOrderResonator => EqTypeProto::SimpleSecondOrderResonatorEqType,
            EqType::SimpleSecondOrderBandStop => EqTypeProto::SimpleSecondOrderBandStopEqType,
            EqType::ButterworthLowPass => EqTypeProto::ButterworthLowPassEqType,
            EqType::ButterworthHighPass => EqTypeProto::ButterworthHighPassEqType,
            EqType::ButterworthResonator => EqTypeProto::ButterworthResonatorEqType,
            EqType::ButterworthBandStop => EqTypeProto::ButterworthBandStopEqType,
            EqType::FirstOrderAllPass => EqTypeProto::FirstOrderAllPassEqType,
            EqType::SecondOrderAllPass => EqTypeProto::SecondOrderAllPassEqType,
        }
    }
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct EqConfig {
    #[proto_enum]
    pub kind: EqType,

    pub fc: Freq,

    pub q: KnobPosition,
}
