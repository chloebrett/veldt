use crate::receiver::ActionReceiver;
use crate::{Action, IndexField, TypeField};
use shared::model::SamplePlacement;

impl ActionReceiver for SamplePlacement {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::SampleId(sample_id)) => {
                let prev = self.sample_id;
                self.sample_id = *sample_id;
                Action::SetChild(TypeField::SampleId(prev))
            }
            Action::SetIndex(IndexField::Mixer(mixer_channel)) => {
                let prev = self.mixer_channel;
                self.mixer_channel = *mixer_channel;
                Action::SetIndex(IndexField::Mixer(prev))
            }
            _ => return None,
        })
    }
}
