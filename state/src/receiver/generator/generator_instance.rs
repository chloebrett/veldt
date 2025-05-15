use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::{Generator, GeneratorInstance};

impl ActionReceiver for GeneratorInstance {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        if let Some(undo) = self.meta.apply(action) {
            return Some(undo);
        }

        match &mut self.it {
            Generator::SimpleWave(config) => config.apply(action),
            Generator::Noise(_) => todo!(),
            Generator::Stingray(_) => todo!(),
        }
    }
}
