use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, TypeField, UintField};
use shared::model::Oscillator;

impl ActionReceiver for Oscillator {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::Wave(wave)) => {
                let prev = self.wave;
                self.wave = *wave;
                Action::SetChild(TypeField::Wave(prev))
            }
            Action::SetFloat(FloatField::Volume, volume) => {
                let prev = self.volume;
                self.volume = *volume;
                Action::SetFloat(FloatField::Volume, prev)
            }
            Action::SetFloat(FloatField::Pan, pan) => {
                let prev = self.pan;
                self.pan = *pan;
                Action::SetFloat(FloatField::Pan, prev)
            }
            Action::SetFloat(FloatField::OscillatorCoarseDetune, detune) => {
                let prev = self.coarse_detune;
                self.coarse_detune = *detune;
                Action::SetFloat(FloatField::OscillatorCoarseDetune, prev)
            }
            Action::SetFloat(FloatField::OscillatorFineDetune, detune) => {
                let prev = self.fine_detune;
                self.fine_detune = *detune;
                Action::SetFloat(FloatField::OscillatorFineDetune, prev)
            }
            Action::SetFloat(FloatField::UnisonDetune, detune) => {
                let prev = self.unison_detune;
                self.unison_detune = *detune;
                Action::SetFloat(FloatField::UnisonDetune, prev)
            }
            Action::SetUint(UintField::OscCount, osc_count) => {
                let prev = self.osc_count;
                self.osc_count = *osc_count;
                Action::SetUint(UintField::OscCount, prev)
            }
            _ => return None,
        })
    }
}
