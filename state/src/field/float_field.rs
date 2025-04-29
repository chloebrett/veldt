use strum::{Display, EnumString};

/// Fields of type f32.
/// Used to distinguish *which* field of this type is being referred to.
#[derive(EnumString, Display, PartialEq, Clone, Debug)]
pub enum FloatField {
    Bpm,
    Volume,
    Offset,
    Duration,
    Pan,
    Detune,
    DelayMs,
    Wet,
    Fc,
    Q,
    Gain,
    Threshold,
    AttackMs,
    ReleaseMs,
    Ratio,
    LfoFreq,
    Feedback,
    ModFactor,
}
