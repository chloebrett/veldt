use strum::{Display, EnumString};

/// Fields of type u32.
/// Used to distinguish *which* field of this type is being referred to.
#[derive(EnumString, Display, PartialEq, Clone, Debug)]
pub enum UintField {
    OscCount,
    OversampleFactor,
    MinDepth,
    MaxDepth,
    PolyphonyLimit,
    VisualPlacement,
}
