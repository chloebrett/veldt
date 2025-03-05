use shared::types::*;
use shared::model::scale::Scale;
use crate::ScaleValue;

pub fn get_scale_values(scale: Scale, key: ScaleValue) -> Vec<ScaleValue> {
    let pitch_values = match scale {
        Scale::Chromatic => vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        Scale::Major => vec![0, 2, 3, 5, 6, 8, 10],
        Scale::HarmonicMinor => vec![0, 2, 3, 5, 7, 8, 10]
    }; 

    let key_value = <ScaleValue as Into<PitchValue>>::into(key);
    pitch_values.iter()
       .map(|pitch_value| {ScaleValue::from(pitch_value + key_value)})
       .collect()
}
