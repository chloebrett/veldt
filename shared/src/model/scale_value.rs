use crate::pmodel::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ScaleValue {
    A,
    ASharp,
    B,
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
}

impl From<ScaleValueProto> for ScaleValue {
    fn from(item: ScaleValueProto) -> Self {
        match item {
            ScaleValueProto::UnknownScaleValue => panic!(""),
            ScaleValueProto::AScaleValue => ScaleValue::A,
            ScaleValueProto::ASharpScaleValue => ScaleValue::ASharp,
            ScaleValueProto::BScaleValue => ScaleValue::B,
            ScaleValueProto::CScaleValue => ScaleValue::C,
            ScaleValueProto::CSharpScaleValue => ScaleValue::CSharp,
            ScaleValueProto::DScaleValue => ScaleValue::D,
            ScaleValueProto::DSharpScaleValue => ScaleValue::DSharp,
            ScaleValueProto::EScaleValue => ScaleValue::E,
            ScaleValueProto::FScaleValue => ScaleValue::F,
            ScaleValueProto::FSharpScaleValue => ScaleValue::FSharp,
            ScaleValueProto::GScaleValue => ScaleValue::G,
            ScaleValueProto::GSharpScaleValue => ScaleValue::GSharp
        }
    }
}

impl From<ScaleValue> for ScaleValueProto {
    fn from(item: ScaleValue) -> Self {
        match item {
            ScaleValue::A => ScaleValueProto::AScaleValue,
            ScaleValue::ASharp => ScaleValueProto::ASharpScaleValue,
            ScaleValue::B => ScaleValueProto::BScaleValue,
            ScaleValue::C => ScaleValueProto::CScaleValue,
            ScaleValue::CSharp => ScaleValueProto::CSharpScaleValue,
            ScaleValue::D => ScaleValueProto::DScaleValue,
            ScaleValue::DSharp => ScaleValueProto::DSharpScaleValue,
            ScaleValue::E => ScaleValueProto::EScaleValue,
            ScaleValue::F => ScaleValueProto::FScaleValue,
            ScaleValue::FSharp => ScaleValueProto::FSharpScaleValue,
            ScaleValue::G => ScaleValueProto::GScaleValue,
            ScaleValue::GSharp => ScaleValueProto::GSharpScaleValue
        }
    }
}
