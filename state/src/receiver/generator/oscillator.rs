use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, Selector, TypeField};
use shared::model::SubSynthConfig;

impl ActionReceiver for SubSynthConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChildIndexed(
                Selector::Oscillator(generator_index, oscillator_index),
                TypeField::Wave(wave),
            ) => {
                let osc = self.oscillators.get_mut(*oscillator_index)?;
                let prev = osc.wave;
                osc.wave = *wave;
                Action::SetChildIndexed(
                    Selector::Oscillator(*generator_index, *oscillator_index),
                    TypeField::Wave(prev),
                )
            }
            Action::SetFloatIndexed(
                Selector::Oscillator(generator_index, oscillator_index),
                FloatField::Volume,
                volume,
            ) => {
                let osc = self.oscillators.get_mut(*oscillator_index)?;
                let prev = osc.volume;
                osc.volume = *volume;
                Action::SetFloatIndexed(
                    Selector::Oscillator(*generator_index, *oscillator_index),
                    FloatField::Volume,
                    prev,
                )
            }
            Action::SetFloatIndexed(
                Selector::Oscillator(generator_index, oscillator_index),
                FloatField::Pan,
                pan,
            ) => {
                let osc = self.oscillators.get_mut(*oscillator_index)?;
                let prev = osc.pan;
                osc.pan = *pan;
                Action::SetFloatIndexed(
                    Selector::Oscillator(*generator_index, *oscillator_index),
                    FloatField::Pan,
                    prev,
                )
            }
            Action::SetFloatIndexed(
                Selector::Oscillator(generator_index, oscillator_index),
                FloatField::FineDetune,
                fine_detune,
            ) => {
                let osc = self.oscillators.get_mut(*oscillator_index)?;
                let prev = osc.fine_detune;
                osc.fine_detune = *fine_detune;
                Action::SetFloatIndexed(
                    Selector::Oscillator(*generator_index, *oscillator_index),
                    FloatField::FineDetune,
                    prev,
                )
            }
            Action::SetFloatIndexed(
                Selector::Oscillator(generator_index, oscillator_index),
                FloatField::CoarseDetune,
                coarse_detune,
            ) => {
                let osc = self.oscillators.get_mut(*oscillator_index)?;
                let prev = osc.coarse_detune;
                osc.coarse_detune = *coarse_detune;
                Action::SetFloatIndexed(
                    Selector::Oscillator(*generator_index, *oscillator_index),
                    FloatField::CoarseDetune,
                    prev,
                )
            }
            _ => return None,
        })
    }
}
