use crate::receiver::ActionReceiver;
use crate::{Action, FloatField};
use shared::model::ModDelayConfig;
use std::cmp::{max, min};

impl ActionReceiver for ModDelayConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetModDelayMinDepth(min_depth) => {
                let prev = self.min_depth;
                // Prevent min_depth from going above max_depth.
                self.min_depth = min(*min_depth, self.max_depth);
                Action::SetModDelayMinDepth(prev)
            }
            Action::SetModDelayMaxDepth(max_depth) => {
                let prev = self.max_depth;
                // Prevent max_depth from going below min_depth.
                self.max_depth = max(*max_depth, self.min_depth);
                Action::SetModDelayMaxDepth(prev)
            }
            Action::SetFloat(FloatField::LfoFreq, freq) => {
                let prev = self.freq;
                self.freq = *freq;
                Action::SetFloat(FloatField::LfoFreq, prev)
            }
            Action::SetModDelayLfoType(lfo_type) => {
                let prev = self.lfo_type;
                self.lfo_type = *lfo_type;
                Action::SetModDelayLfoType(prev)
            }
            _ => return None,
        })
    }
}
