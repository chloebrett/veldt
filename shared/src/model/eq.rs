use crate::{
    pmodel::{EqConfigProto, EqTypeProto},
    types::{Freq, KnobPosition},
};
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

impl From<EqTypeProto> for EqType {
    fn from(item: EqTypeProto) -> Self {
        match item {
            EqTypeProto::UnknownEq => panic!(""),
            EqTypeProto::SimpleResonatorEq => EqType::SimpleResonator,
            EqTypeProto::SmithAngellResonatorEq => EqType::SmithAngellResonator,
            EqTypeProto::SimpleFirstOrderLowPassEq => EqType::SimpleFirstOrderLowPass,
            EqTypeProto::SimpleFirstOrderHighPassEq => EqType::SimpleFirstOrderHighPass,
            EqTypeProto::SimpleSecondOrderLowPassEq => EqType::SimpleSecondOrderLowPass,
            EqTypeProto::SimpleSecondOrderHighPassEq => EqType::SimpleSecondOrderHighPass,
            EqTypeProto::SimpleSecondOrderResonatorEq => EqType::SimpleSecondOrderResonator,
            EqTypeProto::SimpleSecondOrderBandStopEq => EqType::SimpleSecondOrderBandStop,
            EqTypeProto::ButterworthLowPassEq => EqType::ButterworthLowPass,
            EqTypeProto::ButterworthHighPassEq => EqType::ButterworthHighPass,
            EqTypeProto::ButterworthResonatorEq => EqType::ButterworthResonator,
            EqTypeProto::ButterworthBandStopEq => EqType::ButterworthBandStop,
        }
    }
}

impl From<EqType> for EqTypeProto {
    fn from(item: EqType) -> Self {
        match item {
            EqType::SimpleResonator => EqTypeProto::SimpleResonatorEq,
            EqType::SmithAngellResonator => EqTypeProto::SmithAngellResonatorEq,
            EqType::SimpleFirstOrderLowPass => EqTypeProto::SimpleFirstOrderLowPassEq,
            EqType::SimpleFirstOrderHighPass => EqTypeProto::SimpleFirstOrderHighPassEq,
            EqType::SimpleSecondOrderLowPass => EqTypeProto::SimpleSecondOrderLowPassEq,
            EqType::SimpleSecondOrderHighPass => EqTypeProto::SimpleSecondOrderHighPassEq,
            EqType::SimpleSecondOrderResonator => EqTypeProto::SimpleSecondOrderResonatorEq,
            EqType::SimpleSecondOrderBandStop => EqTypeProto::SimpleSecondOrderBandStopEq,
            EqType::ButterworthLowPass => EqTypeProto::ButterworthLowPassEq,
            EqType::ButterworthHighPass => EqTypeProto::ButterworthHighPassEq,
            EqType::ButterworthResonator => EqTypeProto::ButterworthResonatorEq,
            EqType::ButterworthBandStop => EqTypeProto::ButterworthBandStopEq,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EqConfig {
    pub kind: EqType,

    pub fc: Freq,

    pub q: KnobPosition,
}

impl From<EqConfigProto> for EqConfig {
    fn from(item: EqConfigProto) -> Self {
        EqConfig {
            kind: item.kind().into(),
            fc: item.fc,
            q: item.q,
        }
    }
}

impl From<EqConfig> for EqConfigProto {
    fn from(item: EqConfig) -> Self {
        EqConfigProto {
            kind: Into::<EqTypeProto>::into(item.kind).into(),
            fc: item.fc,
            q: item.q,
        }
    }
}
