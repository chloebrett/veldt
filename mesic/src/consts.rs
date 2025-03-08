use shared::model::PitchName;
use shared::model::ScaleValue;
use shared::types::Freq;

pub struct ReferencePitch<'a> {
    pub pitch_name: &'a PitchName,
    pub frequency: Freq,
}

pub const REFERENCE_PITCH: ReferencePitch<'static> = ReferencePitch {
    pitch_name: &PitchName {
        scale_value: ScaleValue::A,
        octave: 4,
    },
    frequency: 440.0,
};

pub const SAMPLE_RATE: i32 = 44_100;
