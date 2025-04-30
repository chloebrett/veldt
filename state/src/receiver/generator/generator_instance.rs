use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::{GeneratorInstance, GeneratorType};

impl ActionReceiver for GeneratorInstance {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        if let Some(undo) = self.meta.apply(action) {
            return Some(undo);
        }

        match &mut self.kind {
            GeneratorType::SimpleWave(config) => config.apply(action),
            GeneratorType::Noise(_) => todo!(),
            GeneratorType::SubSynth(_) => todo!(),
        }
    }
}
