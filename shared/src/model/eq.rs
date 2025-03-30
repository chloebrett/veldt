use crate::{
    pmodel::{EqConfigProto, EqTypeProto},
    types::{Freq, KnobPosition},
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
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct EqConfig {
    #[proto_enum]
    pub kind: EqType,

    pub fc: Freq,

    pub q: KnobPosition,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_eq_type_to_proto_and_back() {
        let eq_type = EqType::SimpleResonator;
        let proto: EqTypeProto = eq_type.clone().into();
        let result: EqType = proto.into();
        assert_eq!(eq_type, result);
    }

    #[test]
    fn convert_eq_config_to_proto_and_back() {
        let eq_config = EqConfig {
            kind: EqType::SimpleResonator,
            fc: 100.0,
            q: 1.0 
        };
        let proto: EqConfigProto = eq_config.clone().into();
        let result: EqConfig = proto.into(); 
        assert_eq!(eq_config, result);
    }
}
