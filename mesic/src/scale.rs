use crate::ScaleValue;
use shared::model::scale::Scale;
use shared::types::*;

pub fn create_scale_values(scale: Scale, key: ScaleValue) -> Vec<ScaleValue> {
    let pitch_values = match scale {
        Scale::Chromatic => vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        Scale::Major => vec![0, 2, 4, 5, 7, 9, 11],
        Scale::NaturalMinor => vec![0, 2, 3, 5, 7, 8, 10],
        Scale::HarmonicMinor => vec![0, 2, 3, 5, 7, 8, 11],
        Scale::Pentatonic => vec![0, 2, 4, 7, 9],
    };

    let key_value = <ScaleValue as Into<PitchValue>>::into(key);
    pitch_values
        .iter()
        .map(|pitch_value| ScaleValue::from(pitch_value + key_value))
        .collect()
}
