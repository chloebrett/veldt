use crate::pmodel::*;
use strum::{Display, EnumIter, EnumString};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString, Display, EnumIter,
)]
pub enum WaveType {
    Sine,
    Square,
    Saw,
    Triangle, // TODO: also add a generator for white noise - but it's not constrained by freq.
}

impl From<WaveType> for i32 {
    fn from(item: WaveType) -> i32 {
        item as i32
    }
}

impl From<i32> for WaveType {
    fn from(item: i32) -> WaveType {
        item.into()
    }
}

impl From<WaveTypeProto> for WaveType {
    fn from(item: WaveTypeProto) -> Self {
        match item {
            WaveTypeProto::Unknown => panic!(""),
            WaveTypeProto::Sine => WaveType::Sine,
            WaveTypeProto::Square => WaveType::Square,
            WaveTypeProto::Saw => WaveType::Saw,
            WaveTypeProto::Triangle => WaveType::Triangle,
        }
    }
}

impl From<WaveType> for WaveTypeProto {
    fn from(item: WaveType) -> Self {
        match item {
            WaveType::Sine => WaveTypeProto::Sine,
            WaveType::Square => WaveTypeProto::Square,
            WaveType::Saw => WaveTypeProto::Saw,
            WaveType::Triangle => WaveTypeProto::Triangle,
        }
    }
}
