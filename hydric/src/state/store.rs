use super::{Action, reducer};
use shared::model::{Scale, ScaleValue};

pub struct Store {
    pub key: ScaleValue,
    pub scale: Scale,
}

impl Default for Store {
    fn default() -> Self {
        Store {
            key: ScaleValue::A,
            scale: Scale::Chromatic,
        }
    }
}

impl Store {
    pub fn dispatch(&mut self, action: Action) {
        reducer(self, action)
    }
}
